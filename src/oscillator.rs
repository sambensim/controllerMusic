use std::f32::consts::PI;

use crate::{sound_engine::SoundEngine};

const VOLUME : f32 = 1.0;

pub fn get_process(mut sound_engine : SoundEngine) -> impl FnMut(f32, f32, f32) -> f32 {
    move |_: f32, _: f32, _: f32| -> f32 {
        sound_engine.handle_input();
        let mut out : f32 = 0.0;
        for instrument in &mut sound_engine.instruments {
            out += instrument.step();
        };
        out *= 0.5_f32.sqrt().powi(sound_engine.instruments.len().max(1) as i32); //TODO, set by active instead of all
        sound_engine.send(out * VOLUME)
    }
}

pub fn get_next_phase(phase : f32, time_step : f32, freq : f32) -> f32 {
    (phase + freq * time_step) % 1.0
}
pub trait Oscillator : Send {
    fn step(&mut self, freq : f32) -> f32;
}

pub struct Saw {
    phase : f32,
    time_step : f32,
}

impl Saw {
    pub fn new(sample_rate : u32) -> Self {
        Saw {
            phase : 0.0,
            time_step : 1.0 / sample_rate as f32,
        }
    }
}

impl Oscillator for Saw {
    fn step(&mut self, freq : f32) -> f32 {
        self.phase = get_next_phase(self.phase, self.time_step, freq);
        self.phase*2.0 - 1.0
    }
}

pub struct Sin {
    phase : f32,
    time_step : f32,
}

impl Sin {
    pub fn new(sample_rate : u32) -> Self {
        Sin {
            phase : 0.0,
            time_step : 1.0 / sample_rate as f32,
        }
    }
}
impl Oscillator for Sin {
    fn step(&mut self, freq : f32) -> f32 {
        self.phase = get_next_phase(self.phase, self.time_step, freq);
        (self.phase*2.0*PI).sin()
    }
}

pub struct Fm {
    carrier : Box<dyn Oscillator>,
    modulator_phase : f32,
    time_step : f32,
    ratio : f32,
    modulation_amplitude : f32,
    target_modulation_amplitude : f32,
}

impl Fm {
    pub fn new(sample_rate : u32, oscillator_factory: impl Fn(u32) -> Box<dyn Oscillator>, ratio : f32, amplitude : f32) -> Self {
        Fm {
            carrier : oscillator_factory(sample_rate),
            modulator_phase : 0.0,
            time_step : 1.0 / sample_rate as f32,
            ratio : ratio,
            modulation_amplitude : amplitude,
            target_modulation_amplitude : amplitude,
        }
    }
    pub fn set(&mut self, ratio : f32, modulation_amplitude : f32) {
        self.ratio = ratio;
        self.target_modulation_amplitude = modulation_amplitude;
    }
}

impl Oscillator for Fm {
    fn step(&mut self, freq : f32) -> f32 {
        self.modulation_amplitude += (self.target_modulation_amplitude - self.modulation_amplitude) * 0.001; //0.001 == smoothing amt
        self.modulator_phase = get_next_phase(self.modulator_phase, self.time_step, self.ratio * freq);
        let modulator_sin = self.modulation_amplitude * (self.modulator_phase * 2.0 * PI).sin();

        (self.carrier.step(freq) + modulator_sin).sin()
    }
}
//    fn handle_input(&mut self, event : crate::controller::InputEvent) {
//         match event {
//             crate::controller::InputEvent::Continuous(crate::controller::ContinuousType::LeftTrigger, v) => self.set(self.ratio, v * 4.0),
//             crate::controller::InputEvent::Button(crate::controller::ButtonType::RBumper, true) => {
//                 let n = 1.0;//[0.25, 0.5, 1.0][fastrand::usize(0..3)];
//                 println!("{n}");
//                 self.set( n, self.modulation_amplitude)
//             },
//             _ => ()
//         };
//         self.carrier.handle_input(event);
//     }

// pub struct GlideSin <T> where T: Oscillator {
//     osc : T,
//     time_step : f32,
//     freq : f32,
//     glide_secs : f32,
// }

// impl<T> GlideSin <T> where T: Oscillator{
//     pub fn set(&mut self,  glide_secs : f32) {
//         self.glide_secs = glide_secs;
//     }
// }

// impl<T> Oscillator for GlideSin <T> where T: Oscillator{
//     fn new(sample_rate : u32) -> Self {
//         GlideSin {
//             osc : T::new(sample_rate),
//             time_step : 1.0 / sample_rate as f32,
//             glide_secs : 1.0,
//             freq : 0.0
//         }
//     }

//     fn step(&mut self, target_freq : f32) -> f32 {
//         if self.freq == 0.0 {
//             self.freq = target_freq
//         } else if self.freq != target_freq {
//             self.freq += (target_freq - self.freq) * (self.time_step / self.glide_secs)
//         }

//         self.osc.step(self.freq)
//     }

//    fn handle_input(&mut self, event : crate::controller::InputEvent) {
//         self.osc.handle_input(event);
//     }
// }

// pub struct BitCrush <T> where T: Oscillator {
//     osc : T,
//     quantize_steps : u32,
// }

// impl<T> BitCrush <T> where T: Oscillator{
//     pub fn set(&mut self,  quantize_steps : u32) {
//         self.quantize_steps = quantize_steps;
//     }
// }

// impl<T> Oscillator for BitCrush <T> where T: Oscillator{
//     fn new(sample_rate : u32) -> Self {
//         BitCrush {
//             osc : T::new(sample_rate),
//             quantize_steps : 66,
//         }
//     }

//     fn step(&mut self, freq : f32) -> f32 {
//         ((self.osc.step(freq) * self.quantize_steps as f32) as u32) as f32 / self.quantize_steps as f32
//     }

//    fn handle_input(&mut self, event : crate::controller::InputEvent) {
//         self.osc.handle_input(event);
//         match event {
//             crate::controller::InputEvent::Continuous(crate::controller::ContinuousType::RightTrigger, v) => self.set(66 - (v * 64.0) as u32),
//             _ => ()
//         };
//    }
// }


// pub struct Gain <T> where T: Oscillator {
//     osc : T,
//     coefficient : f32,
// }

// impl<T> Gain <T> where T: Oscillator{
//     pub fn set(&mut self,  coefficient : f32) {
//         self.coefficient = coefficient;
//     }
// }

// impl<T> Oscillator for Gain <T> where T: Oscillator{
//     fn new(sample_rate : u32) -> Self {
//         Gain {
//             osc : T::new(sample_rate),
//             coefficient : 1.0,
//         }
//     }

//     fn step(&mut self, freq : f32) -> f32 {
//         self.osc.step(freq) * self.coefficient
//     }

//    fn handle_input(&mut self, event : crate::controller::InputEvent) {
//         self.osc.handle_input(event);
//    }
// }

// /*
// delay / echo
// detuned oscillators / detuned pairs (detune fm params?)
// live envelope change
// lfos to hook into other params
// change chord voicing with touchpad
// */