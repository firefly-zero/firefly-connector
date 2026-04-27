#![no_std]
#![no_main]
extern crate alloc;

mod state;

use alloc::{
    string::{String, ToString},
    vec::Vec,
};
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
        Scene::Scanning => update_scanning(state),
        Scene::List => update_list(state),
        Scene::PeerActions => update_peer_actions(state),
    }
}

fn update_scanning(state: &mut State) {
    if matches!(state.input.get(), Input::Back | Input::Select) {
        if state.peers.len() <= 1 {
            quit();
            return;
        }
        state.cursor = 0;
        state.peer = 0;
        state.scene = Scene::List;
    }

    let peers = get_peers();
    let mut names = Vec::new();
    for peer in peers {
        let mut name = get_name_buf(peer);
        if name.is_empty() {
            name = "<empty>".to_string();
        }
        names.push(name);
    }
    if names.len() > 1 && count_full_peers(&state.peers) != count_full_peers(&names) {
        state.cursor = 0;
        state.peer = 0;
        state.scene = Scene::List;
    }
    state.peers = names;
}

fn count_full_peers(names: &[String]) -> usize {
    let mut cnt = 0;
    for name in names {
        if !name.contains('?') {
            cnt += 1;
        }
    }
    cnt
}

fn update_list(state: &mut State) {
    // ...
}

fn update_peer_actions(state: &mut State) {
    match state.input.get() {
        Input::Up | Input::Left => state.cursor = 0,
        Input::Down | Input::Right => state.cursor = 1,
        Input::Select => if state.cursor == 0 {},
        Input::Back | Input::None => {}
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
    let mut point = Point::new(5, i32::from(font.char_height()) - 1);
    let prefix = "hello, ";
    draw_text(prefix, &font, point, theme.primary);
    point.x += font.line_width_ascii(prefix) as i32;
    draw_text(name, &font, point, theme.accent);
}

fn draw_scanning(state: &State) {
    let theme = state.settings.theme;
    let font = state.font.as_font();
    let title = "scanning...";
    let option = if state.peers.len() <= 1 {
        "cancel"
    } else {
        "stop"
    };
    firefly_ui::draw_dialog(
        theme,
        &font,
        title,
        &[option],
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

    let line_h = font.char_height() as i32 + 4;
    for (peer, i) in state.peers.iter().skip(1).zip(2..) {
        let mut point = Point::new(20, 12 + i * line_h);
        if i - 1 == i32::from(state.cursor) && state.input.pressed() {
            point.x += 1;
            point.y += 1;
        }
        draw_text(peer, &font, point, theme.primary);
    }
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
