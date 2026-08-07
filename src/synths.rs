use crate::{chord_engine::ChordEngine, sound_engine::SoundEngine};

const VOLUME : f32 = 0.4;

pub fn get_process(mut sound_engine : SoundEngine) -> impl FnMut(f32, f32, f32) -> f32 {
    {
        move |_: f32, _: f32, _: f32| -> f32 {
            sound_engine.handle_input();
            // let freqs: Vec<f32> = sound_engine.current_chord.iter().map(|n| {ChordEngine::value_to_freq(*n)}).collect();
            let mut out : f32 = 0.0;
            for voice in &mut sound_engine.voices {
                let Some(note_info) = &voice.note_info else {
                    voice.phase = 0.0;
                    continue;
                };
                voice.env.step();
                voice.phase += ChordEngine::value_to_freq(note_info.note) * sound_engine.time_step;
                voice.phase %= 1.0;
                // out += (voice.phase * 2.0 * std::f32::consts::PI).sin() * voice.env.level;
                out += (voice.phase*2.0 - 1.0) * voice.env.level;
                // out += (if voice.phase > 0.5 {1.0} else {-1.0}) * coefficient;
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
                self.level = (self.level + self.atime_secs / self.sample_rate as f32).min(1.0);
                if self.level >= 1.0 {
                    self.phase = AdsrPhase::Decay;
                    println!("hi")
                }
            },
            AdsrPhase::Decay => {
                self.level = (self.level - self.dtime_secs / self.sample_rate as f32).max(self.slevel);
                if self.level <= self.slevel {
                    self.phase = AdsrPhase::Sustain;
                    println!("bye")
                }
            },
            AdsrPhase::Sustain => {
                self.level = self.slevel;
            },
            AdsrPhase::Release => {
                self.level = (self.level - self.rtime_secs / self.sample_rate as f32).max(0.0);
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