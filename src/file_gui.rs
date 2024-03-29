use std::{
    ffi::{CString, OsString},
    fs,
    path::PathBuf,
};

use raylib::{
    audio::RaylibAudio,
    drawing::RaylibScissorModeExt,
    ffi::KeyboardKey,
    math::{Rectangle, Vector2},
    rgui::{IntoCStr, RaylibDrawGui},
    rstr, RaylibHandle, RaylibThread,
};

use crate::{
    gui_main::{gui_highlight_end, gui_highlight_start, Action},
    song::{Playlist, SUPPORTED_FORMATS},
    GuiScreen,
};

const FILE_UP: &std::ffi::CStr = rstr!("#217#..");
const MP3_PLAYER_NAME: &std::ffi::CStr = rstr!("#11#MP3 Player");
const SELECT_THIS_FOLDER: &std::ffi::CStr = rstr!("#217# Open this Folder");
const SAVE_HERE: &std::ffi::CStr = rstr!("#218# Save here");

struct DirEntry {
    file_name: CString,
    is_file: bool,
    raw: OsString,
}

pub struct FileGuiState {
    selected: u32,
    scroll_value: f32,
    cur_path: PathBuf,
    cur_dir_entries: Vec<DirEntry>,
}

impl FileGuiState {
    pub fn default(music_path: &PathBuf, gui_screen: GuiScreen) -> std::io::Result<Self> {
        let mut me = Self {
            cur_path: music_path.clone(),
            scroll_value: 0.0,
            selected: 0,
            cur_dir_entries: vec![],
        };

        me.refresh_folder_entries(gui_screen);
        Ok(me)
    }

    fn refresh_folder_entries(&mut self, gui_screen: GuiScreen) {
        if gui_screen == GuiScreen::Player {
            return;
        }
        let folder_only = matches!(
            gui_screen,
            GuiScreen::FileSelectAddFolder
                | GuiScreen::FileSelectOpenFolder
                | GuiScreen::FileSelectSaveFile
        );

        self.cur_dir_entries.clear();
        if self.selected != 0 {
            self.selected = 1;
        }

        let entries = match fs::read_dir(&self.cur_path) {
            Ok(v) => v,
            _ => return,
        };

        for entry in entries {
            let entry = match entry {
                Ok(e) => e,
                _ => continue,
            };

            let file_type = match entry.file_type() {
                Ok(v) => v,
                _ => continue,
            };
            if !file_type.is_dir() && !file_type.is_file() {
                continue;
            }
            if !file_type.is_dir() && folder_only {
                continue;
            }

            match entry.path().extension() {
                Some(ext) => {
                    if file_type.is_file()
                        && ext != "m3u"
                        && SUPPORTED_FORMATS
                            .iter()
                            .find(|&&extension| ext == extension)
                            .is_none()
                    {
                        continue;
                    }
                }
                _ if file_type.is_file() => continue,
                _ => {}
            }

            let mut name_vec = vec![35]; // char(#)
            name_vec.push(50); // char(2)
            name_vec.push(49); // char(1)
            if file_type.is_dir() {
                name_vec.push(55); // char(7)
            } else {
                name_vec.push(56); // char(7)
            }
            name_vec.push(35); // char(#)
            let raw = entry.file_name();
            name_vec.extend(raw.as_encoded_bytes());

            if name_vec[name_vec.len() - 1] != 0 {
                name_vec.push(0);
            }
            let file_name = match CString::from_vec_with_nul(name_vec) {
                Ok(v) => v,
                _ => continue,
            };

            self.cur_dir_entries.push(DirEntry {
                file_name,
                is_file: file_type.is_file(),
                raw,
            });
        }
    }
}

fn gui_button_text_left(
    d: &mut impl RaylibDrawGui,
    mut rect: Rectangle,
    text: impl IntoCStr,
) -> bool {
    let val_1 = d.gui_button(rect, None);
    rect.x += 3.0;
    val_1 || d.gui_label_button(rect, text)
}

