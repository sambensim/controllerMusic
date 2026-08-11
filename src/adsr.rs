use crate::params;

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
    pub fn new(sample_rate : u32) -> Adsr {
        Adsr {
            atime_secs : 0.0,
            dtime_secs : 0.0,
            slevel : 0.0,
            rtime_secs : 0.0,
            phase : AdsrPhase::Release,
            level : 0.0,
            sample_rate : sample_rate,
        }
    }

    pub fn step(&mut self) -> f32{
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
        self.level
    }

    pub fn trigger(&mut self) {
        self.phase = AdsrPhase::Attack
    }

    pub fn release(&mut self) {
        self.phase = AdsrPhase::Release
    }
}

params!(Adsr {
    "attack" => atime_secs [0.0, 60.0, 0.3],
    "decay" => dtime_secs [0.0, 60.0, 0.5],
    "sustain" => slevel [0.0, 1.0, 0.6],
    "release" => rtime_secs [0.0, 60.0, 0.8],
});