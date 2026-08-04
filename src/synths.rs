use crate::sound_engine::SoundEngine;

pub fn get_process(mut sound_engine : SoundEngine) -> impl FnMut(f32, f32, f32) -> f32 {
    {
        move |_: f32, _: f32, _: f32| -> f32 {
            let freqs = sound_engine.get_chord();
            let mut out : f32 = 0.0;
            for i in 0..SoundEngine::MAX_NOTES {
                if freqs.get(i).is_none() {
                    sound_engine.voices[i].phase = 0.0
                } else {
                    sound_engine.voices[i].phase += freqs[i] * sound_engine.time_step;
                    sound_engine.voices[i].phase %= 1.0;
                    // out += (sound_engine.voices[i].phase * 2.0 * std::f32::consts::PI).sin();
                    out += sound_engine.voices[i].phase*2.0 - 1.0;
                    // out += if (sound_engine.voices[i].phase > 0.5) {1.0} else {-1.0};
                }
            };
            out /= freqs.len().max(1) as f32;
            sound_engine.send(out)
        }
    }
}