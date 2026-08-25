use std::collections::{VecDeque, vec_deque};

use crate::{parameters::{Parameterized}, params};


pub trait Effect : Parameterized + Send {
    fn step(&mut self, samp : f32) -> f32;
}

pub struct Noise {
    mix : f32,
}

impl Noise {
    pub fn new() -> Self {
        Noise {
            mix : 0.0,
        }
    }
}

params!(Noise {
    "mix"      => mix      [0.0, 1.0,  0.02],
});

impl Effect for Noise {
    fn step(&mut self, samp : f32) -> f32 {
        if samp != 0.0 {
            return samp * (1.0 - self.mix) + self.mix * (fastrand::f32_inclusive() * 2.0 - 1.0)
        }
        0.0
    }
}


pub struct Delay {
    mix : f32,
    buffer : VecDeque<f32>,
}

impl Delay {
    pub fn new(sample_rate : u32, time_secs: f32) -> Self {
        Delay {
            mix : 0.0,
            buffer : vec_deque::VecDeque::from(vec![0.0; (sample_rate as f32 * time_secs) as usize])
        }
    }
}
params!(Delay {
    "mix"      => mix      [0.0, 1.0,  0.2],
});

impl Effect for Delay {
    fn step(&mut self, samp : f32) -> f32 {
        self.buffer.push_back(samp);
        samp * (1.0 - self.mix) + self.mix * self.buffer.pop_front().unwrap()
    }
}

pub struct Gain {
    amount : f32,
}

impl Gain {
    pub fn new() -> Self {
        Gain {
            amount : 0.0,
        }
    }
}
params!(Gain {
    "amount"      => amount      [0.0, 1.0,  1.0],
});

impl Effect for Gain {
    fn step(&mut self, samp : f32) -> f32 {
        samp * self.amount
    }
}

// pub struct BitCrush {
//     mix : f32,
//     quantize_steps : u32,
// }

// impl BitCrush {
//     pub fn new(mix : f32, quantize_steps : u32) -> Self {
//         BitCrush {
//             mix : mix,
//             quantize_steps : quantize_steps,
//         }
//     }

//     pub fn set(&mut self, mix : f32) {
//         self.mix = mix;
//     }
// }

// impl Effect for BitCrush {
//     fn step(&mut self, samp : f32) -> f32 {
//         samp * (1.0 - self.mix) + self.mix * ((((((samp + 1.0) / 2.0) * self.quantize_steps as f32) as u32) as f32 / self.quantize_steps as f32) * 2.0 - 1.0)
//     }
// }