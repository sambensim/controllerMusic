use hidapi::HidApi;

mod controller;
mod visuals;
mod sound;

fn main() {
    
    _ = controller::get_dualshock();
    // let update_visual = sound::do_sound();
    // visuals::run(update_visual);
}