import { useEffect, useState } from "react";
import { open as openDialog } from "@tauri-apps/plugin-dialog";

interface Props {
  open: boolean;
  onSubmit: (path: string, displayName: string | undefined) => void;
  onClose: () => void;
}

export function FolderInput({ open, onSubmit, onClose }: Props) {
  const [path, setPath] = useState("");
  const [displayName, setDisplayName] = useState("");

  useEffect(() => {
    if (!open) {
      setPath("");
      setDisplayName("");
    }
  }, [open]);

  if (!open) return null;

  const trimmed = path.trim();
  const canSubmit = trimmed.length > 0;

  const submit = () => {
    if (!canSubmit) return;
    onSubmit(trimmed, displayName.trim() || undefined);
  };

  const browse = async () => {
    try {
      const selected = await openDialog({
        directory: true,
        multiple: false,
        title: "폴더 선택",
      });
      if (typeof selected === "string") {
        setPath(selected);
      }
    } catch (e) {
      console.error("dialog open failed", e);
    }
  };

  return (
    <div className="modal-backdrop" onClick={onClose}>
      <div className="modal" onClick={(e) => e.stopPropagation()}>
        <header className="modal__header">
          <h2>폴더 열기</h2>
          <button className="icon-btn" onClick={onClose} aria-label="닫기">
            ×
          </button>
        </header>
        <div className="modal__form">
          <label className="field">
            <span>폴더 경로</span>
            <div className="field__row">
              <input
                type="text"
                autoFocus
                placeholder="예: ~/Documents/회의록"
                value={path}
                onChange={(e) => setPath(e.target.value)}
                onKeyDown={(e) => {
                  if (e.key === "Enter") submit();
                }}
              />
              <button
                type="button"
                className="secondary"
                onClick={browse}
              >
                찾아보기…
              </button>
            </div>
            <span className="field__hint muted">
              버튼을 탭하면 Finder(macOS) / 파일 탐색기(Windows) 가 이 경로에서 열립니다.
            </span>
          </label>
          <label className="field">
            <span>표시 이름 (선택)</span>
            <input
              type="text"
              placeholder="예: 이번 주 회의록"
              value={displayName}
              onChange={(e) => setDisplayName(e.target.value)}
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
