use std::f32::consts::PI;

use crate::instrument::ParamInfo;

pub fn get_next_phase(phase : f32, time_step : f32, freq : f32) -> f32 {
    (phase + freq * time_step) % 1.0
}
pub trait Oscillator : Send {
    fn step(&mut self, freq : f32) -> f32;

    fn params(&self) -> &'static [ParamInfo];

    fn set_param(&mut self, index : usize, value : f32);
}

macro_rules! osc_params { //AI
    ($ty:ty { $($name:literal => $field:ident [$min:expr, $max:expr, $default:expr]),* $(,)? }) => {
        impl $ty {
            pub const PARAMS: &'static [ParamInfo] = &[
                $(ParamInfo { name: $name, min: $min, max: $max, default: $default },)*
            ];

            fn set_indexed(&mut self, index: usize, value: f32) {
                let mut i = 0usize;
                $(
                    if index == i { self.$field = value; return; }
                    i += 1;
                )*
                let _ = i;
                debug_assert!(false, "param index {} out of range", index);
            }
        }
    };
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
osc_params!(Saw {});

impl Oscillator for Saw {
    fn step(&mut self, freq : f32) -> f32 {
        self.phase = get_next_phase(self.phase, self.time_step, freq);
        self.phase*2.0 - 1.0
    }

    fn params(&self) -> &'static [ParamInfo] {
        Self::PARAMS
    }

    fn set_param(&mut self, index: usize, value: f32) {
        self.set_indexed(index, value)
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
osc_params!(Sin {});

impl Oscillator for Sin {
    fn step(&mut self, freq : f32) -> f32 {
        self.phase = get_next_phase(self.phase, self.time_step, freq);
        (self.phase*2.0*PI).sin()
    }

    fn params(&self) -> &'static [ParamInfo] {
        Self::PARAMS
    }

    fn set_param(&mut self, index: usize, value: f32) {
        self.set_indexed(index, value)
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
    pub fn new(sample_rate : u32, oscillator_factory: impl Fn(u32) -> Box<dyn Oscillator>) -> Self {
        Fm {
            carrier : oscillator_factory(sample_rate),
            modulator_phase : 0.0,
            time_step : 1.0 / sample_rate as f32,
            ratio : 0.0,
            modulation_amplitude : 0.0,
            target_modulation_amplitude : 0.0,
        }
    }
}
osc_params!(Fm {
    "ratio"      => ratio [0.0, 4.0,  0.5],
    "amplitude"  => target_modulation_amplitude [0.0, PI, 0.5],
});

impl Oscillator for Fm {
    fn step(&mut self, freq : f32) -> f32 {
        self.modulation_amplitude += (self.target_modulation_amplitude - self.modulation_amplitude) * 0.001; //0.001 == smoothing amt
        self.modulator_phase = get_next_phase(self.modulator_phase, self.time_step, self.ratio * freq);
        let modulator_sin = self.modulation_amplitude * (self.modulator_phase * 2.0 * PI).sin();

        (self.carrier.step(freq) + modulator_sin).sin()
    }

    fn params(&self) -> &'static [ParamInfo] {
        Self::PARAMS
    }

    fn set_param(&mut self, index: usize, value: f32) {
        self.set_indexed(index, value)
    }
}

// pub struct GlideSin {
//     osc : Box<dyn Oscillator>,
//     time_step : f32,
//     freq : f32,
//     glide_secs : f32,
// }

// impl GlideSin {
//     pub fn new(sample_rate : u32, oscillator_factory: impl Fn(u32) -> Box<dyn Oscillator>, glide_secs : f32) -> Self {
//         GlideSin {
//             osc : oscillator_factory(sample_rate),
//             time_step : 1.0 / sample_rate as f32,
//             glide_secs : glide_secs,
//             freq : 0.0
//         }
//     }

//     pub fn set(&mut self,  glide_secs : f32) {
//         self.glide_secs = glide_secs;
//     }
// }

// impl Oscillator for GlideSin {

//     fn step(&mut self, target_freq : f32) -> f32 {
//         if self.freq == 0.0 {
//             self.freq = target_freq
//         } else if self.freq != target_freq {
//             self.freq += (target_freq - self.freq) * (self.time_step / self.glide_secs)
//         }

//         self.osc.step(self.freq)
//     }
// }


// /*
// delay / echo
// detuned oscillators / detuned pairs (detune fm params?)
// live envelope change
// lfos to hook into other params
// change chord voicing with touchpad
// */