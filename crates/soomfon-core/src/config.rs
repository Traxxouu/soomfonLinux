//! User configuration: profiles, pages, buttons, and where they live on disk.
//!
//! A [`Config`] is the whole persisted document. It holds one or more
//! [`Profile`]s; each profile is an ordered list of [`Page`]s; each page maps a
//! key index to a [`Button`]. Only configured keys are stored — an absent entry
//! means that key is blank.
//!
//! Persistence is JSON via [`ConfigStore`], which lives under the platform
//! config directory (on Linux, `~/.config/soomfonlinux/config.json`). The model
//! is deliberately decoupled from the hardware crate: a button describes what
//! to show (a label and colours) and what to do when pressed (an [`Action`]);
//! turning that into pixels and dispatching the action is the job of the
//! device session, not this module.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::action::Action;

/// Schema version of the persisted config document. Bumped when the on-disk
/// shape changes so older files can be recognised.
pub const CURRENT_VERSION: u32 = 1;

/// Default key background: a dark grey that text reads well on.
const DEFAULT_BG: [u8; 3] = [0x14, 0x14, 0x14];
/// Default label colour.
const DEFAULT_FG: [u8; 3] = [0xFF, 0xFF, 0xFF];

/// One configurable key: what it shows on its LCD and what it does when pressed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Button {
    /// Text drawn on the key. `None` (or empty) leaves the key a solid fill.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    /// Background colour, RGB.
    pub color: [u8; 3],
    /// Label colour, RGB.
    pub text_color: [u8; 3],
    /// What pressing the key does. Defaults to [`Action::None`], and older
    /// config files without this field load as such.
    #[serde(default)]
    pub action: Action,
}

impl Default for Button {
    fn default() -> Self {
        Self {
            label: None,
            color: DEFAULT_BG,
            text_color: DEFAULT_FG,
            action: Action::None,
        }
    }
}

impl Button {
    /// A labelled button using the default colours.
    pub fn labelled(label: impl Into<String>) -> Self {
        Self {
            label: Some(label.into()),
            ..Self::default()
        }
    }
}

/// One screenful of keys, addressed by key index (row-major).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Page {
    /// Display name, shown in the editor.
    pub name: String,
    /// Configured keys. Missing indices are blank.
    #[serde(default)]
    pub buttons: BTreeMap<u8, Button>,
}

impl Page {
    /// An empty page with the given name.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            buttons: BTreeMap::new(),
        }
    }

    /// The button at `key`, if one is configured.
    pub fn button(&self, key: u8) -> Option<&Button> {
        self.buttons.get(&key)
    }

    /// Assign `button` to `key`, replacing any existing one.
    pub fn set_button(&mut self, key: u8, button: Button) {
        self.buttons.insert(key, button);
    }

    /// Blank `key`. Returns the button that was there, if any.
    pub fn clear_button(&mut self, key: u8) -> Option<Button> {
        self.buttons.remove(&key)
    }
}

/// A named set of pages the user switches between.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Profile {
    /// Display name, shown in the editor.
    pub name: String,
    /// Ordered pages; always at least one.
    pub pages: Vec<Page>,
}

impl Profile {
    /// A profile with the given name and a single empty page.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            pages: vec![Page::new("Page 1")],
        }
    }

    /// Append a new empty page and return its index.
    pub fn add_page(&mut self, name: impl Into<String>) -> usize {
        self.pages.push(Page::new(name));
        self.pages.len() - 1
    }
}

/// The whole persisted document: every profile the user has defined.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {
    /// Schema version of this document; see [`CURRENT_VERSION`].
    pub version: u32,
    /// Index into `profiles` of the profile currently in use.
    #[serde(default)]
    pub active_profile: usize,
    /// All profiles; always at least one.
    pub profiles: Vec<Profile>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            version: CURRENT_VERSION,
            active_profile: 0,
            profiles: vec![Profile::new("Default")],
        }
    }
}

impl Config {
    /// The active profile, clamped so it is always valid even if
    /// `active_profile` points past the end of a hand-edited file.
    pub fn active_profile(&self) -> &Profile {
        let idx = self
            .active_profile
            .min(self.profiles.len().saturating_sub(1));
        &self.profiles[idx]
    }

    /// Append a profile and return its index.
    pub fn add_profile(&mut self, name: impl Into<String>) -> usize {
        self.profiles.push(Profile::new(name));
        self.profiles.len() - 1
    }

    /// The page the device currently shows: the first page of the active
    /// profile. Returns `None` only for a hand-edited profile with no pages.
    /// (Page navigation is a later feature; for now the first page is active.)
    pub fn active_page(&self) -> Option<&Page> {
        self.active_profile().pages.first()
    }
}