pub fn render_file_gui(
    rl: &mut RaylibHandle,
    playlist: &mut Playlist,
    gui_state: &mut FileGuiState,
    gui_screen: GuiScreen,
    thread: &RaylibThread,
    audio: &mut RaylibAudio,
) -> Action {
    let mut action = Action::None;
    let special_action = matches!(
        gui_screen,
        GuiScreen::FileSelectSaveFile
            | GuiScreen::FileSelectOpenFolder
            | GuiScreen::FileSelectAddFolder
    );

    let mut d = rl.begin_drawing(thread);
    if d.gui_window_box(
        Rectangle::new(
            0.0,
            0.0,
            d.get_screen_width() as f32,
            d.get_screen_height() as f32,
        ),
        Some(MP3_PLAYER_NAME),
    ) {
        action = Action::SwitchGuiScreen(GuiScreen::Player);
    }

    let height = d.get_screen_height() - 24;
    let width = d.get_screen_width();

    let buttons_height =
        (gui_state.cur_dir_entries.len() * 30 + (if special_action { 62 } else { 32 })) as i32; // 22 buttonheight + 8 padding between buttons, + 1 button for .. (+ 1 button for a special action)

    let (rect, scroll) = d.gui_scroll_panel(
        Rectangle::new(0.0, 24.0, width as f32, height as f32),
        None,
        Rectangle::new(0.0, 24.0, (width - 14) as f32, buttons_height as f32),
        Vector2::new(0.0, gui_state.scroll_value),
    );

    gui_state.scroll_value = scroll.y;

    let x = rect.x;
    let y = rect.y;
    let w = rect.width;

    let mut selected: u32 = 0;

    let button_start_y = rect.y + gui_state.scroll_value + 35.0;

    let mut d = d.begin_scissor_mode(
        x.floor() as i32,
        y.floor() as i32,
        w.floor() as i32,
        rect.height.floor() as i32,
    );

    macro_rules! file_select_button {
        ($text: expr, $i: expr) => {
            if gui_state.selected == ($i + 2) as u32 {
                gui_highlight_start();
                if gui_button_text_left(
                    &mut d,
                    Rectangle::new(
                        x + 5.0,
                        button_start_y as f32 + ($i * 30) as f32,
                        w - 10.0,
                        22.0,
                    ),
                    $text,
                ) {
                    selected = ($i + 2) as u32;
                }
                gui_highlight_end();
            } else if gui_button_text_left(
                &mut d,
                Rectangle::new(
                    x + 5.0,
                    button_start_y as f32 + ($i * 30) as f32,
                    w - 10.0,
                    22.0,
                ),
                $text,
            ) {
                selected = ($i + 2) as u32;
            }
        };
    }

    file_select_button!(Some(FILE_UP), -1);

    for (i, entry) in gui_state.cur_dir_entries.iter().enumerate() {
        if button_start_y + (i * 30) as f32 >= rect.y + rect.height {
            break;
        }
        if (button_start_y + (i * 30 + 22) as f32) < rect.y {
            continue;
        }
        file_select_button!(Some(entry.file_name.as_c_str()), i);
    }

    if special_action && gui_screen == GuiScreen::FileSelectSaveFile {
        // println!("a");
        file_select_button!(Some(SAVE_HERE), gui_state.cur_dir_entries.len());
    } else if special_action
        && matches!(
            gui_screen,
            GuiScreen::FileSelectAddFolder | GuiScreen::FileSelectOpenFolder
        )
    {
        // println!("b");
        file_select_button!(Some(SELECT_THIS_FOLDER), gui_state.cur_dir_entries.len());
    } else {
        file_select_button!(Some(SELECT_THIS_FOLDER), gui_state.cur_dir_entries.len());
    }

    if !matches!(action, Action::None) {
        return action;
    }

    if d.is_key_pressed(KeyboardKey::KEY_ESCAPE) {
        action = Action::SwitchGuiScreen(GuiScreen::Player);
    } else if d.is_key_pressed(KeyboardKey::KEY_ENTER) {
        selected = gui_state.selected;
    }
    let max_val = if special_action {
        gui_state.cur_dir_entries.len() as u32 + 1
    } else {
        gui_state.cur_dir_entries.len() as u32
    };
    if d.is_key_pressed(KeyboardKey::KEY_UP) && gui_state.selected > 0 {
        gui_state.selected -= 1;

        let offset_top = (height - 30) / 2;
        let y_coord = (gui_state.selected * 30 + 5) as i32;
        gui_state.scroll_value = -(y_coord - offset_top).max(0) as f32;
    } else if d.is_key_pressed(KeyboardKey::KEY_DOWN) && gui_state.selected <= max_val {
        gui_state.selected += 1;

        let offset_top = (height - 30) / 2;
        let y_coord = (gui_state.selected * 30 + 5) as i32;
        gui_state.scroll_value = -(y_coord - offset_top).max(0) as f32;
    }

    if selected != 0 {
        if selected == 1 {
            // navigate a folder up
            gui_state.cur_path.pop();
            gui_state.refresh_folder_entries(gui_screen);
        } else if selected - 2 < gui_state.cur_dir_entries.len() as u32 {
            let entry = &gui_state.cur_dir_entries[selected as usize - 2];
            if entry.is_file {
                // do stuff
                match gui_screen {
                    GuiScreen::FileSelectAddFile => {
                        let mut path = gui_state.cur_path.clone();
                        path.push(&entry.raw);
                        let _ = playlist.add_song_by_path(&path);
                        if !playlist.is_music_playing(audio) {
                            playlist.play_ignore_err(0, thread, audio, d.get_screen_height());
                        }
                        action = Action::SwitchGuiScreen(GuiScreen::Player);
                    }
                    GuiScreen::FileSelectOpenFile => {
                        let mut path = gui_state.cur_path.clone();
                        path.push(&entry.raw);
                        playlist.clear(audio);
                        let _ = playlist.add_song_by_path(&path);
                        if !playlist.is_music_playing(audio) {
                            playlist.play_ignore_err(0, thread, audio, d.get_screen_height());
                        }
                        action = Action::SwitchGuiScreen(GuiScreen::Player);
                    }
                    _ => {}
                }
            } else {
                gui_state.cur_path.push(&entry.raw);
                gui_state.refresh_folder_entries(gui_screen);
            }
        } else if selected == gui_state.cur_dir_entries.len() as u32 + 2 && special_action {
            match gui_screen {
                GuiScreen::FileSelectSaveFile => {
                    println!("not implemented yet")
                }
                GuiScreen::FileSelectOpenFolder => {
                    playlist.clear(audio);
                    let _ = playlist.add_song_by_path(&gui_state.cur_path);
                    if !playlist.is_music_playing(audio) {
                        playlist.play_ignore_err(0, thread, audio, d.get_screen_height());
                    }
                    action = Action::SwitchGuiScreen(GuiScreen::Player);
                }
                GuiScreen::FileSelectAddFolder => {
                    let _ = playlist.add_song_by_path(&gui_state.cur_path);
                    if !playlist.is_music_playing(audio) {
                        playlist.play_ignore_err(0, thread, audio, d.get_screen_height());
                    }
                    action = Action::SwitchGuiScreen(GuiScreen::Player);
                }
                _ => {}
            }
        }
    }

    action
}
