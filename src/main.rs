mod controller;
mod visuals;
mod sound;
mod music_theory;

use raylib::prelude::*;
use std::collections::VecDeque;

fn main() {
    let dualshock = controller::get_dualshock().unwrap();
    let controller_channel = controller::start_controller_thread(dualshock);
    let samp_channel = sound::do_sound(controller_channel);
    const WIDTH : f32 = 640_f32;
    const HEIGHT : f32 = 480_f32;

    let (mut rl, thread) = raylib::init()
        .size(WIDTH as i32, HEIGHT as i32)
        .title("Hello, World")
        .build();

    const CAPACITY : usize = 10000;
    let mut draw_points : VecDeque<f32> = VecDeque::from([0.0_f32; CAPACITY]);
    

    while !rl.window_should_close() {
        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::WHITE);

        let mut samp = samp_channel.try_recv();
        while !samp.is_err() {
            draw_points.push_front(samp.unwrap());
            samp = samp_channel.try_recv();
        }
        draw_points.truncate(CAPACITY);
        let points: Vec<Vector2> = draw_points.iter().enumerate().map(|(i, samp)| Vector2 { x: WIDTH - (i as f32)/(CAPACITY as f32)*WIDTH, y: *samp*(HEIGHT/2.0) +HEIGHT/2.0}).collect();
        d.draw_line_strip(&points,Color::BLACK);

        d.draw_text("Hello, world!", 12, 12, 20, Color::BLACK);
    }
}