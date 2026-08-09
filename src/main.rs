use crate::dualshock::DS4;

mod dualshock;
mod visuals;
mod sound;
mod synths;
mod chord_engine;
mod sound_engine;
mod display_engine;
mod input_engine;
mod adsr;
mod controller_trait;
mod intermediate_controller_state;

fn main() {
    let input_engine = input_engine::InputEngine::init::<DS4>();
    let samp_channel = sound::do_sound(input_engine.subscribe());
    visuals::run(samp_channel, input_engine.subscribe(), input_engine);
}