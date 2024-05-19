use std::{fs, path::PathBuf};

use raylib::audio::RaylibAudio;

// #[macro_export]
// macro_rules! cstr {
//     ($str: expr) => {
//         unsafe { std::ffi::CStr::from_bytes_with_nul_unchecked(concat!($str, "\0").as_bytes()) }
//     };
// }

mod file_gui;
mod gui_lyrics;
mod gui_main;
mod song;
use song::Playlist;

use crate::{
    file_gui::FileGuiState,
    gui_lyrics::{render_lyrics_gui, LyricsGuiState},
    gui_main::{render_main_gui, Action, MainGuiState},
};

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum GuiScreen {
    Player,
    Lyrics,
    FileSelectAddFolder,
    FileSelectAddFile,
    FileSelectOpenFolder,
    FileSelectOpenFile,
    FileSelectSaveFile,
}

fn main() {
    let homedir = get_home_directory().expect("Failed to get the home directory!");
    let musicdir = match fs::read_dir(&homedir) {
        Err(..) => panic!("Failed to read the home directory!"),
        Ok(mut dirs) => dirs
            .find(|dir| {
                if let Ok(dir) = dir {
                    dir.file_name().to_ascii_lowercase() == "music"
                } else {
                    false
                }
            })
            .and_then(|val| val.ok()),
    };
    println!("Home: {}", homedir.display());
    let musicdir = match musicdir {
        Some(v) => homedir.join(v.file_name()),
        None => panic!("Failed to read the music directory!"),
    };
    if fs::read_dir(&musicdir).is_err() {
        panic!("Failed to read the music directory");
    }
    println!("Music: {}", musicdir.display());

    println!("Initializing Raylib");

    // register ICON_FOLDER
    load_custom_icon(
        217,
        [
            0x0, 0x0042007e, 0x40027fc2, 0x40024002, 0x40024002, 0x40024002, 0x7ffe4002, 0x0,
        ],
    );
    // register ICON_FILE
    load_custom_icon(
        218,
        [
            0x3ff00000, 0x201c2010, 0x20042004, 0x20042004, 0x20042004, 0x20042004, 0x20042004,
            0x00003ffc,
        ],
    );
    // register ICON_LYRICS
    load_custom_icon(
        219,
        [
            0x0, 0x18000800, 0x28002bfc, 0x0e0008fc, 0x00000efc, 0x00003ffc, 0x00003ffc, 0x0
        ],
    );
    // register ICON_AUDIO_MUTE
    load_custom_icon(
        220,
        [
            0x20000, 0x18a808c4, 0x32a81290, 0x248226c6, 0x26862582, 0x1a903688, 0x28c018a0, 0x4000,
        ],
    );
    // register ICON_FOLDER_ADD
    load_custom_icon(
        221,
        [
            0x0, 0x0042007e, 0x40027fc2, 0x44024002, 0x5f024402, 0x44024402, 0x7ffe4002, 0x0,
        ],
    );
    // register ICON_NO_REPEAT
    load_custom_icon(
        222,
        [
            0x0, 0x06000200, 0x06040ffc, 0x00040204, 0x00040004, 0x00040004, 0x0, 0x0,
        ],
    );
    // register ICON_REPEAT_SINGLE
    load_custom_icon(
        223,
        [
            0x0, 0x3ffc0000, 0x21042004, 0x21002180, 0x23a02100, 0x3ff82030, 0x00200030, 0x0,
        ],
    );
    // register ICON_REPEAT
    load_custom_icon(
        224,
        [
            0x0, 0x3ffc0000, 0x20042004, 0x20002000, 0x20202000, 0x3ff82030, 0x00200030, 0x0,
        ],
    );

    let (mut rl, thread) = raylib::init()
        .width(350)
        .height(500)
        .title("MP3 Player")
        .undecorated()
        .build();
    rl.set_target_fps(60);
    rl.set_exit_key(None);

    let mut audio = RaylibAudio::init_audio_device();

    let mut playlist: Playlist = Default::default();

    playlist.clear(&mut audio);
    // load_dir_recursively_mut_vec(&musicdir, &mut playlist);
    playlist.play_ignore_err(0, &thread, &mut audio, rl.get_screen_height());

    let mut state_maingui: MainGuiState = Default::default();
    let mut state_lyricsgui: LyricsGuiState = Default::default();
    let mut state_filegui: FileGuiState = FileGuiState::default(&musicdir, GuiScreen::Player)
        .expect("Failed to initialise the file gui");
    let mut cur_screen: GuiScreen = GuiScreen::Player;

    // todo:
    // - file selection dialogues
    // - moving the music update function out of render_main_gui(..)

    while !rl.window_should_close() {
        let action = match cur_screen {
            GuiScreen::Player => render_main_gui(
                &mut audio,
                &mut playlist,
                &thread,
                &mut rl,
                &mut state_maingui,
            ),
            GuiScreen::Lyrics => render_lyrics_gui(&mut playlist, &thread, &mut rl, &mut state_lyricsgui),
            GuiScreen::FileSelectAddFolder
            | GuiScreen::FileSelectAddFile
            | GuiScreen::FileSelectOpenFolder
            | GuiScreen::FileSelectOpenFile
            | GuiScreen::FileSelectSaveFile => file_gui::render_file_gui(
                &mut rl,
                &mut playlist,
                &mut state_filegui,
                cur_screen,
                &thread,
                &mut audio,
            ),
        };

        match action {
            Action::None => {}
            Action::ExitProgram => break,
            Action::SwitchGuiScreen(screen @ (GuiScreen::Player | GuiScreen::Lyrics)) => {
                state_maingui = Default::default();
                state_lyricsgui = Default::default();
                cur_screen = screen;
            }
            Action::SwitchGuiScreen(screen) => {
                if let Ok(state) = FileGuiState::default(&musicdir, screen) {
                    state_filegui = state;
                    cur_screen = screen;
                    state_maingui = Default::default();
                } else {
                    cur_screen = GuiScreen::Player;
                }
            }
        }

        gui_main::update_music(
            &mut audio,
            &mut playlist,
            &thread,
            &mut rl,
            &mut state_maingui,
        );
    }
}

fn get_home_directory() -> Option<PathBuf> {
    let mut path = PathBuf::new();
    if cfg!(target_os = "windows") {
        // %HomeDrive%%HomePath% aka $HomeDrive$HomePath
        path.push(std::env::var("HomeDrive").ok()?);
        path.push(std::env::var("HomePath").ok()?);
        panic!("FUCK THIS BS")
    } else if cfg!(any(
        target_os = "linux",
        target_os = "macos",
        target_os = "freebsd",
        target_os = "dragonfly",
        target_os = "openbsd",
        target_os = "netbsd"
    )) {
        // $HOME
        path.push(std::env::var("HOME").ok()?);
    } else if cfg!(target_os = "none") {
        panic!("Embedded architecture isnt supported yet :c");
    } else {
        panic!("Unsupported target_os cfg: {}", std::env::consts::OS);
    }
    Some(path)
}

fn load_custom_icon(id: u8, icon: [u32; 8]) {
    let ptr = unsafe { raylib::ffi::GuiGetIcons().offset(id as isize * 8) };
    unsafe {
        for i in 0..8usize {
            *(ptr.offset(i as isize)) = icon[i];
        }
    }
}
