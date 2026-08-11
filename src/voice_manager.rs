use crate::{adsr::Adsr, oscillator::Oscillator};

pub struct VoiceManager {
}

impl VoiceManager {
    pub fn new() -> Self {
        VoiceManager {  }
    }

    pub fn request_voicebank(&mut self, sample_rate : u32, amount : usize, oscillator_factory: impl Fn(u32) -> Box<dyn Oscillator>, envelope_factory : impl Fn(u32) -> Adsr) -> Result<Voicebank, String> {
        let mut out = Voicebank {
            voices : (0..amount).map(|_| Voice {
                note_info : None,
                osc : oscillator_factory(sample_rate),
                env : envelope_factory(sample_rate),
            }).collect()
        };
        for v in out.voices.iter_mut() {
            Self::seed_defaults(v.osc.as_mut());
        }
        Ok(out)
    }

    fn seed_defaults(osc: &mut dyn Oscillator) {
        for (i, info) in osc.params().iter().enumerate() {
            osc.set_param(i, info.default);
        }
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

    pub fn release_all(&mut self) {
        for v in self.voices.iter_mut() {
            if v.note_info.is_some() {
                v.release()
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

    pub fn set(&mut self, param_index : usize, value : f32) {
        for v in self.voices.iter_mut() {
            v.osc.set_param(param_index, value);
        }
    }

    pub fn params(&self) -> &'static [crate::parameters::ParamInfo] {
        self.voices[0].osc.params()
    }
}

pub struct NoteInfo {
    pub note : u8,
    pub release : Option<std::time::Instant>
}

pub struct Voice {
    pub note_info : Option<NoteInfo>,
    pub osc : Box<dyn crate::oscillator::Oscillator>,
    pub env : crate::adsr::Adsr,
}

impl Voice {
    pub fn release(&mut self) {
        self.env.release();
        self.note_info.as_mut().unwrap().release = Some(std::time::Instant::now());
    }

    pub fn play(&mut self, note : u8) {
        self.note_info = Some(NoteInfo {
            note: note, release: None,
        });
        self.env.trigger();
    }

    pub fn step(&mut self) -> f32 {
        let Some(note_info) = &self.note_info else {
            return 0.0;
        };
        self.env.step() * self.osc.step(crate::chord_engine::ChordEngine::value_to_freq(note_info.note))
    }
}
