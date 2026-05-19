import { invoke } from "@tauri-apps/api/core";
import type {
  AgentStatus,
  InstalledApp,
  Pairing,
  Profile,
} from "../types/protocol";

export const getProfile = () => invoke<Profile>("get_profile");

export const saveProfile = (profile: Profile) =>
  invoke<void>("save_profile", { profile });

export const listInstalledApps = () =>
  invoke<InstalledApp[]>("list_installed_apps");

export const getAgentStatus = () => invoke<AgentStatus>("get_agent_status");

export const approvePair = (requestId: string) =>
  invoke<void>("approve_pair", { requestId });

export const rejectPair = (requestId: string) =>
  invoke<void>("reject_pair", { requestId });

export const listPairings = () => invoke<Pairing[]>("list_pairings");

export const revokePairing = (deviceId: string) =>
  invoke<void>("revoke_pairing", { deviceId });
