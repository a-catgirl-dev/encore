#![cfg_attr(not(feature = "configuration"), allow(unused))]

#[cfg(feature = "configuration")]
use serde::Deserialize;

// TODO: better toml name?
#[cfg(target_os = "linux")]
static DEFAULT_CFG_PATH: &str = ".config/encore/encore.toml";
#[cfg(target_os = "macos")]
static DEFAULT_CFG_PATH: &str = "Library/Preferences/encore/encore.toml";

#[derive(Debug, Default)]
#[cfg_attr(feature = "configuration", derive(Deserialize))]
pub struct Config {
    pub main: TomlMain,
    pub playlist: TomlPlaylist,
}

#[derive(Debug)]
#[cfg_attr(feature = "configuration", derive(Deserialize), serde(default))]
pub struct TomlMain {
    /// i32 value; to get a value that can be compared with Rodio's volume() method, do:
    /// `max_vol as f32 / 100.0`
    pub max_vol: i32,
    /// i32 value; to get a value that can be compared with Rodio's volume() method, do:
    /// `max_vol as f32 / 100.0`
    pub default_vol: i32,
}
impl Default for TomlMain {
    fn default() -> Self {
        Self {
            max_vol: 200,
            default_vol: 100,
        }
    }
}

#[derive(Debug)]
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
        let file = format!("{}/{}", std::env::home_dir().unwrap().to_string_lossy().to_string(), file);

        let buf = read_to_string(file).unwrap();

        let parsed: Config = basic_toml::from_str(&buf).unwrap();
        dbg!(&parsed);

        parsed
    }
    }
}

