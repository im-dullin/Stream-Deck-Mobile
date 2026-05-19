//! WebSocket server. Two entry-point flows for the first message:
//!  - `Hello` (token-bearing returning client) → validate against `PairingDb`
//!  - `PairRequest` (new client) → register pending, notify UI via event channel,
//!    await user approval, issue new token + save pairing
//!
//! After auth, enters the standard loop: dispatch `ButtonPress` to actions,
//! forward `ProfileUpdate` broadcasts, periodic `Ping` keep-alive.

use crate::actions;
use crate::pairings::{self, Pairing, PairingDb};
use crate::protocol::{ClientMessage, Profile, ServerMessage, PROTOCOL_VERSION};
use anyhow::{Context, Result, bail};
use futures_util::{SinkExt, StreamExt};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{broadcast, mpsc, oneshot, Mutex, RwLock};
use tokio_tungstenite::accept_async;
use tokio_tungstenite::tungstenite::Message;

const HELLO_TIMEOUT: Duration = Duration::from_secs(5);
const PAIR_TIMEOUT: Duration = Duration::from_secs(60);
const PING_INTERVAL: Duration = Duration::from_secs(30);
const BROADCAST_CAPACITY: usize = 32;

pub enum PairOutcome {
    Approved,
    Rejected,
}

#[derive(Debug, Clone)]
pub enum AgentEvent {
    PairRequested {
        request_id: String,
        device_id: String,
        device_name: String,
        peer: String,
    },
}

#[derive(Clone)]
pub struct SharedState {
    pub profile: Arc<RwLock<Profile>>,
    pub agent_name: Arc<String>,
    pub profile_tx: broadcast::Sender<Profile>,
    pub pairings: Arc<RwLock<PairingDb>>,
    pub pending_pairs: Arc<Mutex<HashMap<String, oneshot::Sender<PairOutcome>>>>,
    pub event_tx: mpsc::Sender<AgentEvent>,
}

impl SharedState {
    pub fn new(
        profile: Profile,
        pairings: PairingDb,
        agent_name: String,
        event_tx: mpsc::Sender<AgentEvent>,
    ) -> Self {
        let (profile_tx, _) = broadcast::channel(BROADCAST_CAPACITY);
        Self {
            profile: Arc::new(RwLock::new(profile)),
            agent_name: Arc::new(agent_name),
            profile_tx,
            pairings: Arc::new(RwLock::new(pairings)),
            pending_pairs: Arc::new(Mutex::new(HashMap::new())),
            event_tx,
        }
    }
}

pub async fn start(state: SharedState, port: u16) -> Result<SocketAddr> {
    let addr: SocketAddr = format!("0.0.0.0:{}", port).parse()?;
    let listener = TcpListener::bind(addr)
        .await
        .with_context(|| format!("bind {}", addr))?;
    let bound = listener.local_addr()?;
    tracing::info!(addr = %bound, "ws server listening");

    tokio::spawn(async move {
        loop {
            match listener.accept().await {
                Ok((stream, peer)) => {
                    let state = state.clone();
                    tokio::spawn(async move {
                        if let Err(e) = handle_connection(stream, peer, state).await {
                            tracing::warn!(peer = %peer, error = ?e, "connection closed");
                        }
                    });
                }
                Err(e) => {
                    tracing::error!(error = ?e, "accept failed");
                    tokio::time::sleep(Duration::from_millis(200)).await;
                }
            }
        }
    });

    Ok(bound)
}

