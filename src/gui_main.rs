use raylib::{
    audio::RaylibAudio,
    color::Color,
    drawing::{RaylibDraw, RaylibDrawHandle, RaylibScissorModeExt},
    ffi::{GuiControl, GuiControlProperty, KeyboardKey},
    math::{Rectangle, Vector2},
    rgui::RaylibDrawGui,
    rstr, RaylibHandle, RaylibThread,
};

use crate::{
    song::{Playlist, RepeatBehavior},
    GuiScreen,
};

pub enum Action {
    None,
    ExitProgram,
    SwitchGuiScreen(GuiScreen),
}

pub const ICON_PREV: &std::ffi::CStr = rstr!("#129#");
pub const ICON_NEXT: &std::ffi::CStr = rstr!("#134#");
pub const ICON_PLAY: &std::ffi::CStr = rstr!("#131#");
pub const ICON_SHUFFLE: &std::ffi::CStr = rstr!("#78#");
pub const ICON_PAUSE: &std::ffi::CStr = rstr!("#132#");
pub const ICON_PLAYER_STOP: &std::ffi::CStr = rstr!("#133#");
pub const ICON_VOL: &std::ffi::CStr = rstr!("#122#");
pub const ICON_VOL_MUTE: &std::ffi::CStr = rstr!("#220#");
pub const ICON_FOLDER_OPEN: &std::ffi::CStr = rstr!("#003#");
pub const ICON_FILE_OPEN: &std::ffi::CStr = rstr!("#005#");
pub const ICON_FILE_SAVE: &std::ffi::CStr = rstr!("#006#");
pub const ICON_FILE_ADD: &std::ffi::CStr = rstr!("#008#");
pub const ICON_FOLDER_ADD: &std::ffi::CStr = rstr!("#221#");
pub const ICON_FILE_CLOSE: &std::ffi::CStr = rstr!("#009#");

pub fn gui_get_style_color(control: GuiControl, property: GuiControlProperty) -> Color {
    unsafe {
        Color::get_color(u32::from_le_bytes(
            raylib::ffi::GuiGetStyle(control as i32, property as i32).to_le_bytes(),
        ))
    }
}
// pub fn gui_get_style(control: GuiControl, property: GuiControlProperty) -> i32 {
//     unsafe { raylib::ffi::GuiGetStyle(control as i32, property as i32) }
// }

#[derive(Default)]
pub struct MainGuiState {
    current_x: u32,
    current_y: u32,
    currently_unselected: bool,
}

macro_rules! window_bar_button {
    ($id: expr, $icon: expr, $gui_state: expr, $d: expr) => {{
        let should_highlight = $gui_state.current_y == 1 && $gui_state.current_x == $id as u32;
        if should_highlight {
            gui_highlight_start()
        }
        let val = $d.gui_button(
            Rectangle::new(3.0 + 20.0 * ($id as f32), 3.0, 18.0, 18.0),
            Some($icon),
        );
        if should_highlight {
            gui_highlight_end();
            $d.gui_set_style(
                GuiControl::BUTTON,
                GuiControlProperty::BORDER_WIDTH as i32,
                1,
            );
        }
        val || (should_highlight && $d.is_key_released(KeyboardKey::KEY_ENTER))
    }};
}

macro_rules! music_control_button {
    ($id: expr, $icon: expr, $gui_state: expr, $d: expr, $soundcontrol_start_x: expr, $soundcontrol_y: expr) => {{
        let should_highlight = $gui_state.current_y == 4 && $gui_state.current_x == $id as u32;
        if should_highlight {
            gui_highlight_start()
        }
        let val = $d.gui_button(
            Rectangle::new(
                $soundcontrol_start_x + ($id * 38) as f32,
                $soundcontrol_y,
                28.0,
                28.0,
            ),
            Some($icon),
        );
        if should_highlight {
            gui_highlight_end();
        }
        val || (should_highlight && $d.is_key_released(KeyboardKey::KEY_ENTER))
    }};
}

