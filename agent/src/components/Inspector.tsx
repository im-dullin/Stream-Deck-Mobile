import { useState } from "react";
import {
  collapseActions,
  expandActions,
  MAX_MULTI_ACTIONS,
  urlDisplayName,
} from "../types/protocol";
import type { Action, Button, InstalledApp } from "../types/protocol";
import { AppPicker } from "./AppPicker";
import { UrlInput } from "./UrlInput";

interface Props {
  selection: { row: number; col: number } | null;
  button: Button | undefined;
  onUpdate: (next: Button | null) => void;
}

export function Inspector({ selection, button, onUpdate }: Props) {
  const [appPickerOpen, setAppPickerOpen] = useState(false);
  const [urlInputOpen, setUrlInputOpen] = useState(false);

  if (!selection) {
    return (
      <aside className="inspector inspector--empty">
        <p className="muted">셀을 선택하면 설정 화면이 표시됩니다.</p>
      </aside>
    );
  }

  const actions: Action[] = expandActions(button?.action);
  const atLimit = actions.length >= MAX_MULTI_ACTIONS;

  const commit = (
    nextActions: Action[],
    extras?: { label?: string; iconBase64?: string },
  ) => {
    const collapsed = collapseActions(nextActions);
    if (collapsed === null) {
      onUpdate(null);
      return;
    }
    const next: Button = {
      row: selection.row,
      col: selection.col,
      label: extras?.label ?? button?.label,
      iconBase64: extras?.iconBase64 ?? button?.iconBase64,
      action: collapsed,
    };
    onUpdate(next);
  };

  const onPickApp = (app: InstalledApp) => {
    setAppPickerOpen(false);
    const sub: Action = {
      type: "launch_app",
      appPath: app.path,
      appName: app.name,
    };
    const isFirst = actions.length === 0;
    commit(
      [...actions, sub],
      isFirst ? { label: app.name, iconBase64: app.iconBase64 } : undefined,
    );
  };

  const onSubmitUrl = (url: string, displayName: string | undefined) => {
    setUrlInputOpen(false);
    const sub: Action = { type: "open_url", url, displayName };
    const isFirst = actions.length === 0;
    const labelFallback = displayName ?? urlDisplayName(url);
    commit(
      [...actions, sub],
      isFirst ? { label: labelFallback } : undefined,
    );
  };

  const onMove = (idx: number, delta: -1 | 1) => {
    const target = idx + delta;
    if (target < 0 || target >= actions.length) return;
    const next = [...actions];
    [next[idx], next[target]] = [next[target], next[idx]];
    commit(next);
  };

  const onRemove = (idx: number) => {
    const next = actions.filter((_, i) => i !== idx);
    commit(next);
  };

  const onLabelChange = (label: string) => {
    if (!button) return;
    onUpdate({ ...button, label });
  };

  const onClear = () => onUpdate(null);

  return (
    <aside className="inspector">
      <header className="inspector__header">
        <h3>
          버튼 {selection.row + 1}·{selection.col + 1}
        </h3>
        {button && (
          <button className="link danger" onClick={onClear}>
            지우기
          </button>
        )}
      </header>

      <div className="inspector__body">
        {button && (
          <label className="field">
            <span>라벨</span>
            <input
              type="text"
              value={button.label ?? ""}
              onChange={(e) => onLabelChange(e.target.value)}
            />
          </label>
        )}

        <div className="field">
          <div className="actions-meta">
            <span>
              액션{" "}
              {actions.length > 0 && `(${actions.length}/${MAX_MULTI_ACTIONS})`}
            </span>
            {actions.length >= 2 && (
              <span className="muted">순차 실행됨</span>
            )}
          </div>

          {actions.length === 0 ? (
            <p className="muted">지정된 액션이 없습니다.</p>
          ) : (
            <ul className="action-list">
              {actions.map((a, idx) => (
                <li key={idx} className="action-list__item">
                  <div className="action-list__index">{idx + 1}</div>
                  <div className="action-list__info">
                    {a.type === "launch_app" && (
                      <>
                        <div className="action-list__name">{a.appName}</div>
                        <div className="action-list__path">{a.appPath}</div>
                      </>
                    )}
                    {a.type === "open_url" && (
                      <>
                        <div className="action-list__name">
                          🔗 {a.displayName ?? urlDisplayName(a.url)}
                        </div>
                        <div className="action-list__path">{a.url}</div>
                      </>
                    )}
                    {a.type === "multi_action" && (
                      <div className="action-list__name muted">
                        중첩 복합 액션 (실행되지 않음)
                      </div>
                    )}
                  </div>
                  <div className="action-list__controls">
                    <button
                      className="action-list__btn"
                      onClick={() => onMove(idx, -1)}
                      disabled={idx === 0}
                      title="위로"
                      aria-label="위로"
                    >
                      ↑
                    </button>
                    <button
                      className="action-list__btn"
                      onClick={() => onMove(idx, 1)}
                      disabled={idx === actions.length - 1}
                      title="아래로"
                      aria-label="아래로"
                    >
                      ↓
                    </button>
                    <button
                      className="action-list__btn action-list__btn--danger"
                      onClick={() => onRemove(idx)}
                      title="제거"
                      aria-label="제거"
                    >
                      ×
                    </button>
                  </div>
                </li>
              ))}
            </ul>
          )}

          <div className="action-list__add-row">
            <button
              className="primary"
              onClick={() => setAppPickerOpen(true)}
              disabled={atLimit}
            >
              + 앱
            </button>
            <button
              className="secondary"
              onClick={() => setUrlInputOpen(true)}
              disabled={atLimit}
            >
              + URL
            </button>
            {atLimit && (
              <span className="muted">최대 {MAX_MULTI_ACTIONS}개</span>
            )}
          </div>
        </div>
      </div>

      <AppPicker
        open={appPickerOpen}
        onPick={onPickApp}
        onClose={() => setAppPickerOpen(false)}
      />
      <UrlInput
        open={urlInputOpen}
        onSubmit={onSubmitUrl}
        onClose={() => setUrlInputOpen(false)}
      />
    </aside>
  );
}
