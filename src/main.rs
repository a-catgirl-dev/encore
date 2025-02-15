mod song;
mod input;
mod tui;
mod file_format;
mod configuration;
mod threading;

mod mpris_handler;

#[macro_use]
mod macros;

use std::sync::{atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering::Relaxed}, RwLock, mpsc::channel, Arc};
use std::{io::BufReader, fs::File};
use encore::IntegerExtensions;

use threading::ThreadAbstraction;

#[cfg(feature = "dhat-heap")]
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

lazy_static::lazy_static!{
    static ref PLAYLIST: RwLock<Vec<String>> = Default::default();
    static ref CFG_IS_LOOPED: AtomicBool = AtomicBool::new(false);
    static ref SONG_INDEX: AtomicUsize = AtomicUsize::new(0);
    static ref SONG_TOTAL_LEN: AtomicU64 = AtomicU64::new(0);
    static ref SONG_CURRENT_LEN: AtomicU64 = AtomicU64::new(0);
    static ref PAUSED: AtomicBool = AtomicBool::new(false);
    static ref VOLUME_LEVEL: encore::AtomicF32 = encore::AtomicF32::new(0.0);
}

/// skip_num parameter for skipping the first n elements in the vec.
/// needed cuz ./encore 1.mp3 will make argv = ["encore", "1.mp3"], and try to parse argv[0] = "encore"
fn parse_playlist(file: &Vec<String>, skip_num: usize) -> Result<(), Box<dyn std::error::Error>> {
    let mut lines = PLAYLIST.write().unwrap();
    for s in file.iter()
        .skip(skip_num)
        .by_ref()
    {
        let f = if let Ok(k) = File::open(s) { k } else {
            eprintln!("{s} doesn't exist");
            continue
        };
        let mut f = BufReader::new(f);
        if file_format::check_file(&mut f)? != encore::FileFormat::Audio {
            eprintln!("Removing {s} from playlist: not audio file. skipping...");
            continue;
        }
        lines.push(s.to_owned());
    }

    Ok(())
}