pub fn update_music(
    audio: &mut RaylibAudio,
    playlist: &mut Playlist,
    thread: &RaylibThread,
    rl: &mut RaylibHandle,
    main_state: &mut MainGuiState,
) {
    if rl.is_key_pressed(KeyboardKey::KEY_SPACE) {
        // play-pause
        playlist.pause_resume(audio);
    }
    if rl.is_key_pressed(KeyboardKey::KEY_N) {
        if rl.is_key_down(KeyboardKey::KEY_LEFT_SHIFT)
            || rl.is_key_down(KeyboardKey::KEY_RIGHT_SHIFT)
        {
            // prev
            if let Some(idx) = playlist.currently_playing_id() {
                if idx == 0 {
                    playlist.play_ignore_err(idx + 1, &thread, audio, rl.get_screen_height());
                } else {
                    if idx < playlist.len() {
                        playlist.play_ignore_err(idx - 1, thread, audio, rl.get_screen_height());
                    } else if playlist.len() > 0 {
                        playlist.play_ignore_err(
                            playlist.len() - 1,
                            thread,
                            audio,
                            rl.get_screen_height(),
                        );
                    }
                }
            } else {
                playlist.play_ignore_err(0, &thread, audio, rl.get_screen_height());
            }
        } else {
            // next
            if let Some(idx) = playlist.currently_playing_id() {
                if idx + 1 < playlist.len() {
                    playlist.play_ignore_err(idx + 1, &thread, audio, rl.get_screen_height());
                } else {
                    playlist.play_ignore_err(0, &thread, audio, rl.get_screen_height());
                }
            } else {
                playlist.play_ignore_err(0, &thread, audio, rl.get_screen_height());
            }
        }
    }
    if rl.is_key_pressed(KeyboardKey::KEY_R) {
        if rl.is_key_down(KeyboardKey::KEY_LEFT_CONTROL)
            || rl.is_key_down(KeyboardKey::KEY_RIGHT_CONTROL)
        {
            // shuffle playlist
            playlist.shuffle();
        } else {
            // repeat next
            playlist.repeat_behavior.next();
        }
    }
    if rl.is_key_pressed(KeyboardKey::KEY_M) {
        if rl.is_key_down(KeyboardKey::KEY_LEFT_CONTROL)
            || rl.is_key_down(KeyboardKey::KEY_RIGHT_CONTROL)
        {
            // mute/unmute
            if unsafe { raylib::ffi::GetMasterVolume() } != 0.0 {
                audio.set_master_volume(0.0);
            } else {
                audio.set_master_volume(1.0);
            }
        } else {
            // menu buttons (top)
            main_state.current_y = 1;
        }
    }

    if let Some(idx) = playlist.currently_playing_id() {
        if playlist.music_has_reached_the_end(&audio) {
            match playlist.repeat_behavior {
                RepeatBehavior::Normal => {
                    if idx + 1 < playlist.len() {
                        playlist.play_ignore_err(idx + 1, &thread, audio, rl.get_screen_height());
                    } else {
                        playlist.stop_playing(audio);
                    }
                }
                RepeatBehavior::RepeatSingle => playlist.seek(0.0, audio),
                RepeatBehavior::Repeat => {
                    if idx + 1 < playlist.len() {
                        playlist.play_ignore_err(idx + 1, &thread, audio, rl.get_screen_height());
                    } else if playlist.len() > 0 {
                        playlist.play_ignore_err(0, &thread, audio, rl.get_screen_height());
                    } else {
                        playlist.stop_playing(audio);
                    }
                }
            }
            // play next song
        }
        playlist.update(audio);
    }
}

