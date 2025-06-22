use std::sync::{
    RwLock,
    atomic::{
        AtomicUsize,
        AtomicU64,
        AtomicU8,
        AtomicBool
    }
};

use encore_shared::{AtomicF32, LoopMode};

lazy_static::lazy_static!{
    pub static ref PLAYLIST: RwLock<Vec<String>> = Default::default();
    pub static ref SHUFFLE_ORIGINAL_PLAYLIST: RwLock<Option<Vec<String>>> = RwLock::new(None);
    pub static ref SONG_INDEX: AtomicUsize = AtomicUsize::new(0);
    pub static ref SONG_TOTAL_LEN: AtomicU64 = AtomicU64::new(0);
    pub static ref SONG_CURRENT_LEN: AtomicU64 = AtomicU64::new(0);
    pub static ref LOOP_MODE: AtomicU8 = AtomicU8::new(LoopMode::NoLoop as u8);
    pub static ref PAUSED: AtomicBool = AtomicBool::new(false);
    pub static ref VOLUME_LEVEL: AtomicF32 = AtomicF32::new(0.0);

    pub static ref CONFIG: RwLock<encore_configuration::Config> = Default::default();
}
