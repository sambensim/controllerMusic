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
