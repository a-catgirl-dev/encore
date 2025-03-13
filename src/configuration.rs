#![cfg_attr(not(feature = "configuration"), allow(unused))]

#[cfg(feature = "configuration")]
use serde::Deserialize;

// TODO: better toml name?
#[cfg(target_os = "linux")]
static DEFAULT_CFG_PATH: &str = ".config/encore/encore.toml";
#[cfg(target_os = "macos")]
static DEFAULT_CFG_PATH: &str = "Library/Preferences/encore/encore.toml";

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "configuration", derive(Deserialize), serde(default))]
pub struct Config {
    pub main: TomlMain,
    pub playlist: TomlPlaylist,
    pub keybinds: TomlKeybinds,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "configuration", derive(Deserialize), serde(default))]
pub struct TomlMain {
    /// i32 value; to get a value that can be compared with Rodio's volume() method, do:
    /// `max_vol as f32 / 100.0`
    pub max_vol: i32,
    /// i32 value; to get a value that can be compared with Rodio's volume() method, do:
    /// `min_vol as f32 / 100.0`
    pub min_vol: i32,
    /// i32 value; to get a value that can be compared with Rodio's volume() method, do:
    /// `default_vol as f32 / 100.0`
    pub default_vol: i32,
}
impl Default for TomlMain {
    fn default() -> Self {
        Self {
            max_vol: 200,
            min_vol: 0,
            default_vol: 100,
        }
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "configuration", derive(Deserialize), serde(default))]
pub struct TomlPlaylist {
    pub never_use: bool,
    pub highlighted_color: String,
}
impl Default for TomlPlaylist {
    fn default() -> Self {
        Self {
            never_use: false,
            highlighted_color: "f5c2e7".to_string(),
        }
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "configuration", derive(Deserialize), serde(default))]
pub struct TomlKeybinds {
    pub toggle_loop: char,
    pub prev_song: char,
    pub next_song: char,
    pub toggle_pause: char,
    pub toggle_shuffle: char,
    pub exit: char,
}
impl Default for TomlKeybinds {
    fn default() -> Self {
        Self {
            toggle_loop: 'r',
            prev_song: 'k',
            next_song: 'j',
            toggle_pause: ' ',
            toggle_shuffle: 's',
            exit: 'q',
        }
    }
}

impl Config {
    pub fn parse(to_parse: &encore::ConfigurationPath) -> Self {
    #[cfg(not(feature = "configuration"))] {
        Config::default()
    }

    #[cfg(feature = "configuration")] {
        use std::fs::read_to_string;

        let file = match to_parse {
            encore::ConfigurationPath::Default => DEFAULT_CFG_PATH,
            encore::ConfigurationPath::Custom(s) => s
        };
        #[allow(deprecated)]
        let file = format!("{}/{}", std::env::home_dir().unwrap().to_string_lossy(), file);

        let buf = read_to_string(file).unwrap();

        let parsed: Config = basic_toml::from_str(&buf).unwrap();
        dbg!(&parsed);

        parsed
    }
    }
}

