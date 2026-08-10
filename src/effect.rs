pub trait Effect : Send {
    fn step(&mut self, samp : f32) -> f32;
}

pub struct Noise {
    mix : f32,
}

impl Noise {
    pub fn new(mix : f32) -> Self {
        Noise {
            mix : mix,
        }
    }

    pub fn set(&mut self,  mix : f32) {
        self.mix = mix;
    }
}

impl Effect for Noise {
    fn step(&mut self, samp : f32) -> f32 {
        samp * (1.0 - self.mix) + self.mix * (fastrand::f32_inclusive() * 2.0 - 1.0)
    }
}