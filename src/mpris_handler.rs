use std::sync::{mpsc, Arc};

type Tx = Arc<mpsc::Sender<encore_shared::SongControl>>;

#[cfg(feature = "mpris")]
mod inner {
    use encore_shared::SongControl;
    use souvlaki::{MediaControlEvent, MediaControls, MediaMetadata, MediaPlayback, MediaPosition, PlatformConfig};
    use std::time::Duration;
    use std::sync::atomic::Ordering;
    use std::sync::{mpsc, Arc};

    pub(super) struct Inner {
        pub controls: MediaControls
    }

    impl Inner {
        pub fn new() -> Inner {
            #[cfg(target_os = "windows")]
            todo!("implement hWnd bullshit. use linux in the meantime.");

            let hwnd = None;

            let config = PlatformConfig {
                dbus_name: "encore",
                display_name: "Encore",
                hwnd,
            };

            let controls = MediaControls::new(config).expect("hmm");

            Inner {
                controls,
            }
        }

        pub fn update(&mut self) {
            let song_len = Duration::from_secs(crate::SONG_CURRENT_LEN.load(Ordering::Relaxed));
            let total_song_len = Duration::from_secs(crate::SONG_TOTAL_LEN.load(Ordering::Relaxed));

            let song_index = crate::SONG_INDEX.load(Ordering::Relaxed);
            let title = crate::PLAYLIST.read().unwrap();

            let metadata = MediaMetadata {
                title: Some(encore_shared::trim_path(&title[song_index])),
                album: None, // TODO: playlist name
                artist: None, // TODO: implement artist readout
                duration: Some(total_song_len),
                cover_url: None,
            };

            let playback = {
                if crate::PAUSED.load(Ordering::Relaxed) {
                    MediaPlayback::Paused { progress: Some(MediaPosition(song_len)) }
                } else {
                    MediaPlayback::Playing { progress: Some(MediaPosition(song_len)) }
                }
            };
            self.controls
                .set_playback(playback).unwrap();

            self.controls.set_metadata(metadata).unwrap();
        }

        pub fn attach(&mut self, tx: super::Tx) -> Option<()> {
            let attach = self.controls.attach(move |ev| on_media_event(ev, tx.clone()));
            if attach.is_err() {
                return None;
            }

            Some(())
        }
    }

    pub fn on_media_event(ev: MediaControlEvent, tx: Arc<mpsc::Sender<encore_shared::SongControl>>) {
        use souvlaki::MediaControlEvent;

        let r = match ev {
            MediaControlEvent::Pause => SongControl::Pause,
            MediaControlEvent::Play => SongControl::Resume,
            MediaControlEvent::Toggle => SongControl::TogglePause,
            MediaControlEvent::Next => SongControl::NextSong,
            MediaControlEvent::Previous => SongControl::PrevSong,
            MediaControlEvent::Stop => SongControl::DestroyAndExit,
            x => unimplemented!("got event {:?}. how'd you get here?", x),
        };

        tx.send(r).expect("wtf");
    }
}

#[cfg(not(feature = "mpris"))]
mod inner {
    use std::sync::{mpsc, Arc};

    pub(super) struct Inner;

    impl Inner {
        #[inline] pub fn new() -> Inner { Inner }

        #[inline] pub fn update(&mut self) { }

        #[inline] pub fn attach(&mut self, _tx: super::Tx) -> Option<()> {
            None
        }
    }
}

pub struct MediaInfo {
    // TODO: would be more modular if trait object is used.
    inner: inner::Inner,
}

impl MediaInfo {
    pub fn new() -> MediaInfo {
        let inner = inner::Inner::new();

        MediaInfo {
            inner
        }
    }

    pub fn update(&mut self) {
        self.inner.update();
    }

    pub fn attach(&mut self, tx: Tx) -> Option<()> {
        self.inner.attach(tx)
    }
}

