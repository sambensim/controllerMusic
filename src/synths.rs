use std::f32::consts::PI;

use crate::{sound_engine::SoundEngine};

const VOLUME : f32 = 1.0;

pub fn get_process(mut sound_engine : SoundEngine) -> impl FnMut(f32, f32, f32) -> f32 {
    {
        move |_: f32, _: f32, _: f32| -> f32 {
            sound_engine.handle_input();
            let mut out : f32 = 0.0;
            for voice in &mut sound_engine.voices {
                let samp = voice.step();
                out += samp * voice.env.level;
            };
            out *= 0.5_f32.sqrt().powi(sound_engine.voices.len().max(1) as i32);
            sound_engine.send(out * VOLUME)
        }
    }
}


pub trait Oscillator {
    fn new(sample_rate : u32) -> Self;
    fn step(&mut self, freq : f32) -> f32;
    fn handle_input(&mut self, event : crate::controller_trait::InputEvent);
    fn get_next_phase(&self, phase : f32, time_step : f32, freq : f32) -> f32 {
        (phase + freq * time_step) % 1.0
    }
}

pub struct None {}

impl Oscillator for None {
    fn new(_sample_rate : u32) -> Self {
        None {  }
    }
    fn step(&mut self, _freq : f32) -> f32 {
        0.0
    }
    fn handle_input(&mut self, _event : crate::controller_trait::InputEvent) {
        ()
    }
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
        self.phase = self.get_next_phase(self.phase, self.time_step, freq);
        self.phase*2.0 - 1.0
    }

    fn handle_input(&mut self, _event : crate::controller_trait::InputEvent) {
        
    }
}

pub struct Sin {
    phase : f32,
    time_step : f32,
}

impl Oscillator for Sin {
    fn new(sample_rate : u32) -> Self {
        Sin {
            phase : 0.0,
            time_step : 1.0 / sample_rate as f32,
        }
    }

    fn step(&mut self, freq : f32) -> f32 {
        self.phase = self.get_next_phase(self.phase, self.time_step, freq);
        (self.phase*2.0*PI).sin()
    }

    fn handle_input(&mut self, _event : crate::controller_trait::InputEvent) {}
}

pub struct Fm <T> where T: Oscillator {
    carrier : T,
    modulator_phase : f32,
    time_step : f32,
    ratio : f32,
    modulation_amplitude : f32,
    target_modulation_amplitude : f32,
}

impl <T>Fm <T> where T: Oscillator{
    pub fn set(&mut self, ratio : f32, modulation_amplitude : f32) {
        self.ratio = ratio;
        self.target_modulation_amplitude = modulation_amplitude;
    }
}

impl <T>Oscillator for Fm <T> where T: Oscillator{
    fn new(sample_rate : u32) -> Self {
        Fm {
            carrier : T::new(sample_rate),
            modulator_phase : 0.0,
            time_step : 1.0 / sample_rate as f32,
            ratio : 0.5,
            modulation_amplitude : 0.0,
            target_modulation_amplitude : 0.0,
        }
    }

    fn step(&mut self, freq : f32) -> f32 {
        self.modulation_amplitude += (self.target_modulation_amplitude - self.modulation_amplitude) * 0.001; //0.001 == smoothing amt
        self.modulator_phase = self.get_next_phase(self.modulator_phase, self.time_step, self.ratio * freq);
        let modulator_sin = self.modulation_amplitude * (self.modulator_phase * 2.0 * PI).sin();

        (self.carrier.step(freq) + modulator_sin).sin()
    }

   fn handle_input(&mut self, event : crate::controller_trait::InputEvent) {
        match event {
            crate::controller_trait::InputEvent::Continuous(crate::controller_trait::ContinuousType::LeftTrigger, v) => self.set(self.ratio, v * 4.0),
            crate::controller_trait::InputEvent::Button(crate::controller_trait::ButtonType::RBumper, true) => {
                let n = [0.25, 0.5, 1.0][fastrand::usize(1..3)];
                println!("{n}");
                self.set( n, self.modulation_amplitude)
            },
            _ => ()
        };
    }
}

pub struct GlideSin <T> where T: Oscillator {
    osc : T,
    time_step : f32,
    freq : f32,
    glide_secs : f32,
}

impl<T> GlideSin <T> where T: Oscillator{
    pub fn set(&mut self,  glide_secs : f32) {
        self.glide_secs = glide_secs;
    }
}

impl<T> Oscillator for GlideSin <T> where T: Oscillator{
    fn new(sample_rate : u32) -> Self {
        GlideSin {
            osc : T::new(sample_rate),
            time_step : 1.0 / sample_rate as f32,
            glide_secs : 1.0,
            freq : 0.0
        }
    }

    fn step(&mut self, target_freq : f32) -> f32 {
        if self.freq == 0.0 {
            self.freq = target_freq
        } else if self.freq != target_freq {
            self.freq += (target_freq - self.freq) * (self.time_step / self.glide_secs)
        }

        self.osc.step(self.freq)
    }

   fn handle_input(&mut self, _event : crate::controller_trait::InputEvent) {
        ();
    }
}

pub struct BitCrush <T> where T: Oscillator {
    osc : T,
    quantize_steps : u32,
}

impl<T> BitCrush <T> where T: Oscillator{
    pub fn set(&mut self,  quantize_steps : u32) {
        self.quantize_steps = quantize_steps;
    }
}

impl<T> Oscillator for BitCrush <T> where T: Oscillator{
    fn new(sample_rate : u32) -> Self {
        BitCrush {
            osc : T::new(sample_rate),
            quantize_steps : 66,
        }
    }

    fn step(&mut self, freq : f32) -> f32 {
        ((self.osc.step(freq) * self.quantize_steps as f32) as u32) as f32 / self.quantize_steps as f32
    }

   fn handle_input(&mut self, event : crate::controller_trait::InputEvent) {
        match event {
            crate::controller_trait::InputEvent::Continuous(crate::controller_trait::ContinuousType::LeftTrigger, v) => self.set(66 - (v * 64.0) as u32),
            _ => ()
        };
   }
}

/*
delay / echo
detuned oscillators / detuned pairs (detune fm params?)
live envelope change
lfos to hook into other params
change chord voicing with touchpad
*/