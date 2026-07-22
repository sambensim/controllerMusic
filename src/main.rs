mod controller;
mod visuals;
mod sound;
mod music_theory;

fn main() {
    
    let dualshock = controller::get_dualshock().unwrap();
    let controller_state = controller::start_controller_thread(dualshock);
    let update_visual = sound::do_sound(std::sync::Arc::clone(&controller_state));
    // visuals::run(update_visual, Arc::clone(&controller_state));
    visuals::run(update_visual);
}