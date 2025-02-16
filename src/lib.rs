#![forbid(unsafe_code)]

use std::sync::atomic::{AtomicUsize, Ordering};

pub trait IntegerExtensions {
    fn to_rodio(self) -> f32;
}

impl IntegerExtensions for i32 {
    fn to_rodio(self) -> f32 {
        self as f32 / 100.0
    }
}

#[derive(Debug, Copy, PartialEq)]
/// don't Box<SongControl> this value, or you're going to have a very hard time with .clone()
/// because it will panic.
/// :troll:
pub enum SongControl {
    VolumeUp,
    VolumeDown,
    SeekForward,
    SeekBackward,

    ToggleLoop,
    PrevSong,
    NextSong,
    TogglePause,

    Pause,
    Resume,

    ToggleShuffle,
    ShuffleOn,
    ShuffleOff,

    No,

    DestroyAndExit,

    Unset,
}

impl Clone for SongControl {
    fn clone(&self) -> Self {
        panic!("why are we on the heap???");
    }
}

#[derive(PartialEq, Default)]
pub enum RenderMode {
    Safe, // if term is too small, or if under resource constraints, or user specified, or
    #[default]
    Full, // the entire TUI
    NoSpace,
}

#[derive(PartialEq)]
pub enum FileFormat {
    Audio,

    // and if no match
    Other
}

pub enum ConfigurationPath<'a> {
    Default,
    Custom(&'a str)
}

pub struct AtomicF32(AtomicUsize);

/// no hardware support bruh
impl AtomicF32 {
    #[inline] pub fn new(v: f32) -> Self {
        AtomicF32(AtomicUsize::new(v.to_bits().try_into().unwrap()))
    }

    #[inline] pub fn load(&self, order: Ordering) -> f32 {
        f32::from_bits(self.0.load(order).try_into().unwrap())
    }

    #[inline] pub fn store(&self, val: f32, order: Ordering) {
        self.0.store(val.to_bits().try_into().unwrap(), order);
    }
}

pub fn to_vec<R: std::io::BufRead>(reader: R) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut v = Vec::new();

    for line in reader.lines() {
        let line = line?;
        let line = if let Some(l) = normalize_line(&line) { l } else { continue };
        v.push(line); // fast code
    }

    Ok(v)
}

pub fn normalize<R: Iterator<Item = String>>(i: R) -> Vec<String> {
    let mut vec = Vec::new();
    for s in i {
        if let Some(s) = normalize_line(&s) { vec.push(s) } else { continue }
    }

    vec
}

pub fn normalize_line(s: &str) -> Option<String> {
    use std::env;

    let home = if cfg!(unix) { env::var("HOME") } else { env::var("USERPROFILE") }
        .expect("can't find home dir");

    if s.is_empty() { return None };
    Some(s.replacen('~', &home, 1))
}

pub fn trim_path(s: &str) -> &str {
    s.split('/').last().unwrap_or("")
}

pub fn shuffle_playlist(input: &mut [String]) {
    fn bogo_sort<T>(slice: &mut [T])
    where 
        T: Ord
    {
        /// please do not use this for cryptography
        /// (this is not the actual rdrand x86 instruction)
        fn rdrand() -> u64 {
            use std::time::{SystemTime, UNIX_EPOCH};

            let nanos = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .subsec_nanos();

            let seconds = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();

            seconds ^ nanos as u64
        }

        let mut i = slice.len();
        while i > 1 {
            i -= 1;
            let r = rdrand();
            let j = (r % (i as u64 + 1)) as usize;
            slice.swap(i, j);
        }
    }

    bogo_sort(input);
}

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn bogo_sort_without_verification() {
        // rust, please
        let vec: Vec<String> = ["a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l", "m", "n"]
            .into_iter().map(String::from).collect();
        dbg!(&vec);

        // shuffle_playlist mutates the original output, so it doesn't allocate
        let mut output = vec.clone();
        shuffle_playlist(&mut output);
        dbg!(&output);
        assert!(vec != output, "You probably got astronomically (un)lucky: bogo sort returned _the exact same_ results");
    }
}

