use crate::dualshock::DS4;

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
mod voice;
mod voice_manager;

fn main() {
    let input_engine = input_engine::InputEngine::init::<DS4>();
    let samp_channel = sound::do_sound(input_engine.subscribe());
    visuals::run(samp_channel, input_engine.subscribe(), input_engine);
}