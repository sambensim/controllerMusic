use std::{collections::VecDeque, sync::mpsc::Receiver};

use crate::{chord_engine::{self, ChordEngine}, controller::{ButtonType, DiscreteType, InputEvent}, input_engine::FullInputEvent, intermediate_controller_state};

pub struct DisplayEngine {
    controller_channel : tokio::sync::broadcast::Receiver<FullInputEvent>,
    chord_engine : chord_engine::ChordEngine,
    samp_channel : Receiver<f32>,
    samples : VecDeque<f32>,
    pub current_chord : String,
    pub selected_chord : String,
}

impl DisplayEngine {
    pub fn init(controller_channel: tokio::sync::broadcast::Receiver<FullInputEvent>, samp_channel : Receiver<f32>) -> Self {
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
                // InputEvent::Discrete(DiscreteType::TouchX, v) => {
                //     println!("{v}")
                // }
                // InputEvent::Button(ButtonType::Touch, true) => self.play
                InputEvent::Button(ButtonType::RBumper, true) => self.current_chord = self.get_selected_chord(&event.full_state),
                InputEvent::Button(ButtonType::Share, true) => self.chord_engine.increment_key(),
                InputEvent::Button(ButtonType::Options, true) => self.chord_engine.increment_octave(),
                InputEvent::Button(ButtonType::LStickBtn, true) => {
                    self.chord_engine.decrement_key();
                    self.chord_engine.decrement_octave();
                },
                InputEvent::Button(ButtonType::LStickBtn, false) => {
                    self.chord_engine.increment_key();
                    self.chord_engine.increment_octave();
                },
                InputEvent::Button(ButtonType::RStickBtn, true) => self.chord_engine.increment_key(),
                InputEvent::Button(ButtonType::RStickBtn, false) => self.chord_engine.decrement_key(),
                _ => ()//println!("{:?}", event.event_info)
            }
            possible_event = self.controller_channel.try_recv();
            self.selected_chord = self.get_selected_chord(&event.full_state);
        };
    }

    fn get_selected_chord(&self, full_state : &intermediate_controller_state::IntermediateControllerState) -> String {
        if full_state.quantize(DiscreteType::Left, 8) == -1 { "None".to_string() } else {
            chord_engine::ChordEngine::get_chord_name(self.chord_engine.get_key_value(), full_state.quantize(DiscreteType::Left, 8) as i32, full_state.quantize(DiscreteType::Right, 8) as i32)
        }
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

    pub fn get_key(&self) -> String {
        self.chord_engine.get_key()
    }

    pub fn get_octave(&self) -> u8 {
        self.chord_engine.get_octave()
    }

    pub fn get_key_notes(&self) -> Vec<String> {
        ChordEngine::get_key_notes(self.chord_engine.get_key_value())
    }

    pub const SAMPLE_CAPACITY : usize = 1000;
    pub const WIDTH : f32 = 640_f32;
    // pub const WIDTH : f32 = 1280_f32;
    pub const HEIGHT : f32 = 480_f32;
    // pub const HEIGHT : f32 = 960_f32;
}