pub fn render_main_gui(
    audio: &mut RaylibAudio,
    playlist: &mut Playlist,
    thread: &RaylibThread,
    rl: &mut RaylibHandle,
    gui_state: &mut MainGuiState,
) -> Action {
    let mut action: Action = Action::None;

    if !gui_state.currently_unselected {
        if rl.is_key_pressed(KeyboardKey::KEY_DOWN) && gui_state.current_y < 5 {
            gui_state.current_y += 1;
            gui_state.current_x = 0;
            if gui_state.current_y == 4 {
                gui_state.current_x = 2;
            }
        }
        if rl.is_key_pressed(KeyboardKey::KEY_UP) && gui_state.current_y > 0 {
            gui_state.current_y -= 1;
            gui_state.current_x = 0;
            if gui_state.current_y == 4 {
                gui_state.current_x = 1;
            }
        }
        if gui_state.current_y == 1 {
            // the 7 top bar buttons
            if rl.is_key_pressed(KeyboardKey::KEY_RIGHT) && gui_state.current_x < 6 {
                gui_state.current_x += 1;
            }
            if rl.is_key_pressed(KeyboardKey::KEY_LEFT) && gui_state.current_x > 0 {
                gui_state.current_x -= 1;
            }
        }
        if gui_state.current_y == 4 {
            // the 7 top bar buttons
            if rl.is_key_pressed(KeyboardKey::KEY_RIGHT) && gui_state.current_x < 4 {
                gui_state.current_x += 1;
            }
            if rl.is_key_pressed(KeyboardKey::KEY_LEFT) && gui_state.current_x > 0 {
                gui_state.current_x -= 1;
            }
        }
        if rl.is_key_released(KeyboardKey::KEY_ENTER) && gui_state.current_y == 2 {
            gui_state.currently_unselected = true;
            playlist.init_select(rl.get_screen_height());
        }
    }
    if rl.is_key_released(KeyboardKey::KEY_ESCAPE) {
        if gui_state.currently_unselected {
            gui_state.currently_unselected = false;
        } else {
            gui_state.current_y = 0;
        }
    }

    // progress bar
    if gui_state.current_y == 3 || gui_state.current_y == 0 {
        let cur_prog = playlist.music_length_played(audio);
        let max_prog = playlist.music_length_total(audio) - 6.0;

        if rl.is_key_pressed(KeyboardKey::KEY_RIGHT) {
            if cur_prog >= max_prog {
                playlist.pause(audio);
                if let Some(idx) = playlist.currently_playing_id() {
                    playlist.play_ignore_err(idx + 1, thread, audio, rl.get_screen_height());
                }
            } else {
                playlist.seek(cur_prog + 5.0, audio);
            }
        } else if rl.is_key_pressed(KeyboardKey::KEY_LEFT) {
            playlist.seek((cur_prog - 5.0).max(1.0), audio);
        }
    }

    // volume bar
    if gui_state.current_y == 5 {
        let volume = unsafe { raylib::ffi::GetMasterVolume() };

        if rl.is_key_pressed(KeyboardKey::KEY_RIGHT) {
            audio.set_master_volume((volume + 0.05).min(1.0))
        } else if rl.is_key_pressed(KeyboardKey::KEY_LEFT) {
            audio.set_master_volume((volume - 0.05).max(0.0))
        }
    }

    let mut d = rl.begin_drawing(&thread);

    if gui_state.current_y == 1 && gui_state.current_x == 6 {
        gui_highlight_start_single_control(GuiControl::BUTTON);
    }

    if d.gui_window_box(
        Rectangle::new(
            0.0,
            0.0,
            d.get_screen_width() as f32,
            d.get_screen_height() as f32,
        ),
        None,
    ) || (gui_state.current_y == 1
        && gui_state.current_x == 6
        && d.is_key_pressed(KeyboardKey::KEY_ENTER))
    {
        return Action::ExitProgram;
    }

    gui_highlight_end();

    let border_width = d.gui_get_style(GuiControl::BUTTON, GuiControlProperty::BORDER_WIDTH as i32);
    d.gui_set_style(
        GuiControl::BUTTON,
        GuiControlProperty::BORDER_WIDTH as i32,
        1,
    );

    if window_bar_button!(0, ICON_FILE_ADD, gui_state, d) {
        action = Action::SwitchGuiScreen(GuiScreen::FileSelectAddFile);
    }
    if window_bar_button!(1, ICON_FOLDER_ADD, gui_state, d) {
        action = Action::SwitchGuiScreen(GuiScreen::FileSelectAddFolder);
    }
    if window_bar_button!(2, ICON_FILE_OPEN, gui_state, d) {
        action = Action::SwitchGuiScreen(GuiScreen::FileSelectOpenFile);
    }
    if window_bar_button!(3, ICON_FOLDER_OPEN, gui_state, d) {
        action = Action::SwitchGuiScreen(GuiScreen::FileSelectOpenFolder);
    }
    if window_bar_button!(4, ICON_FILE_SAVE, gui_state, d) {
        action = Action::SwitchGuiScreen(GuiScreen::FileSelectSaveFile);
    }
    if window_bar_button!(5, ICON_FILE_CLOSE, gui_state, d) {
        playlist.clear(audio);
    }

    d.gui_set_style(
        GuiControl::BUTTON,
        GuiControlProperty::BORDER_WIDTH as i32,
        border_width,
    );

    let progress = playlist.progress(&audio);

    let soundcontrol_start_x = (d.get_screen_width() / 2 - 90) as f32;
    let soundcontrol_y = (d.get_screen_height() - 80) as f32;

    if gui_state.current_y == 3 {
        gui_highlight_start();
    }
    let new_progress = d.gui_slider_bar(
        Rectangle::new(
            10.0,
            soundcontrol_y - 20.0,
            (d.get_screen_width() - 20) as f32,
            10.0,
        ),
        None,
        None,
        progress,
        0.0,
        1.0,
    );
    if gui_state.current_y == 3 {
        gui_highlight_end();
    }
    if progress != new_progress {
        playlist.seek(new_progress * playlist.music_length_total(&audio), audio);
    }

    if gui_state.current_y == 5 {
        gui_highlight_start();
    }
    let volume = unsafe { raylib::ffi::GetMasterVolume() };
    let new_volume = d.gui_slider_bar(
        Rectangle::new(
            27.0,
            soundcontrol_y + 38.0,
            (d.get_screen_width() - 37) as f32,
            10.0,
        ),
        Some(if volume != 0.0 {
            ICON_VOL
        } else {
            ICON_VOL_MUTE
        }),
        None,
        volume,
        0.0,
        1.0,
    );
    if gui_state.current_y == 5 {
        gui_highlight_end();
    }

    if d.gui_label_button(
        Rectangle::new(10.0, soundcontrol_y + 28.0, 24.0, 24.0),
        None,
    ) {
        if unsafe { raylib::ffi::GetMasterVolume() } != 0.0 {
            audio.set_master_volume(0.0);
        } else {
            audio.set_master_volume(1.0);
        }
    }

    if new_volume != volume {
        audio.set_master_volume(new_volume);
    }

    if music_control_button!(
        0,
        ICON_SHUFFLE,
        gui_state,
        d,
        soundcontrol_start_x,
        soundcontrol_y
    ) {
        playlist.shuffle();
    }
    if music_control_button!(
        1,
        ICON_PREV,
        gui_state,
        d,
        soundcontrol_start_x,
        soundcontrol_y
    ) {
        let idx = playlist.currently_playing_id().unwrap_or(0);

        if idx == 0 {
            playlist.play_ignore_err(0, &thread, audio, d.get_screen_height());
        } else {
            let idx = if idx - 1 < playlist.len() {
                idx - 1
            } else if playlist.len() > 0 {
                playlist.len() - 1
            } else {
                0
            };
            playlist.play_ignore_err(idx, &thread, audio, d.get_screen_height());
        }
    }
    if music_control_button!(
        3,
        ICON_NEXT,
        gui_state,
        d,
        soundcontrol_start_x,
        soundcontrol_y
    ) {
        if let Some(idx) = playlist.currently_playing_id() {
            if idx + 1 < playlist.len() {
                playlist.play_ignore_err(idx + 1, &thread, audio, d.get_screen_height());
            } else {
                playlist.play_ignore_err(0, &thread, audio, d.get_screen_height());
            }
        } else {
            playlist.play_ignore_err(0, &thread, audio, d.get_screen_height());
        }
    }
    if music_control_button!(
        4,
        playlist.repeat_behavior.to_icon(),
        gui_state,
        d,
        soundcontrol_start_x,
        soundcontrol_y
    ) {
        playlist.repeat_behavior.next();
    }

    if playlist.has_music_stream() {
        if music_control_button!(
            2,
            if playlist.is_music_playing(&audio) {
                ICON_PAUSE
            } else {
                ICON_PLAY
            },
            gui_state,
            d,
            soundcontrol_start_x,
            soundcontrol_y
        ) {
            playlist.pause_resume(audio);
        }
    } else {
        if music_control_button!(
            2,
            ICON_PLAYER_STOP,
            gui_state,
            d,
            soundcontrol_start_x,
            soundcontrol_y
        ) {
            playlist.play_ignore_err(0, &thread, audio, d.get_screen_height());
        }
    }

    let text = if let Some(filename) = playlist.filename_vec() {
        unsafe { std::mem::transmute::<&[u8], &str>(&filename[0..filename.len() - 1]) }
    } else {
        "Not Playing"
    };
    d.draw_text(
        text,
        10,
        soundcontrol_y as i32 - 50,
        20,
        Color::get_color(u32::from_be_bytes(
            d.gui_get_style(GuiControl::DEFAULT, 2 /* TEXT_COLOR_NORMAL */)
                .to_be_bytes(),
        )),
    );

    playlist.render(
        &mut d,
        audio,
        thread,
        gui_state.current_y == 2 && gui_state.currently_unselected,
        gui_state.current_y == 2,
    );

    return action;
}

