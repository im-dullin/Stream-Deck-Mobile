import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { approvePair, rejectPair } from "../api/tauri";
import type { PairRequestedEvent } from "../types/protocol";

interface PendingPair extends PairRequestedEvent {
  receivedAt: number;
}

export function PairingBanner() {
  const [pending, setPending] = useState<PendingPair[]>([]);

  useEffect(() => {
    let unlistenFn: (() => void) | undefined;
    listen<PairRequestedEvent>("pair_requested", (event) => {
      const payload = event.payload;
      setPending((prev) => [
        ...prev.filter((p) => p.requestId !== payload.requestId),
        { ...payload, receivedAt: Date.now() },
      ]);
    }).then((unlisten) => {
      unlistenFn = unlisten;
    });
    return () => {
      unlistenFn?.();
    };
  }, []);

  const onApprove = async (requestId: string) => {
    try {
      await approvePair(requestId);
    } catch (e) {
      console.error("approve_pair failed", e);
    } finally {
      setPending((prev) => prev.filter((p) => p.requestId !== requestId));
    }
  };

  const onReject = async (requestId: string) => {
    try {
      await rejectPair(requestId);
    } catch (e) {
      console.error("reject_pair failed", e);
    } finally {
      setPending((prev) => prev.filter((p) => p.requestId !== requestId));
    }
  };

  if (pending.length === 0) return null;

  return (
    <div className="pair-banners">
      {pending.map((p) => (
        <div key={p.requestId} className="pair-banner">
          <div className="pair-banner__icon">📱</div>
          <div className="pair-banner__text">
            <div className="pair-banner__title">
              <strong>{p.deviceName}</strong>이(가) 페어링을 요청했습니다
            </div>
            <div className="pair-banner__sub muted">{p.peer}</div>
          </div>
          <div className="pair-banner__actions">
            <button
              className="secondary"
              onClick={() => onReject(p.requestId)}
            >
              거절
            </button>
            <button
              className="primary"
              onClick={() => onApprove(p.requestId)}
            >
              승인
            </button>
          </div>
        </div>
      ))}
    </div>
  );
}
