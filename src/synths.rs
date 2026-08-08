use std::f32::consts::PI;

use raylib::audio::Sound;

use crate::{chord_engine::ChordEngine, controller::{InputEvent, TriggerType}, sound_engine::SoundEngine};

const VOLUME : f32 = 0.4;

pub fn get_process(mut sound_engine : SoundEngine) -> impl FnMut(f32, f32, f32) -> f32 {
    {
        move |_: f32, _: f32, _: f32| -> f32 {
            sound_engine.handle_input();
            let mut out : f32 = 0.0;
            for voice in &mut sound_engine.voices {
                let samp = voice.step();
                out += samp * voice.env.level;
            };
            out /= sound_engine.voices.len().max(1) as f32;
            sound_engine.send(out * VOLUME)
        }
    }
}

enum AdsrPhase {
    Attack,
    Decay,
    Sustain,
    Release,
}

pub struct Adsr {
    atime_secs : f32,
    dtime_secs : f32,
    slevel : f32,
    rtime_secs : f32,
    phase : AdsrPhase,
    pub level : f32,
    note_pressed : bool,
    sample_rate : u32,
}

impl Adsr {
    pub fn new(sample_rate : u32, a : f32, d : f32, s : f32, r : f32) -> Adsr {
        Adsr {
            atime_secs : a,
            dtime_secs : d,
            slevel : s,
            rtime_secs : r,
            phase : AdsrPhase::Release,
            level : 0.0,
            note_pressed : false,
            sample_rate : sample_rate,
        }
    }

    pub fn step(&mut self) {
        match self.phase {
            AdsrPhase::Attack => {
                self.level = (self.level + (1.0 / self.atime_secs) / self.sample_rate as f32).min(1.0);
                if self.level >= 1.0 {
                    self.phase = AdsrPhase::Decay;
                }
            },
            AdsrPhase::Decay => {
                self.level = (self.level - (1.0 / self.dtime_secs) / self.sample_rate as f32).max(self.slevel);
                if self.level <= self.slevel {
                    self.phase = AdsrPhase::Sustain;
                }
            },
            AdsrPhase::Sustain => {
                self.level = self.slevel;
            },
            AdsrPhase::Release => {
                self.level = (self.level - (1.0 / self.rtime_secs) / self.sample_rate as f32).max(0.0);
            }
        }
    }

    pub fn trigger(&mut self) {
        self.phase = AdsrPhase::Attack
    }

    pub fn release(&mut self) {
        self.phase = AdsrPhase::Release
    }
}

pub trait Oscillator {
    fn new(sample_rate : u32) -> Self;
    fn step(&mut self, freq : f32) -> f32;
    fn handle_input(&mut self, event : crate::controller::InputEvent);
}

pub struct Saw {
    phase : f32,
    time_step : f32,
}

impl Oscillator for Saw {
    fn new(sample_rate : u32) -> Self {
        Saw {
            phase : 0.0,
            time_step : 1.0 / sample_rate as f32,
        }
    }

    fn step(&mut self, freq : f32) -> f32 {
        self.phase += freq * self.time_step ;
        self.phase %= 1.0;
        self.phase*2.0 - 1.0
    }

    fn handle_input(&mut self, _event : crate::controller::InputEvent) {
        
    }

    
}

pub struct Fm {
    carrier_phase : f32,
    modulator_phase : f32,
    time_step : f32,
    ratio : f32,
    modulation_amplitude : f32,
    target_modulation_amplitude : f32,
}

impl Fm {
    pub fn set(&mut self, ratio : f32, modulation_amplitude : f32) {
        self.ratio = ratio;
        self.target_modulation_amplitude = modulation_amplitude;
    }
}

impl Oscillator for Fm {
    fn new(sample_rate : u32) -> Self {
        Fm {
            carrier_phase : 0.0,
            modulator_phase : 0.0,
            time_step : 1.0 / sample_rate as f32,
            ratio : 0.5,
            modulation_amplitude : 0.0,
            target_modulation_amplitude : 0.0,
        }
    }

    fn step(&mut self, freq : f32) -> f32 {
        self.carrier_phase += freq * self.time_step ;
        self.carrier_phase %= 1.0;

        self.modulation_amplitude += (self.target_modulation_amplitude - self.modulation_amplitude) * 0.001;
        self.modulator_phase += self.ratio * freq * self.time_step;
        self.modulator_phase %= 1.0;
        let modulator_sin = self.modulation_amplitude * (self.modulator_phase * 2.0 * PI).sin();

        (self.carrier_phase * 2.0 * PI + modulator_sin).sin()
    }

   fn handle_input(&mut self, event : crate::controller::InputEvent) {
        let InputEvent::Continuous(TriggerType::Left, v) = event else {
            return;
        };
        self.set(self.ratio, v * 4.0) 
    }
}