impl Playlist {
    fn render(
        &mut self,
        d: &mut RaylibDrawHandle,
        audio: &mut RaylibAudio,
        thread: &RaylibThread,
        is_focused: bool,
        is_selected: bool,
    ) {
        if is_focused && self.len() > 0 {
            if d.is_key_pressed(KeyboardKey::KEY_ENTER) {
                self.play_ignore_err(
                    self.__render_current_selected,
                    thread,
                    audio,
                    d.get_screen_height(),
                );
            }
            if d.is_key_pressed(KeyboardKey::KEY_UP) && self.__render_current_selected > 0 {
                self.__render_current_selected -= 1;
                self.adjust_center_song(self.__render_current_selected, d.get_screen_height());
            }
            if d.is_key_pressed(KeyboardKey::KEY_DOWN)
                && self.__render_current_selected < self.len() - 1
            {
                self.__render_current_selected += 1;
                self.adjust_center_song(self.__render_current_selected, d.get_screen_height());
            }
        }

        let width = d.get_screen_width() - 20;
        let height = d.get_screen_height() - 180;
        let currently_playing_id = self.currently_playing_id().unwrap_or(self.len());

        let buttons_height = (self.len() * 30 + 2) as i32; // 22 buttonheight + 8 padding between buttons

        let (rect, scroll) = d.gui_scroll_panel(
            Rectangle::new(10.0, 40.0, width as f32, height as f32),
            None,
            Rectangle::new(10.0, 40.0, (width - 14) as f32, buttons_height as f32),
            Vector2::new(0.0, self.__render_scroll_index),
        );

        if is_focused || is_selected {
            let col = gui_get_style_color(
                GuiControl::DEFAULT,
                GuiControlProperty::BORDER_COLOR_FOCUSED,
            );
            d.draw_rectangle_lines(10, 40, width, height, col);
        }

        self.__render_scroll_index = scroll.y;

        let x = rect.x;
        let y = rect.y;
        let w = rect.width;

        let button_start_y = rect.y + self.__render_scroll_index + 5.0;

        let mut d = d.begin_scissor_mode(
            x.floor() as i32,
            y.floor() as i32,
            w.floor() as i32,
            rect.height.floor() as i32,
        );

        for i in 0..self.len() {
            let path = &self.get_songs()[i];
            if button_start_y + (i * 30) as f32 >= rect.y + rect.height {
                break;
            }
            if (button_start_y + (i * 30 + 22) as f32) < rect.y {
                continue;
            }
            let val = if i == currently_playing_id
                || (is_focused && i == self.__render_current_selected)
            {
                gui_highlight_start();
                let val = d.gui_button(
                    Rectangle::new(x + 5.0, button_start_y + (i * 30) as f32, w - 10.0, 22.0),
                    Some(path.file_name()),
                );
                gui_highlight_end();
                val
            } else {
                d.gui_button(
                    Rectangle::new(x + 5.0, button_start_y + (i * 30) as f32, w - 10.0, 22.0),
                    Some(path.file_name()),
                )
            };

            if val && rect.check_collision_point_rec(d.get_mouse_position()) {
                self.play_ignore_err(i, thread, audio, d.get_screen_height());
            }
        }
    }

