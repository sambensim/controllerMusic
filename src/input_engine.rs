use std::sync::mpsc::Receiver;

use crate::intermediate_controller_state::IntermediateControllerState;

#[derive(Clone)]
pub struct FullInputEvent {
    pub full_state : crate::intermediate_controller_state::IntermediateControllerState,
    pub event_info : crate::controller_trait::InputEvent,
}


pub struct InputEngine {
    last_state : IntermediateControllerState,
    controller_stream : Receiver<crate::intermediate_controller_state::IntermediateControllerState>,
    controller_event_channel : tokio::sync::broadcast::Sender<FullInputEvent>,
}

impl InputEngine {
    pub fn init<T>() -> InputEngine where T: crate::controller_trait::Controller {
        let device = T::get_controller().unwrap();
        let controller_stream = crate::intermediate_controller_state::start_controller_thread::<T>(device);
        let (sender, _) = tokio::sync::broadcast::channel(32);
       
        InputEngine {
            last_state : IntermediateControllerState::get_default(),
            controller_stream : controller_stream,
            controller_event_channel : sender,
        }
    }

    pub fn subscribe(&self) ->  tokio::sync::broadcast::Receiver<FullInputEvent> {
        self.controller_event_channel.subscribe()
    }

    pub fn step(&mut self) {
        let mut new_state = self.controller_stream.try_recv();
        while !new_state.is_err() {
            let events = self.last_state.get_events(new_state.unwrap());
            for e in events {
                let _ = self.controller_event_channel.send(FullInputEvent {
                    full_state : new_state.unwrap(),
                    event_info : e,
                });
                // print!("{:?}\n", e);
            }
            self.last_state = new_state.unwrap();
            new_state = self.controller_stream.try_recv();
        }
    }
}