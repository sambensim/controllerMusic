use std::collections::HashMap;

const ALL_NOTES: [&str; 12] = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];
const MAJOR_INTERVALS: [i32; 7] = [0, 2, 4, 5, 7, 9, 11];

fn chord_type_dict() -> HashMap<&'static str, Vec<i32>> {
    HashMap::from([
        ("maj",     vec![0, 4, 7]),
        ("min",     vec![0, 3, 7]),
        ("dim",     vec![0, 3, 6]),
        ("dom7",    vec![0, 4, 7, 10]),
        ("maj7",    vec![0, 4, 7, 11]),
        ("min7",    vec![0, 3, 7, 10]),
        ("min7b5",  vec![0, 3, 6, 10]),
        ("majadd9", vec![0, 4, 7, 14]),
        ("minadd9", vec![0, 3, 7, 14]),
        ("dimadd9", vec![0, 3, 6, 14]),
        ("sus4",    vec![0, 5, 7]),
        ("maj6",    vec![0, 4, 7, 9]),
        ("sus2",    vec![0, 2, 7]),
        ("aug",     vec![1, 4, 8]),
    ])
}

fn chord_mode_dict() -> HashMap<i32, Vec<&'static str>> {
    HashMap::from([
        (-1, vec!["maj",     "min",     "min",     "maj",     "maj",     "min",     "dim"    ]),
        ( 6, vec!["min",     "maj",     "maj",     "min",     "min",     "maj",     "min"    ]),
        ( 7, vec!["dom7",    "dom7",    "dom7",    "dom7",    "dom7",    "dom7",    "dom7"   ]),
        ( 0, vec!["min7",    "maj7",    "maj7",    "min7",    "min7",    "maj7",    "min7b5" ]),
        ( 1, vec!["majadd9", "minadd9", "minadd9", "majadd9", "majadd9", "minadd9", "dimadd9"]),
        ( 2, vec!["sus4",    "sus4",    "sus4",    "sus4",    "sus4",    "sus4",    "sus4"   ]),
        ( 3, vec!["maj6",    "sus2",    "sus2",    "maj6",    "maj6",    "sus2",    "sus2"   ]),
        ( 4, vec!["dim",     "dim",     "dim",     "dim",     "dim",     "dim",     "dim"    ]),
        ( 5, vec!["aug",     "aug",     "aug",     "aug",     "aug",     "aug",     "aug"    ]),
    ])
}

pub struct ChordEngine {
    key: i32,
    octave: i32,
    key_notes: Vec<String>,
    chord_types: HashMap<&'static str, Vec<i32>>,
    chord_modes: HashMap<i32, Vec<&'static str>>,
}

impl ChordEngine {
    pub fn new(key: i32, octave: i32) -> Self {
        let mut engine = ChordEngine {
            key,
            octave,
            key_notes: Vec::new(),
            chord_types: chord_type_dict(),
            chord_modes: chord_mode_dict(),
        };
        engine.rebuild_key_notes();
        engine
    }

    pub fn change_key(&mut self, new_key: i32) {
        self.key = new_key;
        self.rebuild_key_notes();
    }

    fn rebuild_key_notes(&mut self) {
        self.key_notes = MAJOR_INTERVALS
            .iter()
            .map(|&offset| ALL_NOTES[((self.key + offset) % 12) as usize].to_string())
            .collect();
    }

    fn note_add(&self, note: i32, diff: i32) -> String {
        let mut ni = note + diff;
        let mut bo = self.octave;
        while ni >= 12 {
            ni -= 12;
            bo += 1;
        }
        format!("{}{}", ALL_NOTES[ni as usize], bo)
    }

    fn key_note_all_note_index(key_note_index: usize) -> i32 {
        MAJOR_INTERVALS[key_note_index]
    }

    pub fn get_chord_notes(&self, chord_index: i32, chord_mode: i32) -> Vec<String> {
        let mode = self.chord_modes.get(&chord_mode).expect("unknown chord mode");
        let chord_mode_str = mode[(chord_index % 7) as usize];
        let octave_shift = 12 * (chord_index / 7);
        let root_offset = Self::key_note_all_note_index((chord_index % 7) as usize) + octave_shift;

        self.chord_types
            .get(chord_mode_str)
            .expect("unknown chord type")
            .iter()
            .map(|&offset| self.note_add(self.key, root_offset + offset))
            .collect()
    }

    pub fn get_chord_name(&self, chord_index: i32, chord_mode: i32) -> String {
        let octave_shift = 12 * (chord_index / 7);
        let note = self.note_add(
            self.key,
            Self::key_note_all_note_index((chord_index % 7) as usize) + octave_shift,
        );
        let mode = self.chord_modes.get(&chord_mode).expect("unknown chord mode");
        let chord_mode_str = mode[(chord_index % 7) as usize];
        format!("{}{}", note, chord_mode_str)
    }

    pub fn value_to_freq(value: i32) -> f32 {
        440.0 * 2.0_f32.powf((value - 69) as f32 / 12.0)
    }

    pub fn note_to_freq(note: &str) -> Result<f32, String> {
        Ok(Self::value_to_freq(Self::note_to_value(note)?))
    }

    pub fn note_to_value(note: &str) -> Result<i32, String> {
        // Split at the boundary between the note name and the octave number
        let split = note
            .find(|c: char| c == '-' || c.is_ascii_digit())
            .ok_or_else(|| format!("invalid note: {note}"))?;

        let name = &note[..split];
        let octave_str = &note[split..];
        let octave: i32 = octave_str
            .parse()
            .map_err(|_| format!("invalid octave in note: {note}"))?;

        let note_index = ALL_NOTES
            .iter()
            .position(|&n| n == name)
            .ok_or_else(|| format!("invalid note name: {name}"))?;

        Ok(octave * 12 + note_index as i32)
    }
}