#![no_std]
#![no_main]
extern crate alloc;

mod state;

use firefly_rust::*;
use firefly_ui::Input;
use state::*;

#[unsafe(no_mangle)]
extern "C" fn boot() {
    load_state();
}

#[unsafe(no_mangle)]
extern "C" fn update() {
    let state = get_state();
    state.input.update();
    match state.scene {
        Scene::Scanning => {
            match state.input.get() {
                Input::Back => quit(),
                Input::Select => {
                    // stop...
                    state.cursor = 0;
                    state.peer = 0;
                    state.scene = Scene::List;
                }
                _ => {}
            }
        }
        Scene::List => {}
        Scene::PeerActions => match state.input.get() {
            Input::Up | Input::Left => state.cursor = 0,
            Input::Down | Input::Right => state.cursor = 1,
            Input::Select => if state.cursor == 0 {},
            Input::Back | Input::None => {}
        },
    }
}

#[unsafe(no_mangle)]
extern "C" fn render() {
    let state = get_state();
    let theme = state.settings.theme;
    firefly_ui::draw_bg_grid(theme);
    draw_name(state);
    match state.scene {
        Scene::Scanning => draw_scanning(state),
        Scene::List => draw_list(state),
        Scene::PeerActions => draw_peer_actions(state),
    }
}

fn draw_name(state: &State) {
    let theme = state.settings.theme;
    let font = state.font.as_font();
    let Some(name) = state.peers.first() else {
        return;
    };
    let mut point = Point::new(5, 5 + i32::from(font.char_height()));
    let prefix = "hello, ";
    draw_text(prefix, &font, point, theme.primary);
    point.x += font.line_width_ascii(prefix) as i32;
    draw_text(name, &font, point, theme.accent);
}

fn draw_scanning(state: &State) {
    let theme = state.settings.theme;
    let font = state.font.as_font();
    let title = "scanning...";
    let options = &["stop"];
    firefly_ui::draw_dialog(
        theme,
        &font,
        title,
        options,
        state.cursor,
        state.input.pressed(),
    );
}

fn draw_list(state: &State) {
    let theme = state.settings.theme;
    let font = state.font.as_font();
    firefly_ui::draw_bg_box(theme);
    let title = "connected peers";
    firefly_ui::draw_title(title, false, &font, theme.accent);
}

fn draw_peer_actions(state: &State) {
    let theme = state.settings.theme;
    let font = state.font.as_font();
    let title = &state.peers[usize::from(state.peer)];
    let options = &["disconnect peer", "back to the list"];
    firefly_ui::draw_dialog(
        theme,
        &font,
        title,
        options,
        state.cursor,
        state.input.pressed(),
    );
}