/// Failures from reading or writing the config file.
#[derive(Debug, Error)]
pub enum ConfigError {
    /// The platform has no usable config directory.
    #[error("could not locate a config directory for this platform")]
    NoConfigDir,
    /// Reading or writing the file failed.
    #[error("config i/o error at {path}: {source}")]
    Io {
        /// The file we were touching.
        path: PathBuf,
        /// The underlying OS error.
        source: std::io::Error,
    },
    /// The file's JSON could not be parsed or produced.
    #[error("config serialization error: {0}")]
    Serde(#[from] serde_json::Error),
}

/// Reads and writes a [`Config`] at a fixed path.
pub struct ConfigStore {
    path: PathBuf,
}

impl ConfigStore {
    /// A store backed by an explicit file path (used by tests and callers that
    /// want a non-default location).
    pub fn with_path(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    /// A store at the platform's standard config location
    /// (`~/.config/soomfonlinux/config.json` on Linux).
    pub fn discover() -> Result<Self, ConfigError> {
        let dirs = directories::ProjectDirs::from("", "", "soomfonLinux")
            .ok_or(ConfigError::NoConfigDir)?;
        Ok(Self::with_path(dirs.config_dir().join("config.json")))
    }

    /// The file this store reads and writes.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Load the config, returning [`Config::default`] if the file does not yet
    /// exist (first run).
    pub fn load(&self) -> Result<Config, ConfigError> {
        let bytes = match fs::read(&self.path) {
            Ok(bytes) => bytes,
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(Config::default()),
            Err(source) => {
                return Err(ConfigError::Io {
                    path: self.path.clone(),
                    source,
                })
            }
        };
        Ok(serde_json::from_slice(&bytes)?)
    }

    /// Save the config as pretty JSON. Writes to a sibling temp file and renames
    /// it into place so a crash mid-write can't truncate the real file.
    pub fn save(&self, config: &Config) -> Result<(), ConfigError> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent).map_err(|source| ConfigError::Io {
                path: parent.to_path_buf(),
                source,
            })?;
        }

        let json = serde_json::to_vec_pretty(config)?;
        let tmp = self.path.with_extension("json.tmp");
        fs::write(&tmp, &json).map_err(|source| ConfigError::Io {
            path: tmp.clone(),
            source,
        })?;
        fs::rename(&tmp, &self.path).map_err(|source| ConfigError::Io {
            path: self.path.clone(),
            source,
        })
    }
}

/// Load the config from the platform's standard location.
pub fn load_config() -> Result<Config, ConfigError> {
    ConfigStore::discover()?.load()
}

/// Save the config to the platform's standard location.
pub fn save_config(config: &Config) -> Result<(), ConfigError> {
    ConfigStore::discover()?.save(config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_has_one_profile_with_one_page() {
        let cfg = Config::default();
        assert_eq!(cfg.version, CURRENT_VERSION);
        assert_eq!(cfg.profiles.len(), 1);
        assert_eq!(cfg.active_profile().name, "Default");
        assert_eq!(cfg.active_profile().pages.len(), 1);
    }

    #[test]
    fn active_profile_is_clamped_when_index_is_out_of_range() {
        let cfg = Config {
            active_profile: 99,
            ..Config::default()
        };
        // Does not panic; falls back to the last valid profile.
        assert_eq!(cfg.active_profile().name, "Default");
    }

    #[test]
    fn page_button_crud_round_trips() {
        let mut page = Page::new("p");
        assert!(page.button(2).is_none());

        page.set_button(2, Button::labelled("OBS"));
        assert_eq!(page.button(2).unwrap().label.as_deref(), Some("OBS"));

        let removed = page.clear_button(2);
        assert_eq!(removed.unwrap().label.as_deref(), Some("OBS"));
        assert!(page.button(2).is_none());
    }

    #[test]
    fn config_survives_a_json_round_trip() {
        let mut cfg = Config::default();
        cfg.add_profile("Streaming");
        cfg.profiles[1].add_page("Scenes");
        cfg.profiles[1].pages[0].set_button(0, Button::labelled("Live"));
        cfg.active_profile = 1;

        let json = serde_json::to_string(&cfg).unwrap();
        let back: Config = serde_json::from_str(&json).unwrap();
        assert_eq!(cfg, back);
    }

    #[test]
    fn store_returns_default_when_file_is_absent() {
        let dir = tempfile::tempdir().unwrap();
        let store = ConfigStore::with_path(dir.path().join("config.json"));
        assert_eq!(store.load().unwrap(), Config::default());
    }

    #[test]
    fn store_save_then_load_round_trips_and_creates_dirs() {
        let dir = tempfile::tempdir().unwrap();
        // A nested path that does not exist yet, to exercise create_dir_all.
        let store = ConfigStore::with_path(dir.path().join("nested/config.json"));

        let mut cfg = Config::default();
        cfg.profiles[0].pages[0].set_button(5, Button::labelled("X"));

        store.save(&cfg).unwrap();
        assert!(store.path().exists());
        assert_eq!(store.load().unwrap(), cfg);
    }

    #[test]
    fn missing_optional_fields_deserialize_to_defaults() {
        // A hand-written minimal document: no active_profile, page without
        // buttons, button without a label.
        let json = r#"{
            "version": 1,
            "profiles": [
                { "name": "Default", "pages": [
                    { "name": "Page 1", "buttons": {
                        "0": { "color": [1,2,3], "text_color": [4,5,6] }
                    } }
                ] }
            ]
        }"#;
        let cfg: Config = serde_json::from_str(json).unwrap();
        assert_eq!(cfg.active_profile, 0);
        let button = cfg.profiles[0].pages[0].button(0).unwrap();
        assert_eq!(button.label, None);
        assert_eq!(button.color, [1, 2, 3]);
        // A button predating actions loads with no action.
        assert_eq!(button.action, Action::None);
    }

    #[test]
    fn active_page_is_the_active_profiles_first_page() {
        let mut cfg = Config::default();
        cfg.add_profile("Streaming");
        cfg.profiles[1].pages[0].set_button(0, Button::labelled("Live"));
        cfg.active_profile = 1;

        let page = cfg.active_page().expect("a page exists");
        assert_eq!(page.button(0).unwrap().label.as_deref(), Some("Live"));
    }
}
