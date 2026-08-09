use std::{sync::mpsc::{self}};
use crate::{chord_engine::{self, ChordEngine}, controller_trait::DirectionalType, input_engine::FullInputEvent, synths::{self, Oscillator, Saw}};
use std::time::Instant;

pub struct NoteInfo {
    pub note : u8,
    pub _start : Instant,
    pub release : Option<Instant>,
}

pub struct Voice <T> where T: synths::Oscillator {
    pub note_info : Option<NoteInfo>,
    pub osc : T,
    pub env : crate::adsr::Adsr,
}

impl<T> Voice <T> where T: synths::Oscillator {
    fn new(sample_rate : u32) -> Self {
        Voice {
            note_info : None,
            osc : T::new(sample_rate),
            env : crate::adsr::Adsr::new(sample_rate, 1.0, 3.0, 0.6, 1.0),
        }
    }

    pub fn step(&mut self) -> f32 {
        let Some(note_info) = &self.note_info else {
            return 0.0;
        };
        self.env.step();
        self.osc.step(ChordEngine::value_to_freq(note_info.note))
    }
}

pub struct SoundEngine {
    controller_channel : tokio::sync::broadcast::Receiver<FullInputEvent>,
    chord_engine : chord_engine::ChordEngine,
    freq_send : mpsc::Sender<f32>,
    pub voices : [Voice<Saw>; SoundEngine::MAX_NOTES],
    pub current_chord : Vec<u8>
}

impl SoundEngine {
    pub fn init(controller_channel: tokio::sync::broadcast::Receiver<FullInputEvent>, frequency_send_channel: mpsc::Sender<f32>, sample_rate: u32) -> Self {
        SoundEngine {
            controller_channel : controller_channel,
            chord_engine : chord_engine::ChordEngine::new(0, 4),
            freq_send : frequency_send_channel,
            voices : std::array::from_fn(|_| Voice::new(sample_rate)),
            current_chord : Vec::new(),
        }

    }

    pub fn handle_input(&mut self) {
        let mut possible_event = self.controller_channel.try_recv();
        while !possible_event.is_err() {
            let event = possible_event.unwrap();
            match event.event_info {
                crate::controller_trait::InputEvent::Button(crate::controller_trait::ButtonType::RBumper, true) => {
                    self.release_chord(self.current_chord.clone());
                    self.play_chord({
                        if event.full_state.quantize_directional(DirectionalType::Left, 8) == -1  { vec!() } else {
                            self.chord_engine.get_chord_notes(
                                event.full_state.quantize_directional(DirectionalType::Left, 8) as i32, event.full_state.quantize_directional(DirectionalType::Right, 8) as i32
                            ).into_iter().map(|note| { ChordEngine::note_to_value(&note).unwrap() }).collect()
                        }
                    })
                },
                crate::controller_trait::InputEvent::Button(crate::controller_trait::ButtonType::Share, true) => self.chord_engine.increment_key(),
                crate::controller_trait::InputEvent::Button(crate::controller_trait::ButtonType::Options, true) => self.chord_engine.increment_octave(),
                crate::controller_trait::InputEvent::Button(crate::controller_trait::ButtonType::LStickBtn, true) => {
                    self.chord_engine.decrement_key();
                    self.chord_engine.decrement_octave();
                },
                crate::controller_trait::InputEvent::Button(crate::controller_trait::ButtonType::LStickBtn, false) => {
                    self.chord_engine.increment_key();
                    self.chord_engine.increment_octave();
                },
                crate::controller_trait::InputEvent::Button(crate::controller_trait::ButtonType::RStickBtn, true) => self.chord_engine.increment_key(),
                crate::controller_trait::InputEvent::Button(crate::controller_trait::ButtonType::RStickBtn, false) => self.chord_engine.decrement_key(),
                _ => ()
            }
            for v in &mut self.voices {
                v.osc.handle_input(event.event_info);
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
                        v.env.release();
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
                        v.env.trigger();
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
                            note: *n, _start: Instant::now(), release: None,
                        });
                        v.env.trigger();
                        break;
                    }
                }
            }
        }
        self.current_chord = notes.clone();
    }

    pub fn send(&mut self, freq : f32) -> f32 {
        let _ = self.freq_send.send(freq);
        freq
    }

    pub const MAX_NOTES : usize = 4;
}