async fn handle_connection(
    stream: TcpStream,
    peer: SocketAddr,
    state: SharedState,
) -> Result<()> {
    let ws = accept_async(stream).await.context("ws handshake")?;
    let (mut tx, mut rx) = ws.split();

    let raw_first = match tokio::time::timeout(HELLO_TIMEOUT, rx.next()).await {
        Ok(Some(Ok(Message::Text(t)))) => t,
        Ok(Some(Ok(Message::Close(_)))) | Ok(None) => return Ok(()),
        Ok(Some(Ok(_))) => bail!("expected text frame"),
        Ok(Some(Err(e))) => return Err(e.into()),
        Err(_) => bail!("hello timeout"),
    };
    let first: ClientMessage =
        serde_json::from_str(&raw_first).context("parse first message")?;

    let device_name = match first {
        ClientMessage::Hello {
            protocol_version,
            device_id,
            device_name,
            token,
        } => {
            if protocol_version != PROTOCOL_VERSION {
                send(&mut tx, &ServerMessage::Error {
                    code: "version_mismatch".into(),
                    message: format!("agent uses protocol v{}", PROTOCOL_VERSION),
                }).await.ok();
                bail!("version mismatch ({} != {})", protocol_version, PROTOCOL_VERSION);
            }
            let paired = {
                let db = state.pairings.read().await;
                db.find(&device_id, &token).is_some()
            };
            if !paired {
                send(&mut tx, &ServerMessage::Error {
                    code: "not_paired".into(),
                    message: "device not paired or token revoked".into(),
                }).await.ok();
                bail!("not paired: {} ({})", device_name, device_id);
            }
            tracing::info!(peer = %peer, device_name, "client connected (paired)");
            device_name
        }
        ClientMessage::PairRequest {
            protocol_version,
            device_id,
            device_name,
        } => {
            if protocol_version != PROTOCOL_VERSION {
                send(&mut tx, &ServerMessage::Error {
                    code: "version_mismatch".into(),
                    message: format!("agent uses protocol v{}", PROTOCOL_VERSION),
                }).await.ok();
                bail!("version mismatch");
            }

            let request_id = uuid::Uuid::new_v4().to_string();
            let (resolve_tx, resolve_rx) = oneshot::channel();
            state
                .pending_pairs
                .lock()
                .await
                .insert(request_id.clone(), resolve_tx);

            let _ = state.event_tx.send(AgentEvent::PairRequested {
                request_id: request_id.clone(),
                device_id: device_id.clone(),
                device_name: device_name.clone(),
                peer: peer.to_string(),
            }).await;

            send(&mut tx, &ServerMessage::PairPending {
                request_id: request_id.clone(),
            }).await?;

            let outcome = tokio::time::timeout(PAIR_TIMEOUT, resolve_rx).await;
            state.pending_pairs.lock().await.remove(&request_id);

            match outcome {
                Ok(Ok(PairOutcome::Approved)) => {
                    let new_token = uuid::Uuid::new_v4().to_string();
                    let pairing = Pairing {
                        device_id: device_id.clone(),
                        device_name: device_name.clone(),
                        token: new_token.clone(),
                        paired_at_unix: pairings::now_unix(),
                    };
                    {
                        let mut db = state.pairings.write().await;
                        db.upsert(pairing);
                        pairings::save(&db).await?;
                    }
                    send(&mut tx, &ServerMessage::PairAccepted {
                        token: new_token,
                    }).await?;
                    tracing::info!(peer = %peer, device_name, "pair approved");
                    device_name
                }
                Ok(Ok(PairOutcome::Rejected)) => {
                    send(&mut tx, &ServerMessage::PairRejected {
                        reason: "rejected".into(),
                    }).await.ok();
                    bail!("pair rejected by user");
                }
                Err(_) => {
                    send(&mut tx, &ServerMessage::PairRejected {
                        reason: "timeout".into(),
                    }).await.ok();
                    bail!("pair timed out");
                }
                Ok(Err(_)) => {
                    send(&mut tx, &ServerMessage::PairRejected {
                        reason: "internal".into(),
                    }).await.ok();
                    bail!("pair channel closed");
                }
            }
        }
        _ => {
            send(&mut tx, &ServerMessage::Error {
                code: "expected_hello_or_pair".into(),
                message: "first message must be hello or pair_request".into(),
            }).await.ok();
            bail!("first message wrong type");
        }
    };

    let welcome = ServerMessage::Welcome {
        protocol_version: PROTOCOL_VERSION,
        agent_name: state.agent_name.to_string(),
        profile: state.profile.read().await.clone(),
    };
    send(&mut tx, &welcome).await?;

    let mut profile_rx = state.profile_tx.subscribe();
    let mut ping_interval = tokio::time::interval(PING_INTERVAL);
    ping_interval.tick().await;

    loop {
        tokio::select! {
            incoming = rx.next() => match incoming {
                Some(Ok(Message::Text(t))) => {
                    match serde_json::from_str::<ClientMessage>(&t) {
                        Ok(msg) => dispatch_client_message(msg, &state).await,
                        Err(e) => tracing::warn!(error = %e, raw = %t, "malformed client message"),
                    }
                }
                Some(Ok(Message::Close(_))) | None => break,
                Some(Ok(_)) => {}
                Some(Err(e)) => return Err(e.into()),
            },
            recv = profile_rx.recv() => match recv {
                Ok(updated) => send(&mut tx, &ServerMessage::ProfileUpdate { profile: updated }).await?,
                Err(broadcast::error::RecvError::Lagged(n)) => {
                    tracing::warn!(skipped = n, "profile broadcast lagged");
                }
                Err(broadcast::error::RecvError::Closed) => break,
            },
            _ = ping_interval.tick() => {
                if send(&mut tx, &ServerMessage::Ping).await.is_err() {
                    break;
                }
            }
        }
    }

    tracing::info!(peer = %peer, device_name, "client disconnected");
    Ok(())
}

async fn dispatch_client_message(msg: ClientMessage, state: &SharedState) {
    match msg {
        ClientMessage::ButtonPress { page_id, row, col } => {
            let action = {
                let profile = state.profile.read().await;
                profile
                    .pages
                    .iter()
                    .find(|p| p.id == page_id)
                    .and_then(|p| p.buttons.iter().find(|b| b.row == row && b.col == col))
                    .map(|b| b.action.clone())
            };
            match action {
                Some(a) => {
                    tokio::spawn(async move {
                        if let Err(e) = actions::execute(&a).await {
                            tracing::error!(error = ?e, "action failed");
                        }
                    });
                }
                None => tracing::warn!(page_id, row, col, "no button mapped"),
            }
        }
        ClientMessage::PageChange { .. }
        | ClientMessage::Pong
        | ClientMessage::Hello { .. }
        | ClientMessage::PairRequest { .. } => {
            // Hello/PairRequest mid-session ignored; page_change is observability-only.
        }
    }
}

async fn send<S>(tx: &mut S, msg: &ServerMessage) -> Result<()>
where
    S: SinkExt<Message> + Unpin,
    <S as futures_util::Sink<Message>>::Error: std::error::Error + Send + Sync + 'static,
{
    let json = serde_json::to_string(msg)?;
    tx.send(Message::Text(json))
        .await
        .map_err(|e| anyhow::anyhow!("ws send: {}", e))
}
