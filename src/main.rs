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
    let mut peer_map = 0;
    for peer in state.peers.iter().rev() {
        if peer.state == PeerState::Left {
            continue;
        }
        peer_map = (peer_map << 1) | u8::from(peer.state == PeerState::Connected);
    }
    unsafe { set_peers(peer_map as i32) };
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
        if !has_connected(state) {
            quit();
            return;
        }
        transition(state, Scene::List);
    }

    let mut names = load_names();
    let my_name = names.remove(0);
    if state.my_name.is_empty() {
        state.my_name = my_name;
    }

    let updates = sync_peers(&mut state.peers, names, PeerState::Connected);
    if updates.n_joined != 0 {
        state.cursor = 0;
        state.peer = 0;
        transition(state, Scene::List);
    }
    if let Some(name) = updates.left {
        transition(state, Scene::Disconnected(name));
    }
}

fn transition(state: &mut State, scene: Scene) {
    if state.scene == scene {
        return;
    }
    if scene == Scene::List {
        if !has_connected(state) {
            transition(state, Scene::Scanning);
        }
        if state.scene != Scene::PeerActions {
            state.cursor = 0;
            state.peer = 0;
        }
    }
    state.scene = scene;
}

fn has_connected(state: &State) -> bool {
    state
        .peers
        .iter()
        .any(|peer| peer.state == PeerState::Connected)
}

struct PeerUpdates {
    left: Option<String>,
    n_joined: u8,
}

fn sync_peers(peers: &mut Vec<PeerInfo>, names: Vec<String>, new_state: PeerState) -> PeerUpdates {
    let mut updates = PeerUpdates {
        left: None,
        n_joined: 0,
    };
    for peer in peers.iter_mut() {
        if peer.state != PeerState::Connected {
            continue;
        }
        if !names.contains(&peer.name) {
            peer.state = PeerState::Left;
            updates.left = Some(peer.name.clone());
        }
    }
    for name in names {
        let maybe_peer = peers.iter_mut().find(|peer| peer.name == name);
        if let Some(peer) = maybe_peer {
            if peer.state == PeerState::Left {
                peer.state = PeerState::Connected;
            }
        } else {
            peers.push(PeerInfo {
                name,
                state: new_state,
            });
            updates.n_joined += 1;
        }
    }
    updates
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

fn update_list(state: &mut State) {
    let mut names = load_names();
    names.remove(0);
    let updates = sync_peers(&mut state.peers, names, PeerState::Hidden);
    if let Some(name) = updates.left {
        transition(state, Scene::Disconnected(name));
    }

    match state.input.get() {
        Input::Up => {
            if state.cursor > 0 {
                state.cursor -= 1;
            } else {
                state.peer = state.peer.saturating_sub(1);
            }
        }
        Input::Down => {
            if usize::from(state.peer) < state.peers.len() - 1 {
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
            state.peer = (state.peers.len() - 1) as u8;
            state.cursor = 3;
        }
        Input::Select => {
            match state.cursor {
                // select a peer
                0 => transition(state, Scene::PeerActions),
                // connect more
                1 => transition(state, Scene::Scanning),
                // confirm
                2 => quit(),
                // cancel
                3 => {
                    for peer in &mut state.peers {
                        peer.state = PeerState::Removed;
                    }
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
                let idx = usize::from(state.peer);
                if let Some(peer) = state.peers.get_mut(idx) {
                    peer.state = PeerState::Removed;
                }
                state.peer = 0;
            }
            transition(state, Scene::List);
        }
        Input::Back => {
            state.cursor = 0;
            transition(state, Scene::List);
        }
        Input::None => {}
    }
}

fn update_disconnected(state: &mut State) {
    if state.input.get() == Input::Select {
        transition(state, Scene::List);
    }
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

    if state.my_name.is_empty() {
        return;
    }
    let prefix = Message::Hello.translate(lang);

    draw_rounded_rect(
        Point::new(-4, -4),
        Size::new(
            (font.line_width_ascii(prefix) + font.line_width_ascii(&state.my_name)) as i32 + 13,
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
    draw_text(&state.my_name, font, point, theme.accent);
}

fn draw_scanning(state: &State) {
    let theme = state.settings.theme;
    let lang = state.settings.language;

    let title = Message::Scanning.translate(lang);
    let option = if has_connected(state) {
        Message::Stop
    } else {
        Message::Cancel
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
    for (peer, i) in state.peers.iter().zip(1u8..) {
        let selected = state.cursor == 0 && i - 1 == state.peer;
        if selected {
            draw_cursor(u32::from(i), theme, font, state.input.pressed(), 0);
        }

        let mut point = Point::new(20, 12 + (i as i32 + 1) * line_h);
        if selected && state.input.pressed() {
            point.x += 1;
            point.y += 1;
        }
        let color = if peer.state == PeerState::Connected {
            theme.primary
        } else {
            let w = font.line_width_ascii(&peer.name) as i32;
            draw_line(
                Point::new(point.x, point.y - 2),
                Point::new(point.x + w, point.y - 2),
                LineStyle::new(theme.secondary, 1),
            );
            theme.secondary
        };
        draw_text(&peer.name, font, point, color);
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
    let peer = &state.peers[usize::from(state.peer)];
    let options = &["disconnect peer", "back to the list"];
    firefly_ui::draw_dialog(
        theme,
        &state.font,
        &peer.name,
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
