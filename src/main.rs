mod controller;
mod visuals;
mod sound;
mod music_theory;

use raylib::prelude::*;
// use std::collections::VecDeque;

fn main() {
    let dualshock = controller::get_dualshock().unwrap();
    let controller_channel = controller::start_controller_thread(dualshock);
    let _ = sound::do_sound(controller_channel);

    let (mut rl, thread) = raylib::init()
        .size(640, 480)
        .title("Hello, World")
        .build();

    while !rl.window_should_close() {
        let mut d = rl.begin_drawing(&thread);
        // let points = &mut VecDeque::new();
        // update_visual(points);
        d.clear_background(Color::WHITE);
        d.draw_text("Hello, world!", 12, 12, 20, Color::BLACK);
    }
}