use std::sync::mpsc::{self, Receiver};

use crate::{controller::{self, DS4State}, chord_engine::{self, ChordEngine}};

pub struct SoundEngine {
    pub controller_state : DS4State,
    pub controller_channel : Receiver<DS4State>,
    pub chord_engine : chord_engine::ChordEngine,
    pub phases : [f32; SoundEngine::MAX_NOTES],
    pub freq_send : mpsc::Sender<f32>,
    pub time_step : f32,
}

impl SoundEngine {
    pub fn get_state(&mut self) -> DS4State {
        let new_state = self.controller_channel.try_recv();
        if !new_state.is_err() {
            self.controller_state = new_state.unwrap();
        };
        self.controller_state
    }

    pub fn get_chord(&mut self) -> Vec<f32> {
        let state = self.get_state();
        let loct = controller::get_left_stick_section(&state);
        if loct == -1 {
            return vec!();
        };
        let roct = controller::get_right_stick_section(&state);
        let notes = self.chord_engine.get_chord_notes(loct as i32, roct as i32);
        notes.iter().map(|n : &String| ChordEngine::note_to_freq(n).unwrap()).collect()
    }

    pub fn send(&mut self, freq : f32) -> f32 {
        let _ = self.freq_send.send(freq);
        freq
    }

    pub const MAX_NOTES : usize = 4;
}