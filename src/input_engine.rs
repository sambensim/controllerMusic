use std::sync::mpsc::Receiver;

use crate::controller::{self, DS4State};

#[derive(Clone)]
pub struct InputEvent {
    pub full_state : DS4State,
    pub event_info : controller::InputEvent,
}


pub struct InputEngine {
    last_state : DS4State,
    controller_stream : Receiver<DS4State>,
    controller_event_channel : tokio::sync::broadcast::Sender<InputEvent>,
}

impl InputEngine {
    pub fn init() -> InputEngine {
        let dualshock = controller::get_dualshock().unwrap();
        let controller_stream = controller::start_controller_thread(dualshock);
        let (sender, _) = tokio::sync::broadcast::channel(32);
       
        InputEngine {
            last_state : controller::DS4_EMPTY,
            controller_stream : controller_stream,
            controller_event_channel : sender,
        }
    }

    pub fn subscribe(&self) ->  tokio::sync::broadcast::Receiver<InputEvent> {
        self.controller_event_channel.subscribe()
    }

    pub fn step(&mut self) {
        let mut new_state = self.controller_stream.try_recv();
        while !new_state.is_err() {
            let events = controller::get_events(self.last_state, new_state.unwrap());
            for e in events {
                let _ = self.controller_event_channel.send(InputEvent {
                    full_state : new_state.unwrap(),
                    event_info : e,
                });
                print!("{:?}\n", e);
            }
            self.last_state = new_state.unwrap();
            new_state = self.controller_stream.try_recv();
        }
    }
}