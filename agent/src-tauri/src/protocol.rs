//! Wire protocol between the PC agent and mobile deck clients.
//!
//! JSON over WebSocket. Types here MUST stay in sync with
//! `schema/protocol.ts` (TypeScript source of truth) and
//! `mobile/lib/protocol/messages.dart` (Dart counterpart).
//!
//! Field naming: Rust uses snake_case; wire format uses camelCase via
//! `rename_all = "camelCase"`. Enum variant tags are emitted in snake_case
//! to match the TS schema (`launch_app`, `button_press`, ...).

use serde::{Deserialize, Serialize};

pub const PROTOCOL_VERSION: u32 = 1;

// =====================================================================
// Domain
// =====================================================================

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Profile {
    pub id: String,
    pub name: String,
    pub default_page_id: String,
    pub pages: Vec<Page>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Page {
    pub id: String,
    pub name: String,
    pub rows: u32,
    pub cols: u32,
    pub buttons: Vec<Button>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Button {
    pub row: u32,
    pub col: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon_base64: Option<String>,
    pub action: Action,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case", rename_all_fields = "camelCase")]
pub enum Action {
    LaunchApp {
        app_path: String,
        app_name: String,
    },
    /// Opens a URL in the OS default handler (web, mailto, custom schemes).
    /// `display_name` is editor-only metadata; empty/None falls back to
    /// the URL's hostname.
    OpenUrl {
        url: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        display_name: Option<String>,
    },
    /// Spawns an arbitrary process on the host (Python scripts, shell scripts,
    /// node programs, ...). Arguments are passed positionally — no shell
    /// interpretation, so `|` and `>` aren't pipes; wrap such cases in a
    /// shell script if needed. `~/` is expanded to the user's home on the
    /// agent side.
    RunCommand {
        program: String,
        #[serde(default)]
        args: Vec<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        working_dir: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        display_name: Option<String>,
    },
    /// Sequentially executes up to N sub-actions. Sub-actions are expected
    /// to be non-`MultiAction`; the executor flattens defensively if nested.
    MultiAction {
        actions: Vec<Action>,
    },
}

// =====================================================================
// Client (mobile) -> Server (agent)
// =====================================================================

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case", rename_all_fields = "camelCase")]
pub enum ClientMessage {
    /// Returning client with an existing pairing token.
    Hello {
        protocol_version: u32,
        device_id: String,
        device_name: String,
        token: String,
    },
    /// New client asking the agent's user to approve a fresh pairing.
    PairRequest {
        protocol_version: u32,
        device_id: String,
        device_name: String,
    },
    ButtonPress {
        page_id: String,
        row: u32,
        col: u32,
    },
    PageChange {
        page_id: String,
    },
    Pong,
}

// =====================================================================
// Server (agent) -> Client (mobile)
// =====================================================================

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case", rename_all_fields = "camelCase")]
pub enum ServerMessage {
    Welcome {
        protocol_version: u32,
        agent_name: String,
        profile: Profile,
    },
    ProfileUpdate {
        profile: Profile,
    },
    /// Sent immediately after a `PairRequest` to indicate the agent is
    /// awaiting user approval. Carries an opaque request id (mostly for logs).
    PairPending {
        request_id: String,
    },
    /// Pairing was approved by the agent's user. Client should save `token`
    /// and use it on subsequent reconnects via `Hello`. A `Welcome` follows
    /// immediately on the same connection.
    PairAccepted {
        token: String,
    },
    /// Pairing was rejected or timed out. Connection will close after this.
    PairRejected {
        reason: String,
    },
    Ping,
    Error {
        code: String,
        message: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_profile() -> Profile {
        Profile {
            id: "default".into(),
            name: "Default".into(),
            default_page_id: "p1".into(),
            pages: vec![Page {
                id: "p1".into(),
                name: "Page 1".into(),
                rows: 3,
                cols: 5,
                buttons: vec![Button {
                    row: 0,
                    col: 0,
                    label: Some("Slack".into()),
                    icon_base64: None,
                    action: Action::LaunchApp {
                        app_path: "/Applications/Slack.app".into(),
                        app_name: "Slack".into(),
                    },
                }],
            }],
        }
    }

    #[test]
    fn profile_round_trip() {
        let p = sample_profile();
        let json = serde_json::to_string(&p).unwrap();
        let back: Profile = serde_json::from_str(&json).unwrap();
        assert_eq!(p, back);
    }

    #[test]
    fn profile_uses_camel_case_on_wire() {
        let p = sample_profile();
        let json = serde_json::to_string(&p).unwrap();
        assert!(json.contains("\"defaultPageId\""));
        assert!(!json.contains("\"default_page_id\""));
    }

    #[test]
    fn button_press_round_trip() {
        let m = ClientMessage::ButtonPress {
            page_id: "p1".into(),
            row: 1,
            col: 2,
        };
        let json = serde_json::to_string(&m).unwrap();
        assert!(json.contains("\"type\":\"button_press\""));
        assert!(json.contains("\"pageId\":\"p1\""));
        let back: ClientMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(m, back);
    }

    #[test]
    fn hello_round_trip() {
        let m = ClientMessage::Hello {
            protocol_version: PROTOCOL_VERSION,
            device_id: "d1".into(),
            device_name: "Pixel".into(),
            token: "tok".into(),
        };
        let json = serde_json::to_string(&m).unwrap();
        assert!(json.contains("\"protocolVersion\":1"));
        let back: ClientMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(m, back);
    }

    #[test]
    fn pair_request_round_trip() {
        let m = ClientMessage::PairRequest {
            protocol_version: PROTOCOL_VERSION,
            device_id: "d1".into(),
            device_name: "iPhone".into(),
        };
        let json = serde_json::to_string(&m).unwrap();
        assert!(json.contains("\"type\":\"pair_request\""));
        assert!(json.contains("\"deviceName\":\"iPhone\""));
        let back: ClientMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(m, back);
    }

    #[test]
    fn pair_accepted_round_trip() {
        let m = ServerMessage::PairAccepted { token: "new-tok".into() };
        let json = serde_json::to_string(&m).unwrap();
        assert!(json.contains("\"type\":\"pair_accepted\""));
        let back: ServerMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(m, back);
    }

    #[test]
    fn welcome_round_trip() {
        let m = ServerMessage::Welcome {
            protocol_version: PROTOCOL_VERSION,
            agent_name: "host".into(),
            profile: sample_profile(),
        };
        let json = serde_json::to_string(&m).unwrap();
        let back: ServerMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(m, back);
    }

    #[test]
    fn action_launch_app_tagged() {
        let a = Action::LaunchApp {
            app_path: "/x".into(),
            app_name: "X".into(),
        };
        let json = serde_json::to_string(&a).unwrap();
        assert!(json.contains("\"type\":\"launch_app\""));
        assert!(json.contains("\"appPath\":\"/x\""));
    }

    #[test]
    fn open_url_round_trip() {
        let a = Action::OpenUrl {
            url: "https://www.youtube.com/playlist?list=PLabc".into(),
            display_name: Some("Lofi Beats".into()),
        };
        let json = serde_json::to_string(&a).unwrap();
        assert!(json.contains("\"type\":\"open_url\""));
        assert!(json.contains("\"displayName\":\"Lofi Beats\""));
        let back: Action = serde_json::from_str(&json).unwrap();
        assert_eq!(a, back);
    }

    #[test]
    fn open_url_optional_display_name_omitted() {
        let a = Action::OpenUrl {
            url: "https://example.com".into(),
            display_name: None,
        };
        let json = serde_json::to_string(&a).unwrap();
        assert!(!json.contains("displayName"));
        let back: Action = serde_json::from_str(&json).unwrap();
        assert_eq!(a, back);
    }

    #[test]
    fn run_command_round_trip() {
        let a = Action::RunCommand {
            program: "python3".into(),
            args: vec!["~/scripts/cardnews.py".into(), "--today".into()],
            working_dir: Some("~/projects/cardnews".into()),
            display_name: Some("오늘의 카드뉴스".into()),
        };
        let json = serde_json::to_string(&a).unwrap();
        assert!(json.contains("\"type\":\"run_command\""));
        assert!(json.contains("\"workingDir\""));
        assert!(json.contains("\"displayName\""));
        let back: Action = serde_json::from_str(&json).unwrap();
        assert_eq!(a, back);
    }

    #[test]
    fn run_command_optional_fields_omitted() {
        let a = Action::RunCommand {
            program: "bash".into(),
            args: vec!["~/scripts/backup.sh".into()],
            working_dir: None,
            display_name: None,
        };
        let json = serde_json::to_string(&a).unwrap();
        assert!(!json.contains("workingDir"));
        assert!(!json.contains("displayName"));
        let back: Action = serde_json::from_str(&json).unwrap();
        assert_eq!(a, back);
    }

    #[test]
    fn multi_action_round_trip() {
        let m = Action::MultiAction {
            actions: vec![
                Action::LaunchApp {
                    app_path: "/a".into(),
                    app_name: "A".into(),
                },
                Action::LaunchApp {
                    app_path: "/b".into(),
                    app_name: "B".into(),
                },
            ],
        };
        let json = serde_json::to_string(&m).unwrap();
        assert!(json.contains("\"type\":\"multi_action\""));
        assert!(json.contains("\"actions\""));
        let back: Action = serde_json::from_str(&json).unwrap();
        assert_eq!(m, back);
    }

    #[test]
    fn nested_multi_action_serializes() {
        // The schema permits nesting; the executor flattens at runtime.
        let inner = Action::MultiAction {
            actions: vec![Action::LaunchApp {
                app_path: "/a".into(),
                app_name: "A".into(),
            }],
        };
        let outer = Action::MultiAction {
            actions: vec![inner],
        };
        let json = serde_json::to_string(&outer).unwrap();
        let back: Action = serde_json::from_str(&json).unwrap();
        assert_eq!(outer, back);
    }
}
