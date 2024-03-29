use std::{
    ffi::CStr,
    fs::{self, DirEntry},
    io,
    ops::Deref,
    path::{Path, PathBuf},
};

use raylib::{
    audio::{Music, RaylibAudio}, rstr, RaylibThread
};

#[derive(Clone)]
pub struct SongEntry {
    path: PathBuf,
    filename: Vec<u8>,
}

impl SongEntry {
    pub fn new(path: PathBuf) -> Option<Self> {
        if let Some(file) = path.file_stem() {
            let mut vec = file.as_encoded_bytes().to_vec();
            if vec.len() < 1 {
                return None;
            }
            if vec[vec.len() - 1] != 0 {
                vec.push(0);
            }
            return Some(Self {
                path,
                filename: vec,
            });
        } else {
            None
        }
    }

    pub fn file_name<'a>(&'a self) -> &'a CStr {
        unsafe { CStr::from_bytes_with_nul_unchecked(&self.filename) }
    }
}

pub struct PlayingSong {
    // path: PathBuf,
    filename: Vec<u8>,
    music: Music,
    idx: usize,
}

impl PlayingSong {
    pub fn new_play(
        entry: &SongEntry,
        idx: usize,
        thread: &RaylibThread,
        audio: &mut RaylibAudio,
    ) -> Result<Self, PlayError> {
        let mut this = Self {
            filename: entry.filename.clone(),
            // path: entry.path.clone(),
            idx,
            music: Music::load_music_stream(
                thread,
                entry.path.to_str().ok_or(PlayError::FileNameInvalid)?,
            )
            .map_err(|err| PlayError::IoError(err))?,
        };
        this.music.looping = false;
        audio.play_music_stream(&mut this.music);

        Ok(this)
    }

    pub fn is_playing(&self, audio: &RaylibAudio) -> bool {
        audio.is_music_stream_playing(&self.music)
    }

    pub fn pause(&mut self, audio: &mut RaylibAudio) {
        audio.pause_music_stream(&mut self.music)
    }

    pub fn resume(&mut self, audio: &mut RaylibAudio) {
        audio.resume_music_stream(&mut self.music)
    }

    pub fn seek(&mut self, seek_to: f32, _audio: &mut RaylibAudio) {
        unsafe { raylib::ffi::SeekMusicStream(*(self.music).deref(), seek_to) }
    }

    pub fn progress(&self, audio: &RaylibAudio) -> f32 {
        audio.get_music_time_played(&self.music) / audio.get_music_time_length(&self.music)
    }

    pub fn get_music_length(&self, audio: &RaylibAudio) -> f32 {
        audio.get_music_time_length(&self.music)
    }

    pub fn get_music_length_played(&self, audio: &RaylibAudio) -> f32 {
        audio.get_music_time_played(&self.music)
    }

    pub fn reached_end(&self, audio: &RaylibAudio) -> bool {
        audio.get_music_time_length(&self.music) - 1.0 <= audio.get_music_time_played(&self.music)
    }

    pub fn update(&mut self, audio: &mut RaylibAudio) {
        audio.update_music_stream(&mut self.music)
    }
}

pub enum RepeatBehavior {
    Normal,
    Repeat,
    RepeatSingle,
}

pub const ICON_REPEAT: &std::ffi::CStr = rstr!("#224#");
pub const ICON_NO_REPEAT: &std::ffi::CStr = rstr!("#222#");
pub const ICON_REPEAT_SINGLE: &std::ffi::CStr = rstr!("#223#");

impl RepeatBehavior {
    pub fn to_icon(&self) -> &std::ffi::CStr {
        match self {
            Self::Normal => ICON_NO_REPEAT,
            Self::Repeat => ICON_REPEAT,
            Self::RepeatSingle => ICON_REPEAT_SINGLE,
        }
    }

    pub fn next(&mut self) {
        match self {
            Self::Normal => *self = Self::Repeat,
            Self::Repeat => *self = Self::RepeatSingle,
            Self::RepeatSingle => *self = Self::Normal,
        }
    }
}

pub struct Playlist {
    songs: Vec<SongEntry>,
    current_song: CurrentSong,
    pub repeat_behavior: RepeatBehavior,
    pub __render_scroll_index: f32,
    pub __render_current_selected: usize,
}

pub enum PlayError {
    OutOfBounds,
    IoError(String),
    FileNameInvalid,
}