fn quit_with(e: &str, s: &str) -> Result<std::convert::Infallible, Box<dyn std::error::Error>> {
    eprintln!("{e}");
    Err(s.into())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // use std::thread::spawn;
    use std::time::Duration;
    use encore::{SongControl::*, RenderMode, FileFormat};

    #[cfg(feature = "dhat-heap")]
    let _profiler = dhat::Profiler::new_heap();

    let cfg = configuration::Config::parse(&encore::ConfigurationPath::Default);

    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        quit_with("argv[1] should be a media file or Encore-compatable playlist.", "argv[1] not supplied")?;
    }

    let mut render_requested_mode = RenderMode::default();

    if args.len() == 2 {
        let mut first_arg = if let Some(s) = encore::normalize_line(&args[1]) { BufReader::new(File::open(s)?) }
        else {
            quit_with("No such file or directory", "argv[1] not found.")?;
            unreachable!("quit_with must quit")
        };
        match file_format::check_file(&mut first_arg).unwrap() {
            FileFormat::Audio => {
                let playlist = encore::normalize(&mut args.into_iter());
                parse_playlist(&playlist, 1).unwrap();
            }
            FileFormat::Other => {
                let possible_playlist = encore::to_vec(&mut first_arg).expect("valid utf8");
                parse_playlist(&possible_playlist, 0).unwrap();
            }
        }
    } else {
        let file = encore::normalize(&mut args.into_iter());
        parse_playlist(&file, 1).unwrap();
    }

    let playlist_len = PLAYLIST.read().unwrap().len();
    if playlist_len == 0 {
        quit_with("no songs in playlist array; are all of the paths valid?", "playlist file has zero length")?;
    }
    if playlist_len == 1 {
        render_requested_mode = RenderMode::Safe;
    }

    let (main_tx, main_rx) = channel();
    let main_tx = Arc::new(main_tx);
    let audio_over_mtx = main_tx.clone();
    let ctrlc_mtx = main_tx.clone();
    let mpris_mtx = main_tx.clone();

    let (render_tx, render_rx) = channel();
    let render = ThreadAbstraction::spawn(move || {
        let mut tui = tui::Tui::init()
            .with_rendering_mode(render_requested_mode);
        tui.enter_alt_buffer().unwrap();
        loop {
            tui.tick();
            let receive = render_rx.recv_timeout(Duration::from_secs(1));
            if let Ok(k) = receive {
                match k {
                    DestroyAndExit => break, // the destructor will exit the alt buffer
                    ToggleLoop => CFG_IS_LOOPED.store(!CFG_IS_LOOPED.load(Relaxed), Relaxed),
                    _ => {
                        #[cfg(debug_assertions)]
                        eprintln!("the operation {k:?} is not applicable for rendering");
                    }
                };
            }
        }
    });

    let _input = ThreadAbstraction::spawn(move || {
        let input = input::Input::from_nothing_and_apply();
        loop {
            let i = input.blocking_wait_for_input();
            match i {
                DestroyAndExit => {
                    send_control_errorless!(DestroyAndExit, ctrlc_mtx);
                    break;
                },
                No => (), // there is nothing
                signal => {
                    send_control_errorless!(signal, main_tx);
                }
            }
        }
    });

    let (mpris_tx, mpris_rx) = channel();
    let mpris = ThreadAbstraction::spawn_if(move || {
        let mut media = mpris_handler::MediaInfo::new();

        // let has_dbus = media.attach(move |e| mpris_handler::on_media_event(e, mpris_mtx.clone()));
        let has_dbus = media.attach(mpris_mtx);
        if has_dbus.is_none() {
            eprintln!("Disabling mpris due to lack of dbus...");
            return;
        }
        loop {
            media.update();
            let receive = mpris_rx.recv_timeout(Duration::from_secs(1));
            if receive == Ok(DestroyAndExit) { break };
        }
        // controls.detach is a little slow...
    }, cfg!(feature = "mpris"));

    let mut audio = song::Song::new();
    audio.play();
    loop {
        let receive = main_rx.recv_timeout(Duration::from_secs(1));
        if let Ok(k) = receive {
            match k {
                DestroyAndExit => {
                    send_control!(DestroyAndExit, render_tx);
                    send_control_errorless!(DestroyAndExit, mpris_tx); // may fail
                    drop(audio.sink); // stop audio now

                    // wait for the threads to finish
                    // FIXME: input doesnt seem to work. it hangs.
                    __exit_await_thread!(render, mpris);

                    break;
                }
                NextSong => {
                    let playlist_len = PLAYLIST.read().unwrap().len();
                    if playlist_len == 1 {
                        continue;
                    }
                    let i = SONG_INDEX.load(Relaxed);
                    if i >= playlist_len - 1 {
                        continue;
                    }
                    SONG_INDEX.store(i + 1, Relaxed);
                    audio.rejitter_song();
                }
                PrevSong => {
                    let sub = match SONG_INDEX.load(Relaxed).checked_sub(1) {
                        Some(n) => n,
                        None => continue,
                    };
                    SONG_INDEX.store(sub, Relaxed);
                    audio.rejitter_song();
                }
                TogglePause => if audio.sink.is_paused() { audio.play() } else { audio.pause() }
                // rodio docs sez: `pause()` and `play()` don't have any effects if already paused/playing
                Pause => audio.pause(),
                Resume => audio.play(),
                VolumeUp => {
                    let prev_vol = audio.sink.volume();
                    if prev_vol >= cfg.main.max_vol.to_rodio() {
                        continue;
                    }
                    audio.sink.set_volume(prev_vol + 0.1);
                },
                VolumeDown => {
                    let prev_vol = audio.sink.volume();
                    if prev_vol <= cfg.main.min_vol.to_rodio() {
                        continue;
                    }
                    let request_vol = prev_vol - 0.1;
                    // no .saturating_sub for f32 cause primitive type, so we do this:
                    let normalized_vol = if request_vol < 0.0 { 0.0 } else { request_vol };
                    audio.sink.set_volume(normalized_vol);
                },
                // seeking may fail. if so, then silently fail, because who cares??
                SeekForward => {
                    let _ = audio.sink.try_seek(audio.sink.get_pos() + std::time::Duration::from_secs(5));
                }
                SeekBackward => {
                    let _ = audio.sink.try_seek(audio.sink.get_pos().saturating_sub(std::time::Duration::from_secs(5)));
                }
                _ => {
                    #[cfg(debug_assertions)]
                    eprintln!("the operation {k:?} is not applicable for audio");
                }
            }

            send_control_errorless!(k, render_tx, mpris_tx);
        }

        if audio.sink.empty() {
            // FIXME: this is a little unclean, and may be hard to understand
            let song_index = SONG_INDEX.load(Relaxed);
            if CFG_IS_LOOPED.load(Relaxed) {
                audio.rejitter_song();
                continue;
            } else if song_index >= PLAYLIST.read().unwrap().len() - 1 { // playlist len always + 1 because math
                send_control_errorless!(DestroyAndExit, audio_over_mtx);
            }
            SONG_INDEX.store(song_index + 1, Relaxed);
            audio.rejitter_song();
        } else {
            // task: synchronise global variables based on what we have.

            // there is a bug here: sometimes, this returns None.
            // some mp3s work, but others don't. i dont know why precisely.
            let total_dur = match audio.total_duration {
                Some(n) => n.as_secs(),
                None => 0,
            };
            SONG_CURRENT_LEN.store(audio.sink.get_pos().as_secs(), Relaxed);
            SONG_TOTAL_LEN.store(total_dur, Relaxed);
            PAUSED.store(audio.sink.is_paused(), Relaxed);

            VOLUME_LEVEL.store(audio.sink.volume(), Relaxed);
        }
    }

    Ok(())
}

