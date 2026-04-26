use core::mem::MaybeUninit;
use firefly_rust::*;

static mut STATE: MaybeUninit<State> = MaybeUninit::uninit();

#[derive(Clone, Copy, PartialEq)]
pub enum Kind {
    Rom,
    Data,
    Shots,
    Badges,
    Scores,
}

#[derive(Clone, Copy)]
pub struct Switch {
    pub kind: Kind,
    pub selected: bool,
}

impl Switch {
    fn new(kind: Kind) -> Self {
        Self {
            kind,
            selected: false,
        }
    }
}

pub struct State {
    pub font: FileBuf,
    pub settings: Settings,
    pub input: firefly_ui::InputManager,
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
    };
    #[allow(static_mut_refs)]
    unsafe {
        STATE.write(state)
    };
}
