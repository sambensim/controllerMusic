mod controller;
mod visuals;
mod sound;
mod chord_engine;
mod synths;
mod sound_engine;
mod display_engine;

fn main() {
    let dualshock = controller::get_dualshock().unwrap();
    let (controller_channel,  controller_channel2) = controller::start_controller_thread(dualshock);
    let samp_channel = sound::do_sound(controller_channel);
    visuals::run(samp_channel, controller_channel2);
}