    fn init_select(&mut self, screen_height: i32) {
        if let Some(id) = self.currently_playing_id() {
            self.__render_current_selected = id;
            self.adjust_center_song(id, screen_height);
        } else {
            self.__render_current_selected = 0;
            self.__render_scroll_index = 0.0;
        }
    }
}

pub fn gui_highlight_start() {
    unsafe {
        for i in 0..16 {
            raylib::ffi::GuiSetStyle(
                i,
                GuiControlProperty::TEXT_COLOR_NORMAL as i32,
                raylib::ffi::GuiGetStyle(i, GuiControlProperty::TEXT_COLOR_FOCUSED as i32),
            );
            raylib::ffi::GuiSetStyle(
                i,
                GuiControlProperty::BASE_COLOR_NORMAL as i32,
                raylib::ffi::GuiGetStyle(i, GuiControlProperty::BASE_COLOR_FOCUSED as i32),
            );
            raylib::ffi::GuiSetStyle(
                i,
                GuiControlProperty::BORDER_COLOR_NORMAL as i32,
                raylib::ffi::GuiGetStyle(i, GuiControlProperty::BORDER_COLOR_FOCUSED as i32),
            );
        }
    }
}

pub fn gui_highlight_start_single_control(control: GuiControl) {
    unsafe {
        raylib::ffi::GuiSetStyle(
            control as i32,
            GuiControlProperty::TEXT_COLOR_NORMAL as i32,
            raylib::ffi::GuiGetStyle(
                control as i32,
                GuiControlProperty::TEXT_COLOR_FOCUSED as i32,
            ),
        );
        raylib::ffi::GuiSetStyle(
            control as i32,
            GuiControlProperty::BASE_COLOR_NORMAL as i32,
            raylib::ffi::GuiGetStyle(
                control as i32,
                GuiControlProperty::BASE_COLOR_FOCUSED as i32,
            ),
        );
        raylib::ffi::GuiSetStyle(
            control as i32,
            GuiControlProperty::BORDER_COLOR_NORMAL as i32,
            raylib::ffi::GuiGetStyle(
                control as i32,
                GuiControlProperty::BORDER_COLOR_FOCUSED as i32,
            ),
        );
    }
}

pub fn gui_highlight_end() {
    unsafe {
        raylib::ffi::GuiLoadStyleDefault();
    }
}
