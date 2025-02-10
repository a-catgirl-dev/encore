use encore::SongControl;
use getch_rs::{Getch, Key};

pub struct Input(Getch);

impl Input {
    pub fn from_nothing_and_apply() -> Input {
        Input(Getch::new())
    }

    pub fn blocking_wait_for_input(&self) -> SongControl {
        let chaw = self.0.getch().expect("can't read");
        match chaw {
            // TODO: arrow keys should be changed to respect hjkl
            Key::Up => SongControl::VolumeUp,
            Key::Down => SongControl::VolumeDown,
            Key::Left => SongControl::SeekBackward,
            Key::Right => SongControl::SeekForward,
            Key::Char('r') => SongControl::ToggleLoop,
            Key::Char('k') => SongControl::PrevSong,
            Key::Char('j') => SongControl::NextSong,
            Key::Char(' ') => SongControl::TogglePause,
            Key::Char('s') => SongControl::ToggleShuffle,
            Key::Ctrl('c') | Key::Char('q') => SongControl::DestroyAndExit,
            _ => SongControl::No,
        }
    }
}

