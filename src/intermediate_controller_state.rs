use std::f32::consts::PI;
use std::sync::{mpsc};
use std::thread;
use crate::controller::ContinuousType;
use crate::controller::{BUTTONS, Controller, DiscreteType, InputEvent};

#[derive(Copy, Clone, Debug)]
pub struct IntermediateControllerState {
     // -1.0 to 1.0
    pub left_stick_x: f32,
    pub left_stick_y: f32,
    pub right_stick_x: f32,
    pub right_stick_y: f32,
    pub touchpad_x : f32,
    pub touchpad_y : f32,
    // 0.0 to 1.0
    pub l_trigger: f32,
    pub r_trigger: f32,
    // Buttons
    pub packed_button_states : u16, //Square, Cross, Circle, Triangle, LBumper, RBumper, LTriggerBtn, RTriggerBtn, Share, Options, LStickBtn, RStickBtn,
    // D-pad: 0-7 clockwise from up, 8 == none //TODO - translate to be the same orientation as joysticks
    pub dpad: i8,
}

impl IntermediateControllerState {
    pub fn get_events(&self, new_state : Self) -> impl Iterator<Item = InputEvent> {
        let left = (self.quantize(DiscreteType::Left, 8) != (new_state.quantize(DiscreteType::Left, 8)))
            .then(|| InputEvent::Discrete(DiscreteType::Left, new_state.quantize(DiscreteType::Left, 8), 8))
            .into_iter();
        let right =  (self.quantize(DiscreteType::Right, 8) != (new_state.quantize(DiscreteType::Right, 8)))
            .then(|| InputEvent::Discrete(DiscreteType::Right, new_state.quantize(DiscreteType::Right, 8), 8))
            .into_iter();
        let dpad = (self.dpad != new_state.dpad)
            .then(|| InputEvent::Discrete(DiscreteType::Dpad, new_state.dpad, 8))
            .into_iter();
        let touch_x = (self.quantize(DiscreteType::TouchX, 12) != new_state.quantize(DiscreteType::TouchX, 12))
            .then(|| InputEvent::Discrete(DiscreteType::TouchX, new_state.quantize(DiscreteType::TouchX, 12), 12))
            .into_iter();
        let touch_y = (self.quantize(DiscreteType::TouchY, 8) != new_state.quantize(DiscreteType::TouchY, 8))
            .then(|| InputEvent::Discrete(DiscreteType::TouchY, new_state.quantize(DiscreteType::TouchY, 8), 8))
            .into_iter();
        let buttons = self.button_events(new_state);
        let left_trigger = (self.l_trigger != new_state.l_trigger)
            .then(|| InputEvent::Continuous(ContinuousType::LeftTrigger, new_state.l_trigger))
            .into_iter();
        let right_trigger = (self.r_trigger != new_state.r_trigger)
            .then(|| InputEvent::Continuous(ContinuousType::RightTrigger, new_state.r_trigger))
            .into_iter();

        left.chain(right).chain(dpad).chain(touch_x).chain(touch_y)
            .chain(buttons)
            .chain(left_trigger).chain(right_trigger)
    } 

    pub fn quantize(&self, directional_type : DiscreteType, regions : u8) -> i8 {
        return match directional_type {
            DiscreteType::Left => {
                get_vec_section(self.left_stick_x, self.left_stick_y, regions)
            },
            DiscreteType::Right => {
                get_vec_section(self.right_stick_x, self.right_stick_y, regions)
            },
            DiscreteType::TouchX => {
                ((self.touchpad_x * (regions as f32)) as i8).clamp(0, 7)
            },
            DiscreteType::TouchY => {
                ((self.touchpad_y * (regions as f32)) as i8).clamp(0, 7)
            },
            _ => -1
        };
    }

    pub fn get_default() -> Self {
        IntermediateControllerState {
            left_stick_x: 0.0,
            left_stick_y: 0.0,
            right_stick_x: 0.0,
            right_stick_y: 0.0,
            touchpad_x: 0.0,
            touchpad_y: 0.0,
            l_trigger: 0.0,
            r_trigger: 0.0,
            packed_button_states: 0,
            dpad: 0
        }
    }

    fn button_events(&self, new_state : Self) -> impl Iterator<Item = InputEvent> {
        let prev = self.packed_button_states;
        let next = new_state.packed_button_states;
        let changed = prev ^ next;
        BUTTONS.iter().enumerate()
        .filter(move |(i, _)| changed & (1u16 << i) != 0)
        .map(move |(i, &b)| InputEvent::Button(b, next & (1u16 << i) != 0))
    }
}

pub fn start_controller_thread<T>(device: hidapi::HidDevice) -> mpsc::Receiver<IntermediateControllerState> where T: Controller {
    let (sender, receiver) = mpsc::channel();
    thread::spawn(move || {
        let mut buf = [0u8; 78];
        loop {
            if let Ok(_) = device.read(&mut buf) {
                // _print_data(&buf);
                let parsed = T::parse_report(&buf);
                if let Some(data) = parsed {
                    // println!("{data:?}");
                    // println!("{:?}", get_button_state(&data, ButtonType::RBumper));
                    let _ = sender.send(data);
                }
            }
        }
    });
    receiver
}

const THRESHOLD : f32 = 0.4;
fn get_vec_section(x : f32, y : f32, sections : u8) -> i8 {
    let magnitude = (x * x + y * y).sqrt();
    if magnitude < THRESHOLD { return -1 } ;
    let angle : f32 = y.atan2(x) + (PI / (sections) as f32);
    let normalized_angle : f32 = if angle < 0.0 { angle + 2.0 * PI } else { angle };
    ((normalized_angle / ((2.0 * PI) / sections as f32)) as u8 % sections) as i8
}