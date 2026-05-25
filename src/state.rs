use alloc::{string::String, vec::Vec};
use core::mem::MaybeUninit;
use firefly_rust::*;

static mut STATE: MaybeUninit<State> = MaybeUninit::uninit();

#[derive(PartialEq)]
pub enum Scene {
    /// Searching for more peers.
    Scanning,
    /// Scanning is done, list peers and ask what to do next.
    List,
    /// Context menu for a peer.
    PeerActions,
    Disconnected(String),
}

#[derive(Clone, Copy, PartialEq)]
pub enum PeerState {
    /// The peer was connected but then disconnected out of their own volition.
    Left,
    /// The peer was connected but then was removed by the user of this device.
    Removed,
    /// The peer is connected but hasn't been show to the user yet.
    Hidden,
    /// The peer is connected and the user saw it in the list.
    Connected,
}

pub struct PeerInfo {
    pub name: String,
    pub state: PeerState,
}

pub struct State {
    pub font: FontBuf,
    pub settings: Settings,
    pub input: firefly_ui::InputManager,
    pub scene: Scene,
    /// The currently selected peer.
    pub peer: u8,
    pub cursor: u8,
    pub my_name: String,
    pub peers: Vec<PeerInfo>,
}

pub fn get_state() -> &'static mut State {
    #[allow(static_mut_refs)]
    unsafe {
        STATE.assume_init_mut()
    }
}

pub fn load_state() {
    let settings = get_settings(get_me());
    let font = load_file_buf("ascii").unwrap();
    let state = State {
        font: font.into(),
        settings,
        input: firefly_ui::InputManager::new(),
        scene: Scene::Scanning,
        peer: 0,
        cursor: 0,
        my_name: String::new(),
        peers: Vec::new(),
    };
    #[allow(static_mut_refs)]
    unsafe {
        STATE.write(state)
    };
}
