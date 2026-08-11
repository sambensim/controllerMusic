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
     pub fn new(sample_rate : u32) -> Self {
        Voice {
            note_info : None,
            osc : Box::new(crate::oscillator::Saw::new(sample_rate)),
            env : crate::adsr::Adsr::new(sample_rate, 1.0, 3.0, 0.6, 1.0),
        }
    }

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
