import { useEffect, useMemo, useState } from "react";
import { listInstalledApps } from "../api/tauri";
import type { InstalledApp } from "../types/protocol";

interface Props {
  open: boolean;
  onPick: (app: InstalledApp) => void;
  onClose: () => void;
}

export function AppPicker({ open, onPick, onClose }: Props) {
  const [apps, setApps] = useState<InstalledApp[]>([]);
  const [loading, setLoading] = useState(false);
  const [filter, setFilter] = useState("");
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!open) return;
    setLoading(true);
    setError(null);
    listInstalledApps()
      .then((a) => setApps(a))
      .catch((e: unknown) => setError(String(e)))
      .finally(() => setLoading(false));
  }, [open]);

  const filtered = useMemo(() => {
    const q = filter.trim().toLowerCase();
    if (!q) return apps;
    return apps.filter((a) => a.name.toLowerCase().includes(q));
  }, [apps, filter]);

  if (!open) return null;

  return (
    <div className="modal-backdrop" onClick={onClose}>
      <div className="modal" onClick={(e) => e.stopPropagation()}>
        <header className="modal__header">
          <h2>응용 프로그램 선택</h2>
          <button className="icon-btn" onClick={onClose} aria-label="닫기">
            ×
          </button>
        </header>
        <input
          type="search"
          className="modal__search"
          autoFocus
          placeholder="앱 검색…"
          value={filter}
          onChange={(e) => setFilter(e.target.value)}
        />
        <div className="modal__body">
          {loading ? (
            <p className="muted">설치된 앱 스캔 중…</p>
          ) : error ? (
            <p className="error">앱 목록을 불러올 수 없습니다: {error}</p>
          ) : filtered.length === 0 ? (
            <p className="muted">일치하는 앱이 없습니다.</p>
          ) : (
            <ul className="app-list">
              {filtered.map((a) => (
                <li key={a.path}>
                  <button className="app-list__item" onClick={() => onPick(a)}>
                    {a.iconBase64 ? (
                      <img
                        className="app-list__icon"
                        src={`data:image/png;base64,${a.iconBase64}`}
                        alt=""
                      />
                    ) : (
                      <div className="app-list__icon app-list__icon--placeholder" />
                    )}
                    <div className="app-list__meta">
                      <span className="app-list__name">{a.name}</span>
                      <span className="app-list__path">{a.path}</span>
                    </div>
                  </button>
                </li>
              ))}
            </ul>
          )}
        </div>
      </div>
    </div>
  );
}
