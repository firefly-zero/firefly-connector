#![no_std]
#![no_main]
extern crate alloc;

mod state;

use firefly_rust::*;
use state::*;

#[unsafe(no_mangle)]
extern "C" fn boot() {
    load_state();
}

#[unsafe(no_mangle)]
extern "C" fn update() {
    let state = get_state();
    state.input.update();
}

#[unsafe(no_mangle)]
extern "C" fn render() {
    let state = get_state();
    let theme = state.settings.theme;
    let font = state.font.as_font();
    firefly_ui::draw_bg(theme);
}
