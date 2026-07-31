mod controller;
mod visuals;
mod sound;
mod music_theory;

use raylib::prelude::*;
use std::collections::VecDeque;

fn main() {
    let dualshock = controller::get_dualshock().unwrap();
    let controller_state = controller::start_controller_thread(dualshock);
    let mut update_visual = sound::do_sound(std::sync::Arc::clone(&controller_state));
    // visuals::run(update_visual, Arc::clone(&controller_state));
    // visuals::run(update_visual);

    let (mut rl, thread) = raylib::init()
        .size(640, 480)
        .title("Hello, World")
        .build();

    while !rl.window_should_close() {
        let mut d = rl.begin_drawing(&thread);
        let temp = &mut VecDeque::new();
        update_visual(temp);
        // let mut f = update_visual;
        // (*f)(&mut model.display_buffer);
        d.clear_background(Color::WHITE);
        d.draw_text("Hello, world!", 12, 12, 20, Color::BLACK);
    }
}