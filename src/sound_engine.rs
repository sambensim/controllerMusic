use std::{sync::mpsc::{self}};
use crate::{chord_engine::{self, ChordEngine}, controller::{self}, input_engine::InputEvent};
use std::time::Instant;

pub struct NoteInfo {
    pub note : u8,
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
    controller_channel : tokio::sync::broadcast::Receiver<InputEvent>,
    chord_engine : chord_engine::ChordEngine,
    freq_send : mpsc::Sender<f32>,
    pub time_step : f32,
    pub voices : [Voice; SoundEngine::MAX_NOTES],
    pub current_chord : Vec<u8>
}

impl SoundEngine {
    pub fn init(controller_channel: tokio::sync::broadcast::Receiver<InputEvent>, frequency_send_channel: mpsc::Sender<f32>, sample_rate: f32) -> Self {
        SoundEngine {
            controller_channel : controller_channel,
            chord_engine : chord_engine::ChordEngine::new(0, 4),
            freq_send : frequency_send_channel,
            time_step : 1.0 / sample_rate,
            voices : std::array::from_fn(|_| Voice::new()),
            current_chord : Vec::new(),
        }
    }

    pub fn handle_input(&mut self) {
        let mut possible_event = self.controller_channel.try_recv();
        while !possible_event.is_err() {
            let event = possible_event.unwrap();
            match event.event_info {
                controller::InputEvent::Button(controller::ButtonType::RBumper, true) => {
                    self.release_chord(self.current_chord.clone());
                    self.play_chord({
                        if controller::get_left_stick_section(&event.full_state) == -1  { vec!() } else {
                            self.chord_engine.get_chord_notes(
                                controller::get_left_stick_section(&event.full_state) as i32, controller::get_right_stick_section(&event.full_state) as i32
                            ).into_iter().map(|note| { ChordEngine::note_to_value(&note).unwrap() }).collect()
                        }
                    })
                },
                _ => ()
            }
            possible_event = self.controller_channel.try_recv();
        };
    }

    fn release_chord(&mut self, notes : Vec<u8>) {
        for n in notes {
            for v in self.voices.iter_mut() {
                if let Some(ni) = &mut v.note_info {
                    if ni.note == n && ni.release.is_none() {
                        ni.release = Some(Instant::now());
                        break;
                    }
                }
            }
        }
    }

    fn play_chord(&mut self, notes : Vec<u8>) {
        for n in notes.iter() {
            let mut assigned : bool = false;
            for v in self.voices.iter_mut() {
                if let Some(ni) = &mut v.note_info {
                    if ni.release.is_some() {
                        ni.release = None;
                        ni.note = *n;
                        assigned = true;
                        break;
                    }
                }
            }
            if !assigned {
                for v in self.voices.iter_mut() {
                    if v.note_info.is_none() {
                        v.note_info = Some(NoteInfo {
                            note: *n, start: Instant::now(), release: None,
                        });
                        break;
                    }
                }
            }
            // if !assigned {
            //     TODO – PRINT ERROR AND OVERWRITE VOICE
            // }
        }
        self.current_chord = notes.clone();
    }

    pub fn send(&mut self, freq : f32) -> f32 {
        let _ = self.freq_send.send(freq);
        freq
    }

    pub const MAX_NOTES : usize = 4;
}