use raylib::drawing::RaylibDraw as _;

use crate::display_engine::DisplayEngine;

//TODO – organize into folders
mod dualshock;
mod visuals;
mod sound;
mod oscillator;
mod chord_engine;
mod sound_engine;
mod display_engine;
mod input_engine;
mod adsr;
mod controller;
mod intermediate_controller_state;
mod effect;
mod instrument;
mod voice_manager;
mod parameters;

fn main() {
    let mut input_engine = input_engine::InputEngine::init::<crate::dualshock::DS4>();
    let mut display_engine : DisplayEngine = DisplayEngine::init();
   
    display_engine.setup(input_engine.subscribe(), sound::do_sound(input_engine.subscribe()));

    let (mut rl, thread) = raylib::init()
        .size(DisplayEngine::WIDTH as i32, DisplayEngine::HEIGHT as i32)
        .title("Contoller Music")
        .build();
    
    while !rl.window_should_close() {
        let mut d = rl.begin_drawing(&thread);
        d.clear_background(raylib::ffi::Color::WHITE);
        input_engine.step();
        display_engine.handle_input();
        visuals::handle_visuals(d, &mut display_engine);
    }
}