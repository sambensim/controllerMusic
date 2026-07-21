use std::{collections::VecDeque, thread::sleep};

use crate::sound::do_sound;

mod controller;
mod visuals;
mod sound;

fn main() {
    // use gilrs::{Gilrs, Button, Event};

    // let mut gilrs = Gilrs::new().unwrap();
    // sleep(std::time::Duration::from_millis(1000));

    // // Iterate over all connected gamepads
    // for (_id, gamepad) in gilrs.gamepads() {
    //     println!("{} is {:?}", gamepad.name(), gamepad.power_info());
    // }

    // let mut active_gamepad = None;

    // loop {
    //     // Examine new events
    //     while let Some(Event { id, event, time, .. }) = gilrs.next_event() {
    //         println!("{:?} New event from {}: {:?}", time, id, event);
    //         active_gamepad = Some(id);
    //     }

    //     // You can also use cached gamepad state
    //     if let Some(gamepad) = active_gamepad.map(|id| gilrs.gamepad(id)) {
    //         if gamepad.is_pressed(Button::South) {
    //             println!("Button South is pressed (XBox - A, PS - X)");
    //         }
    //     }
    // }
    // nannou::app(model).simple_window(view).run();
    let update_visual = do_sound();
    // let mut display_buffer = VecDeque::new();
    visuals::run(update_visual);
    // loop {
    //     update_visual(&mut display_buffer);
    //     // draw display_buffer
    // }
}