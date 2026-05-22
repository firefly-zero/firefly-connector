use alloc::{string::String, vec::Vec};
use core::mem::MaybeUninit;
use firefly_rust::*;

static mut STATE: MaybeUninit<State> = MaybeUninit::uninit();

#[derive(Clone, Copy, PartialEq)]
pub enum Scene {
    /// Searching for more peers.
    Scanning,
    /// Scanning is done, list peers and ask what to do next.
    List,
    /// Context menu for a peer.
    PeerActions,
}

pub struct State {
    pub font: FileBuf,
    pub settings: Settings,
    pub input: firefly_ui::InputManager,
    pub scene: Scene,
    /// The currently selected peer.
    pub peer: u8,
    pub cursor: u8,
    /// The list of names of all connected peers.
    pub peers: Vec<String>,
    pub peers_map: u32,
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
        font,
        settings,
        input: firefly_ui::InputManager::new(),
        scene: Scene::Scanning,
        peer: 0,
        cursor: 0,
        peers: Vec::new(),
        peers_map: 0,
    };
    #[allow(static_mut_refs)]
    unsafe {
        STATE.write(state)
    };
}
