// this crappy TUI engine is very high overhead; the code, in debug, has a catastrophically high,
// 0.0% CPU usage. This was done on an AMD A4-6210 with AMD Radeon R3 Graphics (4) @ 1.80 GHz.

#![allow(unused_must_use)]

use std::io::{stdout, StdoutLock, BufWriter, Write};
use std::sync::atomic::Ordering::Relaxed;
use encore::{RenderMode, LoopMode};
use crate::{SONG_INDEX, PLAYLIST, SONG_CURRENT_LEN, SONG_TOTAL_LEN, VOLUME_LEVEL, LOOP_MODE};

macro_rules! not_enough_space {
    ($tooey:expr) => {{
        $tooey.render_set_mode(RenderMode::NoSpace);
        // forgive me for this unfortunate error message.
        return Err("No room".into());
    }}
}

pub struct Tui<'a> {
    handle: BufWriter<StdoutLock<'a>>,
    rendering_mode: RenderMode,

    width: u16,
    height: u16,
    scrolling_offset: usize,
    pub cursor_index_queue: usize,
}

impl Tui<'_> {
    /// creates and primes the Tooey type, which... does the tui stuff
    ///
    /// it is recommended to create this on another thread.
    pub fn init() -> Tui<'static> {
        // lock stdout for perf; no other component should write directly there.
        // panic! writes to stderr, and can be captured via redirection (2>)
        let stdout = stdout().lock();
        // to avoid excessive syscalls (which yields the current thread and requires a context
        // switch, so increases overhead on the system itself), we buffer the stdout.
        let handle = BufWriter::new(stdout);

        Tui {
            handle,
            rendering_mode: RenderMode::default(),
            width: 0,
            height: 0,
            scrolling_offset: 0,
            cursor_index_queue: 0,
        }
    }

    fn determine_terminal_size(&mut self) {
        use terminal_size::{Width, Height, terminal_size};

        let (Width(width), Height(height)) = terminal_size().unwrap();
        self.width = width;
        self.height = height;
    }

    fn render_set_mode(&mut self, mode: RenderMode) {
        self.rendering_mode = mode;
    }

    pub fn with_rendering_mode(mut self, mode: RenderMode) -> Self {
        self.render_set_mode(mode);
        self
    }

    pub fn tick(&mut self) {
        let time = std::time::Instant::now();
        self.rerender_display();
        writeln!(self.handle, "time taken to draw last frame: {}µs", time.elapsed().as_micros());
        self.handle.flush();
    }

    fn rerender_display(&mut self) {
        self.__pre_rerender_display();
        if let Err(err) = self.__rerender_display() {
            if self.rendering_mode == RenderMode::NoSpace {
                self.rerender_display(); // rerender the nospace view right now, instead of waiting 1s
            } else {
                eprintln!("Unrecoginized error: {err}");
            }
        }
    }

    fn __pre_rerender_display(&mut self) {
        self.determine_terminal_size();
        self.__calculate_offset(); // needs to be calculated at some point after finding term size,
                                   // because if working on old values, its not gonna work.
        self.cursor_index_queue = SONG_INDEX.load(Relaxed);

        self.__blankout_terminal();
    }

    fn __rerender_display(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        match self.rendering_mode {
            RenderMode::Full => self.__draw_full()?,
            RenderMode::Safe => self.__draw_safe()?,
            RenderMode::NoSpace => {
                self.__draw_safe()?;
                self.render_set_mode(RenderMode::Full);
            }
        }

        self.render_misc_info();

        Ok(())
    }

    fn render_misc_info(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let loop_mode: LoopMode = LOOP_MODE.load(Relaxed).into();
        writeln!(self.handle, "Loop: {loop_mode}")?;

        Ok(())
    }

    fn __calculate_offset(&mut self) {
        if self.cursor_index_queue >= (self.height as usize).saturating_sub(13) + self.scrolling_offset {
            // HACK: if last element in playlist, don't increment the offset
            if self.cursor_index_queue + 1 + self.scrolling_offset == PLAYLIST.read().unwrap().len() {
                return;
            }
            self.scrolling_offset += 1;
        }
        else if self.cursor_index_queue.saturating_sub(1) < self.scrolling_offset {
            self.scrolling_offset -= 1;
        }
    }

    fn __draw_full(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let songs = &PLAYLIST.read().unwrap();

        if self.cursor_index_queue >= songs.len() {
            // wrap back to the size of songs; the user is trying to access songs.len() + 1
            // will panic otherwise, but callers dont need to care
            self.cursor_index_queue = songs.len() - 1;
            SONG_INDEX.store(self.cursor_index_queue, Relaxed);
        } else if self.scrolling_offset >= songs.len() {
            self.scrolling_offset = songs.len() - 1;
        }
        writeln!(self.handle, "current song index: {}, SONG_INDEX: {}, len: {}", self.cursor_index_queue, SONG_INDEX.load(Relaxed), songs.len());
        self.handle.flush();
        // TODO: make this only calculate once in determine_terminal_size, when size changes?
        let opening_box = draw_box::<true>("queue", self.width);
        let closing_box = draw_box::<false>("", self.width);
        let opening_box1 = draw_box::<true>("", self.width);
        let closing_box2 = draw_box::<false>("asdadsad", self.width);

        writeln!(self.handle, "{opening_box}");

        // HACK: for some reason, this code thinks cursor_index_queue^self.scrolling_offset is the
        // currently selected song. subtract it now.
        // i will give you a hug if you find out why that is, and a workaround that isn't this ugly.
        self.cursor_index_queue = self.cursor_index_queue.saturating_sub(self.scrolling_offset);
        let times = (self.height as usize).saturating_sub(12) + self.scrolling_offset;
        // 0 also works, but it seems to switch between 1 and 0, so bail on 1
        // the terminal won't appear empty in that case, but with only one entry
        // i think that's fair.
        if times <= 2 {
            not_enough_space!(self); // should this be reached, the terminal's height is not large
                                     // enough, and the playlist will appear empty... and also the
                                     // current playing song indicator will be alternating between
                                     // two songs.
                                     // also monospace <3
        }
        for i in 0..times {
            if i < self.scrolling_offset {
                continue;
            }
            if self.scrolling_offset + i >= songs.len() {
                continue;
            }
            if i >= songs.len() {
                // fill all empty spaces
                let entry = self.draw_entry("").unwrap();
                write!(self.handle, "{entry}");
                continue;
            }

            let line = songs[i + self.scrolling_offset].split('/').next_back().unwrap_or("");
            let line = &ellipsize(line, self.width as usize - 2, EllipsizeMode::End);
            let mut entry: String = String::with_capacity(self.width.into());
            if i == self.cursor_index_queue {
                entry.push_str(&self.draw_highlighted_entry(line)?);
            } else {
                entry.push_str(&self.draw_entry(line)?);
            }
            write!(self.handle, "{entry}");
        }
        write!(self.handle, "{closing_box}");

        let line = songs[self.cursor_index_queue + self.scrolling_offset].split('/').next_back().unwrap_or("");
        let line = &ellipsize(line, self.width as usize - 2, EllipsizeMode::End);
        let line = self.draw_entry_centered(line)?;
        // playback bar
        write!(self.handle, "{opening_box1}");
        write!(self.handle, "{line}");
        write!(self.handle, "{closing_box2}");
        writeln!(self.handle, "{}, {}", self.scrolling_offset, self.cursor_index_queue);

        Ok(())
    }

    fn __blankout_terminal(&mut self) {
        write!(self.handle, "\x1b[2J\x1b[H"); // top left corner; clear screen
    }

    fn __draw_safe(&mut self) -> Result<(), std::io::Error> {
        let songs = &PLAYLIST.read().unwrap();
        if self.cursor_index_queue >= songs.len() {
            self.cursor_index_queue = songs.len() - 1;
            SONG_INDEX.store(self.cursor_index_queue, Relaxed);
        }
        let song = songs[self.cursor_index_queue].split('/').last().unwrap_or("");

        writeln!(self.handle, "{song}");
        let current_len = format_time(SONG_CURRENT_LEN.load(Relaxed));
        let total_len = format_time(SONG_TOTAL_LEN.load(Relaxed));
        let vol = f32_to_percent(VOLUME_LEVEL.load(Relaxed));
        writeln!(self.handle, "{current_len} / {total_len}");
        writeln!(self.handle, "󰕾 {vol}%");

        Ok(())
    }

    fn __draw_not_enough_space(&mut self) -> Result<(), std::io::Error> {
        writeln!(self.handle, "Encore error\n")?;
        writeln!(self.handle, "Not enough space for the terminal!")?;
        writeln!(self.handle, "Resize your terminal in order to see the queue. Keyboard input is still functional.")?;
        writeln!(self.handle, "To suppress this message, enter rm -rf /* in another shell session running under UID0 (root).")?;
        self.render_set_mode(RenderMode::Full); // TODO: change this to know what was there
                                                // previously

        Ok(())
    }

    pub fn enter_alt_buffer(&mut self) -> Result<(), std::io::Error> {
        writeln!(self.handle, "\x1B[?1049h")?;
        Ok(())
    }

    pub fn leave_alt_buffer(&mut self) -> Result<(), std::io::Error> {
        writeln!(self.handle, "\x1B[?1049l")?;
        Ok(())
    }

    fn draw_entry_centered(&mut self, text: &str) -> Result<String, Box<dyn std::error::Error>> {
        let padding = 0;

        let pad_len = self.width as usize - text.len();
        let mut ntext = String::with_capacity((self.width - 2).into());

        // :(
        // to see why this is here, run this on a terminal whose width is 84 chars with the song
        // name:
        // /home/william/Desktop/encorets_audio/badapple.mp3
        // TODO: get rid of this somehow
        if text.len() % 2 == 0 && self.width % 2 == 0 {
            ntext.push_str(&" ".repeat(pad_len - 2));
        } else {
            ntext.push_str(&" ".repeat(pad_len));
        }
        // put this here to hopefully center the text if both self.width and text.len's remainders
        // after a division of 2 equal 0
        if text.len() % 2 == 0 {
            ntext.push(' ');
        }
        ntext.push_str(text);
        ntext.push_str(&" ".repeat(pad_len));
        if self.width % 2 == 0 {
            ntext.push(' ');
        }

        Ok(box_draw_entry(&ntext, padding))
    }

    fn draw_entry(&mut self, text: &str) -> Result<String, Box<dyn std::error::Error>> {
        let width = self.width as usize;
        let padding = width - (text.len() + 2);

        Ok(box_draw_entry(text, padding))
    }

    fn draw_highlighted_entry(&mut self, text: &str) -> Result<String, Box<dyn std::error::Error>> {
        let width = self.width as usize;
        let padding = width - (text.len() + 2);

        let out = format!("\x1B[48;2;245;194;231m\x1B[38;2;30;30;46m{text}");
        Ok(box_draw_entry(&out, padding))
    }
}

