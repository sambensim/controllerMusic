use std::thread::sleep;

mod controller;
mod visuals;
mod sound;
mod synths;
mod chord_engine;
mod sound_engine;
mod display_engine;
mod input_engine;
mod adsr;

fn main() {
    let input_engine = input_engine::InputEngine::init();
    let samp_channel = sound::do_sound(input_engine.subscribe());
    visuals::run(samp_channel, input_engine.subscribe(), input_engine);
}