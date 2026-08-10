use crate::{adsr::Adsr, effect::Effect, input_engine::FullInputEvent, oscillator::Oscillator, voice_manager::{VoiceManager, Voicebank}};

pub struct Instrument {
    voicebank : Voicebank,
    post_processing : Vec<Box<dyn Effect>>,
}

impl Instrument {
    pub fn new(sample_rate: u32, oscillator_factory: impl Fn(u32) -> Box<dyn Oscillator>, envelope_factory: impl Fn(u32) -> Adsr, voice_manager : &mut VoiceManager, effects : Vec<Box<dyn Effect>>, polyphony : usize) -> Self {
        Instrument {
            voicebank: voice_manager.request_voicebank(sample_rate, polyphony, oscillator_factory, envelope_factory).unwrap(),
            post_processing: effects,
        }
    }

    pub fn step(&mut self) -> f32 {
        let mut out = self.voicebank.step();
        for e in &mut self.post_processing {
            out = e.step(out);
        }
        out
    }

    pub fn handle_input(&mut self, event : &FullInputEvent) {
        // todo!()
    }

    pub fn play(&mut self, note : u8) {
        self.voicebank.play(note);
    }

    pub fn release(&mut self, note : u8) {
        self.voicebank.release(note);
    }
}

// pub struct Saw4 {
//     voicebank : [Voice; Self::POLYPHONY]
// }

// impl Saw4 {
//     const POLYPHONY : usize = 4;

//     fn new(voice_manager : &mut VoiceManager) -> Self {
//         let voicebank : [Voice; Self::POLYPHONY] = std::array::from_fn(|_| voice_manager.request_voice().unwrap());
//         Saw4{
//             voicebank : voicebank
//         }
//     }
// }

// impl Instrument for Saw4 {
//     fn step(&mut self) -> f32 {
//         todo!()
//     }

//     fn handle_input(&mut self, event : &FullInputEvent) {
//         todo!()
//     }

//     fn play(&mut self, note : u8) {
//         todo!()
//     }

//     fn release(&mut self, note : u8) {
//         todo!()
//     }
// }