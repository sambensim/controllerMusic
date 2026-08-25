use std::{collections::VecDeque, sync::mpsc::Receiver};

use crate::{chord_engine::{self, ChordEngine}, controller::{ButtonType, DiscreteType, InputEvent}, display_engine::Visual::Main, input_engine::FullInputEvent, intermediate_controller_state::{self, IntermediateControllerState}};

pub enum Visual {
    Setup,
    Main
}

pub struct DisplayEngine {
    controller_channel : Option<tokio::sync::broadcast::Receiver<FullInputEvent>>,
    chord_engine : chord_engine::ChordEngine,
    samp_channel : Option<Receiver<f32>>,
    samples : VecDeque<f32>,
    pub current_chord : String,
    pub selected_chord : String,
    pub current_visual : Visual,
    last_state : IntermediateControllerState
}

impl DisplayEngine {
    pub fn init() -> Self {
        DisplayEngine {
            controller_channel : None,
            chord_engine : chord_engine::ChordEngine::new(0, 4),
            samp_channel : None,
            samples : VecDeque::from([0.0_f32; DisplayEngine::SAMPLE_CAPACITY]),
            current_chord : "None".to_string(),
            selected_chord : "None".to_string(),
            current_visual : Visual::Setup,
            last_state : IntermediateControllerState::get_default(),
        }
    }

    pub fn handle_input(&mut self) {
        if let Some(controller) = &mut self.controller_channel {
            let mut possible_event = controller.try_recv();
            while !possible_event.is_err() {
                let event = possible_event.unwrap();
                match event.event_info {
                    InputEvent::Button(ButtonType::RBumper, true) => self.current_chord = Self::get_selected_chord(&self.chord_engine, &event.full_state),
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
                self.last_state = event.full_state;
                possible_event = controller.try_recv();
                self.selected_chord = Self::get_selected_chord(&self.chord_engine, &event.full_state);
            };
        }
    }

    fn get_selected_chord(chord_engine : &ChordEngine, full_state : &intermediate_controller_state::IntermediateControllerState) -> String {
        if full_state.quantize(DiscreteType::Left, 8) == -1 { "None".to_string() } else {
            chord_engine::ChordEngine::get_chord_name(chord_engine.get_key_value(), full_state.quantize(DiscreteType::Left, 8) as i32, full_state.quantize(DiscreteType::Right, 8) as i32 + (if full_state.get_button(ButtonType::LBumper) {9} else {0}))
        }
    }

    pub fn get_modes(&self) -> [&str; 8] {
        if !self.last_state.get_button(ButtonType::LBumper) {
            return ["7", "add9", "9", "dim", "aug", "sus4", "5", "6"];
        } else {
            return ["flip7", "flipadd9", "flip9", "dim7", "dom7", "7sus4", "sus2", "flip6"];
        }
    }

    pub fn get_samples(&mut self) -> &VecDeque<f32> {
        if let Some(samp_channel) = &self.samp_channel {
            let mut samp = samp_channel.try_recv();
            while !samp.is_err() {
                self.samples.push_front(samp.unwrap());
                samp = samp_channel.try_recv();
            }
            self.samples.truncate(DisplayEngine::SAMPLE_CAPACITY);
        }
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

    pub fn setup(&mut self, controller_channel : tokio::sync::broadcast::Receiver<FullInputEvent>, samp_channel : Receiver<f32>) {
        self.controller_channel = Some(controller_channel);
        self.samp_channel = Some(samp_channel);
        self.current_visual = Main;
    }

    pub const SAMPLE_CAPACITY : usize = 1000;
    pub const WIDTH : f32 = 640_f32;
    // pub const WIDTH : f32 = 1280_f32;
    pub const HEIGHT : f32 = 480_f32;
    // pub const HEIGHT : f32 = 960_f32;
}