use encore::SongControl;
use getch_rs::{Getch, Key};

pub struct Input(Getch);

impl Input {
    pub fn from_nothing_and_apply() -> Input {
        Input(Getch::new())
    }

    pub fn blocking_wait_for_input(&self) -> SongControl {
        let config = &crate::CONFIG.read().unwrap().keybinds;

        let chaw = self.0.getch().expect("can't read");
        match chaw {
            // TODO: arrow keys should be changed to respect hjkl
            Key::Up => SongControl::VolumeUp,
            Key::Down => SongControl::VolumeDown,
            Key::Left => SongControl::SeekBackward,
            Key::Right => SongControl::SeekForward,
            Key::Char(key) => {
                // TODO: more computer sciency approach lmao
                // possibly something like .to_action() in TomlKeybinds or something
                if key == config.toggle_loop {
                    SongControl::ToggleLoop
                } else if key == config.prev_song {
                    SongControl::PrevSong
                } else if key == config.next_song {
                    SongControl::NextSong
                } else if key == config.toggle_pause {
                    SongControl::TogglePause
                } else if key == config.toggle_shuffle {
                    SongControl::ToggleShuffle
                } else if key == config.exit {
                    SongControl::DestroyAndExit
                } else {
                    SongControl::No
                }
            },
            Key::Ctrl('c') => { SongControl::DestroyAndExit }
            _ => SongControl::No,
        }
    }
}

