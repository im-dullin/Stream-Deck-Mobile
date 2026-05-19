import { useCallback, useEffect, useState } from "react";
import { getAgentStatus, getProfile, saveProfile } from "./api/tauri";
import { Grid } from "./components/Grid";
import { Inspector } from "./components/Inspector";
import { PairingBanner } from "./components/PairingBanner";
import { StatusBar } from "./components/StatusBar";
import type { AgentStatus, Button, Page, Profile } from "./types/protocol";
import "./App.css";

interface Selection {
  row: number;
  col: number;
}

export default function App() {
  const [profile, setProfile] = useState<Profile | null>(null);
  const [status, setStatus] = useState<AgentStatus | null>(null);
  const [selected, setSelected] = useState<Selection | null>(null);
  const [error, setError] = useState<string | null>(null);

  const refreshStatus = useCallback(() => {
    getAgentStatus().then(setStatus).catch((e) => console.error(e));
  }, []);

  useEffect(() => {
    Promise.all([getProfile(), getAgentStatus()])
      .then(([p, s]) => {
        setProfile(p);
        setStatus(s);
      })
      .catch((e) => setError(String(e)));
    const id = setInterval(refreshStatus, 5000);
    return () => clearInterval(id);
  }, [refreshStatus]);

  const activePage: Page | null = profile
    ? profile.pages.find((p) => p.id === profile.defaultPageId) ??
      profile.pages[0] ??
      null
    : null;

  const selectedButton: Button | undefined =
    selected && activePage
      ? activePage.buttons.find(
          (b) => b.row === selected.row && b.col === selected.col,
        )
      : undefined;

  const onUpdate = useCallback(
    async (next: Button | null) => {
      if (!profile || !activePage || !selected) return;
      const updatedButtons = activePage.buttons.filter(
        (b) => !(b.row === selected.row && b.col === selected.col),
      );
      if (next) updatedButtons.push(next);

      const updatedPage: Page = { ...activePage, buttons: updatedButtons };
      const updatedProfile: Profile = {
        ...profile,
        pages: profile.pages.map((p) =>
          p.id === updatedPage.id ? updatedPage : p,
        ),
      };
      setProfile(updatedProfile);
      try {
        await saveProfile(updatedProfile);
      } catch (e) {
        setError(String(e));
      }
    },
    [profile, activePage, selected],
  );

  if (error) {
    return (
      <main className="error-screen">
        <h1>에이전트를 불러올 수 없습니다</h1>
        <pre>{error}</pre>
      </main>
    );
  }

  if (!profile || !status || !activePage) {
    return (
      <main className="loading-screen">
        <p>불러오는 중…</p>
      </main>
    );
  }

  return (
    <div className="app">
      <StatusBar status={status} />
      <PairingBanner />
      <div className="workspace">
        <section className="canvas">
          <h2 className="canvas__title">{activePage.name}</h2>
          <Grid
            page={activePage}
            selected={selected}
            onSelect={(row, col) => setSelected({ row, col })}
          />
        </section>
        <Inspector
          selection={selected}
          button={selectedButton}
          onUpdate={onUpdate}
        />
      </div>
    </div>
  );
}
