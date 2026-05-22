#![no_std]
#![no_main]
extern crate alloc;

mod state;

use alloc::{string::ToString, vec::Vec};
use firefly_rust::*;
use firefly_ui::{Input, draw_cursor};
use state::*;

#[link(wasm_import_module = "misc")]
unsafe extern "C" {
    pub(crate) unsafe fn set_peers(peer_map: i32);
}

#[unsafe(no_mangle)]
extern "C" fn before_exit() {
    let state = get_state();
    unsafe { set_peers(state.peers_map as i32) };
}

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
        if state.peers_map == 0 {
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
    if names.len() > 1 && names.len() > state.peers.len() {
        state.cursor = 0;
        state.peer = 0;
        state.scene = Scene::List;
        state.peers_map = (state.peers_map << 1) | 1;
    }
    state.peers = names;
}

fn update_list(state: &mut State) {
    match state.input.get() {
        Input::Up => {
            if state.cursor > 0 {
                state.cursor -= 1;
            } else {
                state.peer = state.peer.saturating_sub(1);
            }
        }
        Input::Down => {
            if usize::from(state.peer) < state.peers.len() - 2 {
                state.peer += 1;
            } else if state.cursor < 3 {
                state.cursor += 1;
            }
        }
        Input::Left => {}
        Input::Right => {}
        Input::Select => {
            match state.cursor {
                // select a peer
                0 => state.scene = Scene::PeerActions,
                // connect more
                1 => state.scene = Scene::Scanning,
                // confirm
                2 => quit(),
                // cancel
                3 => {
                    state.peers_map = 0;
                    quit();
                }
                _ => {}
            }
        }
        Input::Back => {}
        Input::None => {}
    }
}

fn update_peer_actions(state: &mut State) {
    match state.input.get() {
        Input::Up | Input::Left => state.cursor = 0,
        Input::Down | Input::Right => state.cursor = 1,
        Input::Select => {
            if state.cursor == 0 {
                state.peers_map &= !(1u32 << state.peer);
                state.peer = 0;
                state.scene = if state.peers_map == 0 {
                    Scene::Scanning
                } else {
                    Scene::List
                };
            } else {
                state.scene = Scene::List;
            }
            state.cursor = 0;
        }
        Input::Back => {
            state.cursor = 0;
            state.scene = Scene::List;
        }
        Input::None => {}
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
    let option = if state.peers_map == 0 {
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
    let mut peers_map = state.peers_map;
    for (peer, i) in state.peers.iter().skip(1).zip(1u8..) {
        let removed = peers_map & 1 == 0;
        peers_map >>= 1;
        let selected = state.cursor == 0 && i - 1 == state.peer;
        if selected {
            draw_cursor(u32::from(i), theme, &font, state.input.pressed(), 0);
        }

        let mut point = Point::new(20, 12 + (i as i32 + 1) * line_h);
        if selected && state.input.pressed() {
            point.x += 1;
            point.y += 1;
        }
        let color = if removed {
            theme.secondary
        } else {
            theme.primary
        };
        draw_text(peer, &font, point, color);
    }

    let y = 12 + 6 * line_h + 2;
    draw_line(
        Point::new(12, y),
        Point::new(WIDTH - 12, y),
        LineStyle::new(theme.primary, 1),
    );

    if state.cursor == 1 {
        draw_cursor(6, theme, &font, state.input.pressed(), 0);
    }
    let point = Point::new(20, 12 + 7 * line_h);
    draw_text("connect more peers", &font, point, theme.primary);

    if state.cursor == 2 {
        draw_cursor(7, theme, &font, state.input.pressed(), 0);
    }
    let point = Point::new(20, 12 + 8 * line_h);
    draw_text("confirm", &font, point, theme.primary);

    if state.cursor == 3 {
        draw_cursor(8, theme, &font, state.input.pressed(), 0);
    }
    let point = Point::new(20, 12 + 9 * line_h);
    draw_text("cancel", &font, point, theme.primary);
}

fn draw_peer_actions(state: &State) {
    let theme = state.settings.theme;
    let font = state.font.as_font();
    let title = &state.peers[usize::from(state.peer) + 1];
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