impl Drop for Tui<'_> {
    fn drop(&mut self) {
        self.leave_alt_buffer().unwrap();
    }
}

fn format_time(t: u64) -> String {
    let hrs =   t / 3600;
    let mins = (t % 3600) / 60;
    let secs =  t % 60;

    if hrs == 0 {
        format!("{mins:02}:{secs:02}")
    } else {
        format!("{hrs:02}:{mins:02}:{secs:02}")
    }
}

/// nah not really you need to append the % yourself
fn f32_to_percent(f: f32) -> f32 {
    ((f * 100.0).trunc() / 10.0).round() * 10.0
}

fn box_draw_entry(text: &str, padding: usize) -> String {
    format!("│{}{}{}", text, &" ".repeat(padding), "\x1B[0m│")
}

fn draw_box<const CLOSING: bool>(text: &str, term_len: u16) -> String {
    let first = if CLOSING { "╭─" } else { "╰" };
    let adding = if CLOSING { 3 } else { 2 };
    let closing = if CLOSING { "╮" } else { "╯" };

    let trailing = if CLOSING {
        "─".repeat((term_len - adding - text.len() as u16).into())
    } else {
        "─".repeat((term_len - adding).into())
    };

    if CLOSING {
        format!("{first}{text}{trailing}{closing}")
    } else {
        format!("{first}{trailing}{closing}")
    }
}

#[allow(unused)] // TODO: these are marked as unused as of right now, because EllipsizeMode::End is
                 // hardcoded.
enum EllipsizeMode {
    Beginning,
    Middle,
    End,
}

fn ellipsize(s: &str, max_len: usize, mode: EllipsizeMode) -> String {
    if s.len() <= max_len {
        return s.to_string();
    }

    let ellipsis = "...";
    let ellipsis_len = ellipsis.len();

    if max_len <= ellipsis_len {
        return ellipsis.to_string();
    }

    match mode {
        EllipsizeMode::Beginning => format!("{}{}", ellipsis, &s[(s.len() - max_len + ellipsis_len)..]),
        EllipsizeMode::Middle => {
            let part_len = (max_len - ellipsis_len) / 2;
            format!("{}{}{}", &s[..part_len], ellipsis, &s[s.len() - part_len..])
        }
        EllipsizeMode::End => format!("{}{}", &s[..max_len - ellipsis_len], ellipsis),
    }
}

