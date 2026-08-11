use std::{sync::mpsc::{self}};
use crate::{adsr::Adsr, chord_engine::{self, ChordEngine}, controller::{ButtonType, ContinuousType::LeftTrigger, DiscreteType, InputEvent, InputSource}, effect::{Delay, Gain, Noise}, input_engine::FullInputEvent, instrument::{self, InputMapSpec, Instrument, ParamOverride, Target, TargetSpec}, intermediate_controller_state::IntermediateControllerState, oscillator::{Fm, Saw, Sin}, voice::Voice, voice_manager::{self, VoiceManager}};


pub struct SoundEngine {
    controller_channel : tokio::sync::broadcast::Receiver<FullInputEvent>,
    chord_engine : chord_engine::ChordEngine,
    freq_send : mpsc::Sender<f32>,
    pub instruments : Vec<Box<Instrument>>,
    pub main_instrument : usize,
    pub lead_instrument : usize,
    pub current_chord : Vec<u8>,
    pub lead_note : u8,
}

impl SoundEngine {
    pub fn init(controller_channel: tokio::sync::broadcast::Receiver<FullInputEvent>, frequency_send_channel: mpsc::Sender<f32>, sample_rate: u32) -> Self {
        let mut voice_manager= VoiceManager::new();
        SoundEngine {
            controller_channel : controller_channel,
            chord_engine : chord_engine::ChordEngine::new(0, 4),
            freq_send : frequency_send_channel,
            instruments : vec![

                Box::new(Instrument::new(sample_rate,
                    "Fm+Delay",
                    |sr : u32| {Box::new(Fm::new(sr, |sr : u32| {Box::new(Sin::new(sr))}))},
                    |sr : u32| {Adsr::new(sr, 1.0, 1.0, 0.8, 1.0)},
                    &mut voice_manager,
                    &mut vec![
                        ("delay", Box::new(Delay::new(sample_rate, 1.0))),
                        ("delay2", Box::new(Delay::new(sample_rate, 1.0))),
                        ("delay3", Box::new(Delay::new(sample_rate, 1.0))),
                        ("delay4", Box::new(Delay::new(sample_rate, 1.0))),
                        ("noise", Box::new(Noise::new())),
                        ("gain", Box::new(Gain::new())),
                        ],
                    &mut vec![
                        InputMapSpec {
                            target : TargetSpec::Osc,
                            param : "amplitude",
                            input : InputSource::Continuous(LeftTrigger),
                            response : instrument::Curve::Linear(),
                        }
                    ],
                    vec![
                        ParamOverride {
                            target : TargetSpec::Effect("gain"),
                            param : "amount",
                            value : 2.4,
                        },
                        ParamOverride {
                            target : TargetSpec::Effect("noise"),
                            param : "mix",
                            value : 0.005,
                        },
                        ParamOverride {
                            target : TargetSpec::Effect("delay"),
                            param : "mix",
                            value : 0.4,
                        },
                        ParamOverride {
                            target : TargetSpec::Effect("delay2"),
                            param : "mix",
                            value : 0.3,
                        },
                        ParamOverride {
                            target : TargetSpec::Effect("delay3"),
                            param : "mix",
                            value : 0.3,
                        },
                        ParamOverride {
                            target : TargetSpec::Effect("delay4"),
                            param : "mix",
                            value : 0.3,
                        },
                    ],
                    8,
                )),
                
                Box::new(Instrument::new(sample_rate,
                    "Saw",
                    |sr : u32| {Box::new(Saw::new(sr))},
                    |sr : u32| {Adsr::new(sr, 1.0, 1.0, 0.8, 1.0)},
                    &mut voice_manager,
                    &mut vec![
                        ("gain", Box::new(Gain::new())),
                    ],
                    &mut vec![],
                    vec![
                        ParamOverride {
                            target : TargetSpec::Effect("gain"),
                            param : "amount",
                            value : 0.8,
                        },
                    ],
                    8,
                )),
            
                ],
            main_instrument : 0,
            lead_instrument : 1,
            current_chord : Vec::new(),
            lead_note : 0,
        }

    }

    const MAIN : isize = -1;
    const LEAD : isize = -2;

