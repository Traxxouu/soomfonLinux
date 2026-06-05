// Typed wrappers around the Tauri commands exposed by the Rust backend.

import { invoke } from "@tauri-apps/api/core";
import type { Config } from "./config";

export type DeviceSummary = { model: string; keys: number };
export type Status = { version: string; devices: DeviceSummary[] };

/** Live application snapshot: core version and connected decks. */
export function getStatus(): Promise<Status> {
  return invoke<Status>("get_status");
}

/** Load the persisted user configuration. */
export function getConfig(): Promise<Config> {
  return invoke<Config>("get_config");
}

/** Persist the user configuration. */
export function saveConfig(config: Config): Promise<void> {
  return invoke("save_config", { config });
}