impl Default for Playlist {
    fn default() -> Self {
        Self {
            current_song: None,
            __render_scroll_index: 0.0,
            __render_current_selected: 0,
            songs: vec![],
            repeat_behavior: RepeatBehavior::Normal,
        }
    }
}

impl Playlist {
    #[allow(dead_code)]
    pub fn play(
        &mut self,
        idx: usize,
        thread: &RaylibThread,
        audio: &mut RaylibAudio,
        screen_height: i32,
    ) -> Result<(), PlayError> {
        println!("playing song #{idx}");
        if let Some(mut song) = self.current_song.take() {
            if song.idx == idx {
                song.seek(0.0, audio);
                self.current_song = Some(song);
                return Ok(());
            } else {
                song.pause(audio);
            }
        }

        if let Some(song) = self.songs.get(idx) {
            self.current_song = Some(PlayingSong::new_play(song, idx, thread, audio)?);
            self.adjust_center_song(idx, screen_height);
            Ok(())
        } else {
            Err(PlayError::OutOfBounds)
        }
    }

    pub fn play_ignore_err(
        &mut self,
        idx: usize,
        thread: &RaylibThread,
        audio: &mut RaylibAudio,
        screen_height: i32,
    ) {
        println!("playing song #{idx}");
        if let Some(mut song) = self.current_song.take() {
            if song.idx == idx {
                song.seek(0.1, audio);
                self.current_song = Some(song);
                return;
            } else {
                song.pause(audio);
            }
        }

        if let Some(song) = self.songs.get(idx) {
            if let Ok(song) = PlayingSong::new_play(song, idx, thread, audio) {
                self.current_song = Some(song);
                self.adjust_center_song(idx, screen_height);
            }
        }
    }

    pub fn adjust_center_song(&mut self, idx: usize, screen_height: i32) {
        let height = screen_height - 180;
        let offset_top = (height - 30) / 2;
        let y_coord = (idx * 30 + 5) as i32;
        self.__render_scroll_index = -(y_coord - offset_top).max(0) as f32;
    }

    pub fn pause(&mut self, audio: &mut RaylibAudio) {
        if let Some(ref mut song) = self.current_song {
            song.pause(audio)
        }
    }

    #[allow(dead_code)]
    pub fn resume(&mut self, audio: &mut RaylibAudio) {
        if let Some(ref mut song) = self.current_song {
            song.resume(audio)
        }
    }

    pub fn pause_resume(&mut self, audio: &mut RaylibAudio) {
        if let Some(ref mut song) = self.current_song {
            if song.is_playing(audio) {
                song.pause(audio);
            } else {
                song.resume(audio);
            }
        }
    }

    pub fn seek(&mut self, seek_to: f32, audio: &mut RaylibAudio) {
        if let Some(ref mut song) = self.current_song {
            song.seek(seek_to, audio)
        }
    }

    pub fn update(&mut self, audio: &mut RaylibAudio) {
        if let Some(ref mut song) = self.current_song {
            song.update(audio)
        }
    }

    pub fn shuffle(&mut self) {
        let len = self.songs.len();
        if len < 1 { return; }
        let mut tmp_song = self.songs[0].clone();
        for _ in 0..len {
            let max_random_index = (self.songs.len() - 1) as i32;
            let idx_old = unsafe { raylib::ffi::GetRandomValue(0, max_random_index) as usize };
            let idx_new = unsafe { raylib::ffi::GetRandomValue(0, max_random_index) as usize };
            if idx_old == idx_new { continue; }

            if let Some(ref mut song) = self.current_song {
                if song.idx == idx_old {
                    song.idx = idx_new;
                } else if song.idx == idx_new {
                    song.idx = idx_old;
                }
            }

            std::mem::swap(&mut self.songs[idx_new], &mut tmp_song);
            std::mem::swap(&mut self.songs[idx_old], &mut tmp_song);
            std::mem::swap(&mut self.songs[idx_new], &mut tmp_song);
        }
    }

    pub fn len(&self) -> usize {
        self.songs.len()
    }

    pub fn currently_playing_id(&self) -> Option<usize> {
        match self.current_song {
            Some(ref song) => Some(song.idx),
            _ => None,
        }
    }

    pub fn stop_playing(&mut self, audio: &mut RaylibAudio) {
        self.pause(audio);
        self.current_song = None;
    }

    pub fn has_music_stream(&self) -> bool {
        self.current_song.is_some()
    }

    pub fn is_music_playing(&self, audio: &RaylibAudio) -> bool {
        if let Some(ref song) = self.current_song {
            song.is_playing(audio)
        } else {
            false
        }
    }

