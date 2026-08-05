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
                let decay_ms = 500.0;
                let coefficient = if note_info.release.is_none() { 1.0 } else {(1.0 - note_info.release.unwrap().elapsed().as_millis() as f32/decay_ms).max(0.0)};
                voice.phase += ChordEngine::value_to_freq(note_info.note) * sound_engine.time_step;
                voice.phase %= 1.0;
                // out += (voice.phase * 2.0 * std::f32::consts::PI).sin() * coefficient;
                out += (voice.phase*2.0 - 1.0) * coefficient;
                // out += (if voice.phase > 0.5 {1.0} else {-1.0}) * coefficient;
            };
            out /= sound_engine.voices.len().max(1) as f32;
            sound_engine.send(out * VOLUME)
        }
    }
}