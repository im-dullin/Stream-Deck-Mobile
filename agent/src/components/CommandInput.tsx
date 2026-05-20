import { useEffect, useState } from "react";

export interface ParsedCommand {
  program: string;
  args: string[];
  workingDir?: string;
  displayName?: string;
}

interface Props {
  open: boolean;
  onSubmit: (cmd: ParsedCommand) => void;
  onClose: () => void;
}

/**
 * Split a command line into program + positional args, honoring single and
 * double quotes (but not backslash escapes — keep it simple). Empty input
 * yields an empty list.
 */
function tokenize(input: string): string[] {
  const tokens: string[] = [];
  const re = /"([^"]*)"|'([^']*)'|(\S+)/g;
  let match: RegExpExecArray | null;
  while ((match = re.exec(input)) !== null) {
    tokens.push(match[1] ?? match[2] ?? match[3]);
  }
  return tokens;
}

export function CommandInput({ open, onSubmit, onClose }: Props) {
  const [command, setCommand] = useState("");
  const [workingDir, setWorkingDir] = useState("");
  const [displayName, setDisplayName] = useState("");

  useEffect(() => {
    if (!open) {
      setCommand("");
      setWorkingDir("");
      setDisplayName("");
    }
  }, [open]);

  if (!open) return null;

  const tokens = tokenize(command.trim());
  const canSubmit = tokens.length > 0;

  const submit = () => {
    if (!canSubmit) return;
    onSubmit({
      program: tokens[0],
      args: tokens.slice(1),
      workingDir: workingDir.trim() || undefined,
      displayName: displayName.trim() || undefined,
    });
  };

  return (
    <div className="modal-backdrop" onClick={onClose}>
      <div className="modal" onClick={(e) => e.stopPropagation()}>
        <header className="modal__header">
          <h2>명령 실행</h2>
          <button className="icon-btn" onClick={onClose} aria-label="닫기">
            ×
          </button>
        </header>
        <div className="modal__form">
          <label className="field">
            <span>명령</span>
            <input
              type="text"
              autoFocus
              placeholder="예: python3 ~/scripts/cardnews.py --today"
              value={command}
              onChange={(e) => setCommand(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === "Enter") submit();
              }}
            />
            <span className="field__hint muted">
              공백이 포함된 경로·인자는 큰따옴표로 감싸세요. 파이프(|)·리다이렉트(&gt;)는 셸 스크립트로 감싸야 동작합니다.
            </span>
          </label>
          <label className="field">
            <span>작업 디렉토리 (선택)</span>
            <input
              type="text"
              placeholder="예: ~/projects/cardnews"
              value={workingDir}
              onChange={(e) => setWorkingDir(e.target.value)}
            />
          </label>
          <label className="field">
            <span>표시 이름 (선택)</span>
            <input
              type="text"
              placeholder="예: 오늘의 카드뉴스"
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
