use std::iter::Map;

use crate::{adsr::Adsr, oscillator::Oscillator, voice::Voice};

pub struct VoiceManager {
}

impl VoiceManager {
    pub fn new() -> Self {
        VoiceManager {  }
    }

    pub fn request_voicebank(&mut self, sample_rate : u32, amount : usize, oscillator_factory: impl Fn(u32) -> Box<dyn Oscillator>, envelope_factory : impl Fn(u32) -> Adsr) -> Result<Voicebank, String> {
        return Ok(Voicebank {
            voices : (0..amount).map(|_| Voice {
                note_info : None,
                osc : oscillator_factory(sample_rate),
                env : envelope_factory(sample_rate),
            }).collect()
        })
    }
}

pub struct Voicebank {
    pub voices : Vec<Voice>,
}

impl Voicebank {
    pub fn play(&mut self, note: u8) {
        let idx = self.get_voice_index();
        self.voices[idx].play(note);
    }

    pub fn release(&mut self, note: u8) {
        for v in self.voices.iter_mut() {
            if let Some(ni) = &v.note_info {
                if ni.note == note {
                    v.release()
                }
            }
        }
    }

    pub fn step(&mut self) -> f32 {
        let mut out = 0_f32;
        for v in self.voices.iter_mut() {
            out += v.step();
        }
        return out / (self.get_active() as f32);
    }

    fn get_active(&self) -> usize {
        self.voices.len()
        // todo!()
    }

    fn get_voice_index(&self) -> usize {
        // check for voice doing nothing
        for (i, v) in self.voices.iter().enumerate() {
            if v.note_info.is_none() {
                return i;
            }
        }
        // check for voice with a note no longer being held
        for (i, v) in self.voices.iter().enumerate() {
            if let Some(ni) = &v.note_info {
                if ni.release.is_some() {
                    return i;
                }
            }
        }
        println!("max voices exceeded");
        0
    }
}