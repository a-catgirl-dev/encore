use encore_shared::SongControl;
use getch_rs::{Getch, Key};

pub struct Input(Getch);

impl Input {
    pub fn from_nothing_and_apply() -> Input {
        Input(Getch::new())
    }

    pub fn blocking_wait_for_input(&self) -> SongControl {
        let config = &crate::CONFIG.read().unwrap().keybinds;

        let mut keybind_map: std::collections::HashMap<char, SongControl> = std::collections::HashMap::new();
        keybind_map.insert(config.toggle_loop, SongControl::ToggleLoop);
        keybind_map.insert(config.prev_song, SongControl::PrevSong);
        keybind_map.insert(config.next_song, SongControl::NextSong);
        keybind_map.insert(config.toggle_pause, SongControl::TogglePause);
        keybind_map.insert(config.toggle_shuffle, SongControl::ToggleShuffle);
        keybind_map.insert(config.exit, SongControl::DestroyAndExit);

        let chaw = self.0.getch().expect("can't read");

        match chaw {
            // TODO: arrow keys should be changed to respect hjkl
            Key::Up => SongControl::VolumeUp,
            Key::Down => SongControl::VolumeDown,
            Key::Left => SongControl::SeekBackward,
            Key::Right => SongControl::SeekForward,
            Key::Char(key) => keybind_map.get(&key).cloned().unwrap_or(SongControl::No),
            Key::Ctrl('c') => { SongControl::DestroyAndExit }
            _ => SongControl::No,
        }
    }
}

