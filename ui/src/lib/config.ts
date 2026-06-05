// TypeScript mirror of the Rust `soomfon_core::config` model.
//
// Field names match the JSON produced by serde exactly, so a `Config` can be
// round-tripped through the `get_config` / `save_config` Tauri commands without
// any transformation. `Page.buttons` is keyed by the stringified key index,
// because serde serialises the Rust `BTreeMap<u8, Button>` with string keys.

export type Rgb = [number, number, number];

/**
 * What a key does when pressed. Tagged by `type` to match serde's internally
 * tagged enum (`#[serde(tag = "type")]`).
 */
export type Action =
  | { type: "none" }
  | { type: "run_command"; program: string; args: string[] };

export interface Button {
  /** Text drawn on the key. Omitted (undefined) leaves a solid fill. */
  label?: string;
  /** Background colour. */
  color: Rgb;
  /** Label colour. */
  text_color: Rgb;
  /** Behaviour on press. */
  action: Action;
}

export interface Page {
  name: string;
  /** Configured keys, by key index as a string. Missing keys are blank. */
  buttons: Record<string, Button>;
}

export interface Profile {
  name: string;
  pages: Page[];
}

export interface Config {
  version: number;
  active_profile: number;
  profiles: Profile[];
}

/**
 * Drawable keys on the Soomfon Stream Controller SE: the top two rows of the
 * 3x3 grid. (The bottom row is plain push buttons with no screen.) Once the
 * backend exposes per-device screen counts this will come from the device.
 */
export const LCD_KEYS = 6;
/** The panel is laid out three keys wide. */
export const GRID_COLS = 3;

/** Sensible defaults for a freshly-configured key, matching the Rust side. */
export function defaultButton(): Button {
  return {
    color: [0x14, 0x14, 0x14],
    text_color: [0xff, 0xff, 0xff],
    action: { type: "none" },
  };
}

/** `[r, g, b]` -> `"#rrggbb"` for `<input type="color">`. */
export function rgbToHex([r, g, b]: Rgb): string {
  return "#" + [r, g, b].map((v) => v.toString(16).padStart(2, "0")).join("");
}

/** `"#rrggbb"` -> `[r, g, b]`. */
export function hexToRgb(hex: string): Rgb {
  const n = parseInt(hex.slice(1), 16);
  return [(n >> 16) & 0xff, (n >> 8) & 0xff, n & 0xff];
}

/** The active profile index, clamped to a valid range. */
export function activeProfileIndex(config: Config): number {
  const last = config.profiles.length - 1;
  return Math.min(Math.max(config.active_profile, 0), last);
}

/** The page currently being edited: the active profile's first page. */
export function activePage(config: Config): Page {
  return config.profiles[activeProfileIndex(config)].pages[0];
}
