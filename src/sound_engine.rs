use std::{sync::mpsc::{self, Receiver}};
use crate::{controller::{self, DS4State}, chord_engine::{self, ChordEngine}};
use std::time::Instant;

pub struct NoteInfo {
    pub note : i8,
    pub start : Instant,
    pub release : Option<Instant>,
}

pub struct Voice {
    pub note_info : Option<NoteInfo>,
    pub phase : f32,
}

impl Voice {
    fn new() -> Self {
        Voice {
            note_info : None,
            phase : 0.0,
        }
    }
}

pub struct SoundEngine {
    controller_state : DS4State,
    controller_channel : Receiver<DS4State>,
    chord_engine : chord_engine::ChordEngine,
    freq_send : mpsc::Sender<f32>,
    pub time_step : f32,
    pub voices : [Voice; SoundEngine::MAX_NOTES],
}

impl SoundEngine {
    pub fn init(controller_channel: Receiver<DS4State>, frequency_send_channel: mpsc::Sender<f32>, sample_rate: f32) -> Self {
        SoundEngine {
            controller_state : controller_channel.recv().unwrap(),
            controller_channel : controller_channel,
            chord_engine : chord_engine::ChordEngine::new(0, 4),
            freq_send : frequency_send_channel,
            time_step : 1.0 / sample_rate,
            voices : std::array::from_fn(|_| Voice::new()),
        }
    }
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