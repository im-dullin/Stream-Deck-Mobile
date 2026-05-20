// Wire protocol — single source of truth for messages exchanged between
// the PC agent (Rust/Tauri) and the mobile deck (Flutter).
//
// Matching types must be kept in sync in:
//   - agent/src-tauri/src/protocol.rs (serde-tagged enums)
//   - mobile/lib/protocol/messages.dart (sealed classes)
//
// Wire format: JSON over WebSocket. The `type` field is the discriminator.

export const PROTOCOL_VERSION = 1;

// =====================================================================
// Domain
// =====================================================================

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
  /** Optional inline PNG, base64-encoded. Capped at ~32KB for wire efficiency. */
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
  /** Absolute path to the executable or .app bundle. */
  appPath: string;
  /** Display name; used when rendering and for telemetry. */
  appName: string;
}

/**
 * Opens a URL in the OS default handler (browser for http/https; OS routes
 * for mailto:, custom protocols, etc.). YouTube playlist URLs autoplay in the
 * browser.
 */
export interface OpenUrlAction {
  type: "open_url";
  url: string;
  /** Editor-only friendly name. Falls back to the URL's hostname. */
  displayName?: string;
}

/**
 * Reveal a directory in the host's file manager (Finder on macOS, Explorer
 * on Windows). `~/` is expanded to the user's home directory on the agent
 * side, so portable paths like `~/Documents/회의록` work.
 */
export interface OpenFolderAction {
  type: "open_folder";
  path: string;
  /** Editor-only friendly name. Falls back to the last path segment. */
  displayName?: string;
}

/**
 * Runs up to N sub-actions sequentially. Sub-actions are expected to be
 * single (non-`MultiAction`); nested multi-actions are flattened/ignored
 * at execution time. The editor UI enforces a max of 10 sub-actions.
 */
export interface MultiAction {
  type: "multi_action";
  actions: Action[];
}

// =====================================================================
// Client (mobile) → Server (PC agent)
// =====================================================================

export type ClientMessage =
  | HelloMessage
  | PairRequestMessage
  | ButtonPressMessage
  | PageChangeMessage
  | PongMessage;

/** Returning client with an existing pairing token. */
export interface HelloMessage {
  type: "hello";
  protocolVersion: number;
  deviceId: string;
  deviceName: string;
  token: string;
}

/** New client asking the agent's user to approve a fresh pairing. */
export interface PairRequestMessage {
  type: "pair_request";
  protocolVersion: number;
  deviceId: string;
  deviceName: string;
}

export interface ButtonPressMessage {
  type: "button_press";
  pageId: string;
  row: number;
  col: number;
}

export interface PageChangeMessage {
  type: "page_change";
  pageId: string;
}

export interface PongMessage {
  type: "pong";
}

// =====================================================================
// Server (PC agent) → Client (mobile)
// =====================================================================

export type ServerMessage =
  | WelcomeMessage
  | ProfileUpdateMessage
  | PairPendingMessage
  | PairAcceptedMessage
  | PairRejectedMessage
  | PingMessage
  | ErrorMessage;

export interface WelcomeMessage {
  type: "welcome";
  protocolVersion: number;
  agentName: string;
  profile: Profile;
}

export interface ProfileUpdateMessage {
  type: "profile_update";
  profile: Profile;
}

/** Pairing request acknowledged; agent is awaiting user approval. */
export interface PairPendingMessage {
  type: "pair_pending";
  requestId: string;
}

/** Pairing approved by the agent's user; `token` is the new auth secret. */
export interface PairAcceptedMessage {
  type: "pair_accepted";
  token: string;
}

/** Pairing was rejected or timed out. */
export interface PairRejectedMessage {
  type: "pair_rejected";
  reason: string;
}

export interface PingMessage {
  type: "ping";
}

export interface ErrorMessage {
  type: "error";
  /** Stable, machine-readable code. e.g. "auth_failed", "version_mismatch", "not_paired". */
  code: string;
  /** Human-readable detail; safe to display. */
  message: string;
}
