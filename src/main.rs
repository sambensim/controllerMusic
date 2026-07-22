use hidapi::HidApi;

mod controller;
mod visuals;
mod sound;

fn main() {
    
    let dualshock = controller::get_dualshock().unwrap();
    controller::print_data(dualshock);
    // let update_visual = sound::do_sound();
    // visuals::run(update_visual);
}