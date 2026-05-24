#![no_std]
#![no_main]
extern crate alloc;

mod state;
mod translations;

use alloc::{
    string::{String, ToString},
    vec::Vec,
};
use firefly_rust::*;
use firefly_ui::{Input, Translate, draw_cursor};
use state::*;
use translations::*;

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
        Scene::Disconnected(_) => update_disconnected(state),
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

    let names = load_names();
    if names != state.peers {
        if peers_added(&state.peers, &names) {
            state.cursor = 0;
            state.peer = 0;
            state.scene = Scene::List;
            state.peers_map = (state.peers_map << 1) | 1;
        }
        if peers_removed(&state.peers, &names) {
            state.cursor = 0;
            state.peer = 0;
            state.scene = Scene::Disconnected("???".to_string());
            state.peers_map >>= state.peers.len().saturating_sub(names.len()).max(1);
        }
        state.peers = names;
    }
}

fn load_names() -> Vec<String> {
    let peers = get_peers();
    let mut names = Vec::new();
    for peer in peers {
        let mut name = get_name_buf(peer);
        if name.is_empty() {
            name = "<empty>".to_string();
        }
        names.push(name);
    }
    names
}

fn peers_added(old: &[String], new: &[String]) -> bool {
    if old.is_empty() {
        return false;
    }
    if new.len() > old.len() {
        return true;
    }
    new.iter().any(|name| !old.contains(name))
}

fn peers_removed(old: &[String], new: &[String]) -> bool {
    if new.len() < old.len() {
        return true;
    }
    old.iter().any(|name| !new.contains(name))
}

fn update_list(state: &mut State) {
    let names = load_names();
    if names != state.peers {
        if peers_removed(&state.peers, &names) {
            state.cursor = 0;
            state.peer = 0;
            state.scene = Scene::Disconnected("???".to_string());
        }
        state.peers = names;
        return;
    }
    if state.peers_map == 0 {
        state.scene = Scene::Scanning;
    };

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
        Input::Left => {
            state.peer = 0;
            state.cursor = 0;
        }
        Input::Right => {
            state.peer = (state.peers.len() - 2) as u8;
            state.cursor = 3;
        }
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
            }
            state.cursor = 0;
            state.scene = Scene::List;
        }
        Input::Back => {
            state.cursor = 0;
            state.scene = Scene::List;
        }
        Input::None => {}
    }
}

fn update_disconnected(state: &mut State) {
    if state.input.get() != Input::Select {
        return;
    }
    reset_peers_map(state);
    state.scene = Scene::List;
}

fn reset_peers_map(state: &mut State) {
    state.peers_map = 0;
    for _ in 0..state.peers.len() - 1 {
        state.peers_map = (state.peers_map << 1) | 1;
    }
    state.peer = 0;
}

#[unsafe(no_mangle)]
extern "C" fn render() {
    let state = get_state();
    let theme = state.settings.theme;
    firefly_ui::draw_bg_grid(theme);
    match &state.scene {
        Scene::Scanning => draw_scanning(state),
        Scene::List => draw_list(state),
        Scene::PeerActions => draw_peer_actions(state),
        Scene::Disconnected(name) => draw_disconnected(state, name),
    }
    draw_name(state);
}

fn draw_name(state: &State) {
    let theme = state.settings.theme;
    let lang = state.settings.language;
    let font = &state.font;

    let Some(name) = state.peers.first() else {
        return;
    };
    let prefix = Message::Hello.translate(lang);

    draw_rounded_rect(
        Point::new(-4, -4),
        Size::new(
            (font.line_width_ascii(prefix) + font.line_width_ascii(name)) as i32 + 13,
            i32::from(font.char_height()) + 10,
        ),
        Size::new(4, 4),
        Style {
            fill_color: theme.bg,
            stroke_color: theme.primary,
            stroke_width: 1,
        },
    );

    let mut point = Point::new(5, i32::from(font.char_height()) - 1);
    draw_text(prefix, font, point, theme.primary);
    point.x += font.line_width_ascii(prefix) as i32;
    draw_text(name, font, point, theme.accent);
}

fn draw_scanning(state: &State) {
    let theme = state.settings.theme;
    let lang = state.settings.language;

    let title = Message::Scanning.translate(lang);
    let option = if state.peers_map == 0 {
        Message::Cancel
    } else {
        Message::Stop
    };
    let option = option.translate(lang);
    firefly_ui::draw_dialog(
        theme,
        &state.font,
        title,
        &[option],
        state.cursor,
        state.input.pressed(),
    );
}

fn draw_list(state: &State) {
    let theme = state.settings.theme;
    let font = &state.font;
    let lang = state.settings.language;

    firefly_ui::draw_bg_box(theme);
    let title = Message::ConnectedPeers.translate(lang);
    firefly_ui::draw_title(title, false, font, theme.accent);

    let line_h = font.char_height() as i32 + 4;
    let mut peers_map = state.peers_map;
    for (peer, i) in state.peers.iter().skip(1).zip(1u8..) {
        let removed = peers_map & 1 == 0;
        peers_map >>= 1;
        let selected = state.cursor == 0 && i - 1 == state.peer;
        if selected {
            draw_cursor(u32::from(i), theme, font, state.input.pressed(), 0);
        }

        let mut point = Point::new(20, 12 + (i as i32 + 1) * line_h);
        if selected && state.input.pressed() {
            point.x += 1;
            point.y += 1;
        }
        let color = if removed {
            let w = font.line_width_ascii(peer) as i32;
            draw_line(
                Point::new(point.x, point.y - 2),
                Point::new(point.x + w, point.y - 2),
                LineStyle::new(theme.secondary, 1),
            );
            theme.secondary
        } else {
            theme.primary
        };
        draw_text(peer, font, point, color);
    }

    let y = 12 + 6 * line_h + 2;
    draw_line(
        Point::new(12, y),
        Point::new(WIDTH - 12, y),
        LineStyle::new(theme.primary, 1),
    );

    let msgs = &[Message::ConnectMorePeers, Message::Confirm, Message::Cancel];
    for (msg, i) in msgs.iter().zip(1u8..) {
        if state.cursor == i {
            draw_cursor(5 + u32::from(i), theme, font, state.input.pressed(), 0);
        }
        let msg = msg.translate(lang);
        let point = Point::new(20, 12 + i32::from(6 + i) * line_h);
        draw_text(msg, font, point, theme.primary);
    }
}

fn draw_peer_actions(state: &State) {
    let theme = state.settings.theme;
    let title = &state.peers[usize::from(state.peer) + 1];
    let options = &["disconnect peer", "back to the list"];
    firefly_ui::draw_dialog(
        theme,
        &state.font,
        title,
        options,
        state.cursor,
        state.input.pressed(),
    );
}

fn draw_disconnected(state: &State, name: &str) {
    let theme = state.settings.theme;
    let prompt = alloc::format!("{name} disconnected");
    firefly_ui::draw_dialog(
        theme,
        &state.font,
        &prompt,
        &["ok"],
        0,
        state.input.pressed(),
    );
}
