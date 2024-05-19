use raylib::{
    color::Color, drawing::RaylibDraw, math::{Rectangle, Vector2}, rgui::RaylibDrawGui, rstr, text::measure_text, RaylibHandle, RaylibThread
};

use crate::{gui_main::Action, song::Playlist, GuiScreen};

#[derive(Default)]
pub struct LyricsGuiState {
    scroll: Vector2,
}

const MP3_PLAYER_NAME_LYRICS: &std::ffi::CStr = rstr!("#11#MP3 Player - Lyrics");

pub fn render_lyrics_gui(
    playlist: &mut Playlist,
    thread: &RaylibThread,
    rl: &mut RaylibHandle,
    state: &mut LyricsGuiState,
) -> Action {
    let mut d = rl.begin_drawing(thread);

    if d.gui_window_box(
        Rectangle::new(
            0.0,
            0.0,
            d.get_screen_width() as f32,
            d.get_screen_height() as f32,
        ),
        Some(MP3_PLAYER_NAME_LYRICS),
    ) || d.is_key_pressed(raylib::ffi::KeyboardKey::KEY_ESCAPE)
    {
        return Action::SwitchGuiScreen(GuiScreen::Player);
    }

    let Some(song) = playlist.currently_playing() else {
        d.draw_text(
            "No lyrics found",
            (d.get_screen_width() - measure_text("No lyrics found", 20)) / 2,
            30,
            20,
            Color::GRAY,
        );
        return Action::None;
    };

    if song.lyrics_dimensions == None {
        song.lyrics_dimensions = Some(get_dimensions(&song.lyrics, 10));
    }
    let Some((w, h)) = song.lyrics_dimensions else {
        return Action::None;
    };

    if d.is_key_pressed(raylib::ffi::KeyboardKey::KEY_UP) {
        state.scroll.y += 15.0;
    }
    if d.is_key_pressed(raylib::ffi::KeyboardKey::KEY_DOWN) {
        state.scroll.y -= 15.0;
    }
    if d.is_key_pressed(raylib::ffi::KeyboardKey::KEY_LEFT) {
        state.scroll.x += 15.0;
    }
    if d.is_key_pressed(raylib::ffi::KeyboardKey::KEY_RIGHT) {
        state.scroll.x -= 15.0;
    }
    
    {
        let (_, scroll) = d.gui_scroll_panel(
            Rectangle::new(
                0.0,
                24.0,
                d.get_screen_width() as f32,
                (d.get_screen_height() - 24) as f32,
            ),
            None,
            Rectangle::new(0.0, 24.0, w as f32, h as f32),
            state.scroll,
        );
        state.scroll = scroll;
    }

    let offset_x = 3 + state.scroll.x as i32;
    let mut offset_y = 26 + 6 + state.scroll.y as i32; // 6px padding top & bottom (thus we need 6px offset top)

    for line in song.lyrics.lines() {
        if offset_y >= 24 {
            d.draw_text(line, offset_x, offset_y, 10, Color::BLACK);
        }
        offset_y += 15; // line height + line padding
        if offset_y + 10 >= d.get_screen_height() { // text is no longer visible (beyond screen at the bottom, so we dont have to try to print further cuz no lines are gonna be in the screen again)
            break;
        }
    }

    Action::None
}

pub fn get_dimensions(text: &str, font_size: i32) -> (i32, i32) {
    let mut w = 0;
    let mut h = 12; // 6 px padding top and bottom (thus we need a 12px higher lyrics container)

    for line in text.lines() {
        w = w.max(measure_text(line, font_size));
        h += font_size;
        h += font_size / 2; // 0.5*font_size line height
    }

    (w, h)
}