    pub fn music_has_reached_the_end(&self, audio: &RaylibAudio) -> bool {
        if let Some(ref song) = self.current_song {
            song.reached_end(audio)
        } else {
            false
        }
    }

    pub fn progress(&self, audio: &RaylibAudio) -> f32 {
        match self.current_song {
            None => 0.0,
            Some(ref song) => song.progress(audio),
        }
    }

    pub fn music_length_played(&self, audio: &RaylibAudio) -> f32 {
        match self.current_song {
            None => 0.0,
            Some(ref song) => song.get_music_length_played(audio),
        }
    }

    pub fn music_length_total(&self, audio: &RaylibAudio) -> f32 {
        match self.current_song {
            None => 0.0,
            Some(ref song) => song.get_music_length(audio),
        }
    }

    pub fn filename_vec<'a>(&'a self) -> Option<&'a Vec<u8>> {
        match self.current_song {
            Some(ref song) => Some(&song.filename),
            _ => None,
        }
    }

    pub fn clear(&mut self, audio: &mut RaylibAudio) {
        self.songs.clear();
        self.stop_playing(audio);
    }

    pub fn get_songs(&self) -> &Vec<SongEntry> {
        &self.songs
    }

    pub fn add_song(&mut self, entry: SongEntry) {
        self.songs.push(entry);
    }

    pub fn add_song_by_path<P: AsRef<Path>>(&mut self, path: P) -> io::Result<()> {
        let metadata = fs::metadata(&path)?;
        if metadata.is_dir() {
            load_dir_recursively_mut_vec(&path, self);
        } else if metadata.is_file() {
            if let Some(extension) = path.as_ref().extension() {
                if SUPPORTED_FORMATS
                    .iter()
                    .find(|&&ext| ext == extension)
                    .is_some()
                {
                    SongEntry::new(path.as_ref().to_path_buf()).map(|entry| self.add_song(entry));
                }
            }
        }

        Ok(())
    }

    #[allow(dead_code)]
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) {
        match path.as_ref().extension() {
            Some(ext) => {
                if ext != "m3u" {
                    return;
                }
            }
            _ => return,
        }

        let mut str = String::with_capacity(self.len() * 30); // just assuming that each path is around 30 characters long

        for e in &self.songs {
            if let Some(path) = e.path.to_str() {
                str.push_str(path);
            }
        }

        let _ = fs::write(path, str);
    }

    #[allow(dead_code)]
    pub fn from_file<P: AsRef<Path>>(path: P) -> Self {
        let mut me = Self::default();
        me.load_from_file(path);
        me
    }

    pub fn load_from_file<P: AsRef<Path>>(&mut self, path: P) {
        match path.as_ref().extension() {
            Some(ext) => {
                if ext != "m3u" {
                    return;
                }
            }
            _ => return,
        }

        let str = match fs::read_to_string(path) {
            Ok(v) => v,
            _ => return,
        };
        for file in str.split('\n') {
            let _ = self.add_song_by_path(file);
        }
    }
}

pub type CurrentSong = Option<PlayingSong>;

pub const SUPPORTED_FORMATS: &[&str] = &["mp3", "ogg", "wav", "qoa", "flac", "xm", "mod"];

fn process_entry(path: &dyn AsRef<Path>, entry: &DirEntry, playlist: &mut Playlist) {
    let typ = match entry.file_type() {
        Ok(v) => v,
        _ => return,
    };

    if typ.is_dir() {
        load_dir_recursively_mut_vec(
            &Path::join(path.as_ref(), entry.file_name()).as_path(),
            playlist,
        );
    } else if typ.is_file() {
        let file_path = entry.file_name();
        let file_path = Path::new(&file_path);
        if let Some(extension) = file_path.extension() {
            if SUPPORTED_FORMATS
                .iter()
                .find(|&&ext| ext == extension)
                .is_some()
            {
                SongEntry::new(Path::join(path.as_ref(), entry.file_name()))
                    .map(|entry| playlist.add_song(entry));
            } else if extension == "m3u" {
                playlist.load_from_file(Path::join(path.as_ref(), entry.file_name()))
            }
        }
    }
}

fn load_dir_recursively_mut_vec(path: &dyn AsRef<Path>, vec: &mut Playlist) -> Option<()> {
    for entry in fs::read_dir(path).ok()? {
        if let Ok(entry) = entry {
            process_entry(path, &entry, vec);
        }
    }

    Some(())
}
