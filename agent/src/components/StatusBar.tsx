import type { AgentStatus } from "../types/protocol";

interface Props {
  status: AgentStatus;
}

export function StatusBar({ status }: Props) {
  return (
    <header className="status-bar">
      <div className="status-bar__title">
        <span className="dot" />
        <strong>{status.agentName}</strong>
        <span className="muted">가상 스트림덱</span>
      </div>
      <div className="status-bar__info">
        <span className="muted">
          {status.lanIp ?? "—"}:{status.boundPort}
        </span>
        <span className="status-bar__paired">
          📱 페어 디바이스 {status.pairedCount}
        </span>
      </div>
    </header>
  );
}