    pub fn handle_input(&mut self) {
        let mut possible_event = self.controller_channel.try_recv();
        while !possible_event.is_err() {
            let event = possible_event.unwrap();
            match event.event_info {
                InputEvent::Button(ButtonType::RBumper, true) => {
                    self.release_chord(self.current_chord.clone(), Self::MAIN);
                    self.play_chord({
                        if event.full_state.quantize(DiscreteType::Left, 8) == -1  { vec!() } else {
                            self.chord_engine.get_chord_notes(
                                event.full_state.quantize(DiscreteType::Left, 8) as i32, event.full_state.quantize(DiscreteType::Right, 8) as i32
                            ).into_iter().map(|note| { ChordEngine::note_to_value(&note).unwrap() }).collect()
                        }
                    }, Self::MAIN)
                },
                InputEvent::Button(ButtonType::Touch, true) | InputEvent::Discrete(DiscreteType::TouchX, _, _)=> {
                    self.update_lead(event.full_state, 12)
                },
                InputEvent::Button(ButtonType::Touch, false) => self.release_note(self.lead_note, -2),
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
                InputEvent::Button(ButtonType::PS, true) => {
                    self.instruments[self.main_instrument].release_all();
                    self.instruments[self.lead_instrument].release_all();
                    self.main_instrument = (self.main_instrument + 1) % self.instruments.len();
                    self.lead_instrument = (self.lead_instrument + 1) % self.instruments.len();
                },
                _ => {
                    for instr in &mut self.instruments {
                        instr.handle_input(&event.event_info);
                    }
                }
            }
            
            possible_event = self.controller_channel.try_recv();
        };
    }

    fn update_lead(&mut self, state : IntermediateControllerState, regions : u8) {
        let region = state.quantize(DiscreteType::TouchX, regions);
        let mut note;
        let key;
        if self.current_chord.len() > 0 {
            let base = self.current_chord[region as usize % self.current_chord.len()];
            note = base + 12_u8 * (region / self.current_chord.len() as i8) as u8;
        } else {
            key = self.chord_engine.get_key_value();
            let kn = ChordEngine::get_key_notes(key);
            let base = &kn[region as usize % kn.len()];
            let oct = 5;
            note = ChordEngine::note_to_value(&format!("{}{}", &base, oct)).unwrap();
            let temp = &ChordEngine::note_add(note, 12 * (region as usize / kn.len()) as u8);
            note = ChordEngine::note_to_value(&temp).unwrap();
        }
        self.release_note(self.lead_note, -2);
        self.lead_note = note;
        self.play_note(note, -2);
    }

    fn get_instrument(&mut self, instr : isize) -> &mut Box<Instrument> {
        &mut self.instruments[
        match instr {
                Self::MAIN => self.main_instrument,
                Self::LEAD => self.lead_instrument,
                _ => panic!("unknown index: {instr}")
            }
        ]
    }

    fn release_note(&mut self, note : u8, instr : isize) {
        self.get_instrument(instr).release(note);
    }

    fn release_chord(&mut self, notes : Vec<u8>, instr : isize) {
        for n in notes {
            self.release_note(n, instr);
        }
    }

    fn play_note(&mut self, note : u8, instr : isize) {
        self.get_instrument(instr).play(note);
    }

    fn play_chord(&mut self, notes : Vec<u8>, instr : isize) {
        for n in notes.iter() {
            self.play_note(*n, instr);
        }
        self.current_chord = notes.clone();
    }

    pub fn send(&mut self, freq : f32) -> f32 {
        let _ = self.freq_send.send(freq);
        freq
    }
}

pub fn get_process(mut sound_engine : SoundEngine) -> impl FnMut(f32, f32, f32) -> f32 {
    move |_: f32, _: f32, _: f32| -> f32 {
        sound_engine.handle_input();
        let mut out : f32 = 0.0;
        for instrument in &mut sound_engine.instruments {
            out += instrument.step();
        };
        out *= 0.5_f32.sqrt().powi(sound_engine.instruments.len().max(1) as i32); //TODO, set by active instead of all
        sound_engine.send(out)
    }
}