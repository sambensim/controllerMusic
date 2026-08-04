use std::{collections::VecDeque, sync::mpsc::Receiver};

use crate::{chord_engine, controller::{self, DS4State}, input_engine::InputEvent};

pub struct DisplayEngine {
    controller_state : DS4State,
    controller_channel : tokio::sync::broadcast::Receiver<InputEvent>,
    chord_engine : chord_engine::ChordEngine,
    samp_channel : Receiver<f32>,
    samples : VecDeque<f32>,
}

impl DisplayEngine {
    pub fn init(controller_channel: tokio::sync::broadcast::Receiver<InputEvent>, samp_channel : Receiver<f32>) -> Self {
        DisplayEngine {
            controller_state : controller::DS4_EMPTY,
            controller_channel : controller_channel,
            chord_engine : chord_engine::ChordEngine::new(0, 4),
            samp_channel : samp_channel,
            samples : VecDeque::from([0.0_f32; DisplayEngine::SAMPLE_CAPACITY]),
        }
    }

    pub fn get_state(&mut self) -> DS4State {
        let new_state = self.controller_channel.try_recv();
        if !new_state.is_err() {
            self.controller_state = new_state.unwrap().full_state;
        };
        self.controller_state
    }

    pub fn get_chord(&mut self) -> String {
        let state = self.get_state();
        let loct = controller::get_left_stick_section(&state);
        if loct == -1 {
            return "none".to_string();
        }
        let roct = controller::get_right_stick_section(&state);
        self.chord_engine.get_chord_name(loct as i32, roct as i32)
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

    pub const SAMPLE_CAPACITY : usize = 1000;
    pub const WIDTH : f32 = 640_f32;
    pub const HEIGHT : f32 = 480_f32;
}