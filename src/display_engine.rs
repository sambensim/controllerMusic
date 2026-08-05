use std::{collections::VecDeque, sync::mpsc::Receiver};

use crate::{chord_engine, controller::{self}, input_engine::InputEvent};

pub struct DisplayEngine {
    controller_channel : tokio::sync::broadcast::Receiver<InputEvent>,
    chord_engine : chord_engine::ChordEngine,
    samp_channel : Receiver<f32>,
    samples : VecDeque<f32>,
    pub current_chord : String,
    pub selected_chord : String,
}

impl DisplayEngine {
    pub fn init(controller_channel: tokio::sync::broadcast::Receiver<InputEvent>, samp_channel : Receiver<f32>) -> Self {
        DisplayEngine {
            controller_channel : controller_channel,
            chord_engine : chord_engine::ChordEngine::new(0, 4),
            samp_channel : samp_channel,
            samples : VecDeque::from([0.0_f32; DisplayEngine::SAMPLE_CAPACITY]),
            current_chord : "None".to_string(),
            selected_chord : "None".to_string(),
        }
    }

    pub fn handle_input(&mut self) {
        let mut possible_event = self.controller_channel.try_recv();
        while !possible_event.is_err() {
            let event = possible_event.unwrap();
            match event.event_info {
                controller::InputEvent::Directional(controller::DirectionalType::Left, _) | controller::InputEvent::Directional(controller::DirectionalType::Right, _)  => self.selected_chord = {
                    if controller::get_left_stick_section(&event.full_state) == -1  { "None".to_string() } else {
                        self.chord_engine.get_chord_name(controller::get_left_stick_section(&event.full_state) as i32, controller::get_right_stick_section(&event.full_state) as i32)
                    }
                },
                controller::InputEvent::Button(controller::ButtonType::RBumper, true) => self.current_chord = {
                    if controller::get_left_stick_section(&event.full_state) == -1  { "None".to_string() } else {
                        self.chord_engine.get_chord_name(controller::get_left_stick_section(&event.full_state) as i32, controller::get_right_stick_section(&event.full_state) as i32)
                    }
                },
                controller::InputEvent::Button(controller::ButtonType::Share, true) => self.chord_engine.increment_key(),
                controller::InputEvent::Button(controller::ButtonType::Options, true) => self.chord_engine.increment_octave(),
                _ => ()
            }
            possible_event = self.controller_channel.try_recv();
        };
    }

    pub fn get_samples(&mut self) -> &VecDeque<f32> {
        let mut samp = self.samp_channel.try_recv();
        while !samp.is_err() {
            self.samples.push_front(samp.unwrap());
            samp = self.samp_channel.try_recv();
        }
        self.samples.truncate(DisplayEngine::SAMPLE_CAPACITY);
        &self.samples
    }

    pub fn get_key(&self) -> u8 {
        self.chord_engine.get_key()
    }

    pub fn get_octave(&self) -> u8 {
        self.chord_engine.get_octave()
    }

    pub const SAMPLE_CAPACITY : usize = 1000;
    pub const WIDTH : f32 = 640_f32;
    // pub const WIDTH : f32 = 1280_f32;
    pub const HEIGHT : f32 = 480_f32;
    // pub const HEIGHT : f32 = 960_f32;
}