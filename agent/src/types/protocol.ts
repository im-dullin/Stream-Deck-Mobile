// TypeScript mirror of agent/src-tauri/src/protocol.rs.
// Kept in sync manually; see schema/protocol.ts for the single source of truth.

export const PROTOCOL_VERSION = 1;

export interface Profile {
  id: string;
  name: string;
  defaultPageId: string;
  pages: Page[];
}

export interface Page {
  id: string;
  name: string;
  rows: number;
  cols: number;
  buttons: Button[];
}

export interface Button {
  row: number;
  col: number;
  label?: string;
  iconBase64?: string;
  action: Action;
}

export type Action =
  | LaunchAppAction
  | OpenUrlAction
  | OpenFolderAction
  | MultiAction;

export interface LaunchAppAction {
  type: "launch_app";
  appPath: string;
  appName: string;
}

export interface OpenUrlAction {
  type: "open_url";
  url: string;
  displayName?: string;
}

export interface OpenFolderAction {
  type: "open_folder";
  path: string;
  displayName?: string;
}

/** Up to 10 sub-actions, executed sequentially. */
export interface MultiAction {
  type: "multi_action";
  actions: Action[];
}

export const MAX_MULTI_ACTIONS = 10;

/** Best-effort short label for a URL — strips `www.` from the hostname. */
export function urlDisplayName(url: string): string {
  try {
    return new URL(url).hostname.replace(/^www\./, "");
  } catch {
    return url;
  }
}

/** Last path segment as a folder's friendly name. */
export function folderDisplayName(path: string): string {
  const cleaned = path.replace(/[\\/]+$/, "");
  const parts = cleaned.split(/[\\/]/);
  return parts[parts.length - 1] || cleaned;
}

/** Normalize a button's action into a flat list (single → [single]; multi → list). */
export function expandActions(action: Action | undefined): Action[] {
  if (!action) return [];
  if (action.type === "multi_action") return action.actions;
  return [action];
}

/** Collapse a flat list back to a single action or multi-action wrapper. */
export function collapseActions(actions: Action[]): Action | null {
  if (actions.length === 0) return null;
  if (actions.length === 1) return actions[0];
  return { type: "multi_action", actions };
}

export interface InstalledApp {
  name: string;
  path: string;
  iconBase64?: string;
}

export interface AgentStatus {
  agentName: string;
  boundPort: number;
  lanIp: string | null;
  pairedCount: number;
}

export interface Pairing {
  deviceId: string;
  deviceName: string;
  token: string;
  pairedAtUnix: number;
}

export interface PairRequestedEvent {
  requestId: string;
  deviceId: string;
  deviceName: string;
  peer: string;
}
