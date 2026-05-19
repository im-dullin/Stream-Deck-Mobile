import { useEffect, useState } from "react";

interface Props {
  open: boolean;
  onSubmit: (url: string, displayName: string | undefined) => void;
  onClose: () => void;
}

export function UrlInput({ open, onSubmit, onClose }: Props) {
  const [url, setUrl] = useState("");
  const [displayName, setDisplayName] = useState("");

  useEffect(() => {
    if (!open) {
      setUrl("");
      setDisplayName("");
    }
  }, [open]);

  if (!open) return null;

  const trimmed = url.trim();
  const canSubmit = trimmed.length > 0;

  const submit = () => {
    if (!canSubmit) return;
    onSubmit(trimmed, displayName.trim() || undefined);
  };

  return (
    <div className="modal-backdrop" onClick={onClose}>
      <div className="modal" onClick={(e) => e.stopPropagation()}>
        <header className="modal__header">
          <h2>URL 열기</h2>
          <button className="icon-btn" onClick={onClose} aria-label="닫기">
            ×
          </button>
        </header>
        <div className="modal__form">
          <label className="field">
            <span>URL</span>
            <input
              type="url"
              autoFocus
              placeholder="https://www.youtube.com/playlist?list=…"
              value={url}
              onChange={(e) => setUrl(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === "Enter") submit();
              }}
            />
          </label>
          <label className="field">
            <span>표시 이름 (선택)</span>
            <input
              type="text"
              placeholder="예: 로파이 플레이리스트"
              value={displayName}
              onChange={(e) => setDisplayName(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === "Enter") submit();
              }}
            />
          </label>
          <div className="modal__actions">
            <button className="secondary" onClick={onClose}>
              취소
            </button>
            <button
              className="primary"
              onClick={submit}
              disabled={!canSubmit}
            >
              추가
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}
