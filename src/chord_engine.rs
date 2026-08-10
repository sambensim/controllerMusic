use std::{collections::HashMap, vec};

const ALL_NOTES: [&str; 12] = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];
const MAJOR_INTERVALS: [u8; 7] = [0, 2, 4, 5, 7, 9, 11];

pub fn chord_type_dict() -> &'static HashMap<&'static str, Vec<u8>> {
    static DICT: std::sync::OnceLock<HashMap<&'static str, Vec<u8>>> = std::sync::OnceLock::new();
    DICT.get_or_init(|| {
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
    })
}

pub fn chord_mode_dict() -> &'static HashMap<i32, Vec<&'static str>> {
    static DICT: std::sync::OnceLock<HashMap<i32, Vec<&'static str>>> = std::sync::OnceLock::new();
    DICT.get_or_init(|| {
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
    })
}

pub struct ChordEngine {
    key: u8,
    octave: u8,
    key_notes: Vec<String>,
}

impl ChordEngine {
    pub fn new(key: u8, octave: u8) -> Self {
        ChordEngine {
            key,
            octave,
            key_notes: ChordEngine::get_key_notes(key),
        }
    }

    pub fn set_key(&mut self, key: u8) {
        self.key = key;
        self.key_notes = ChordEngine::get_key_notes(key);
    }

    pub fn decrement_key(&mut self) {
        self.set_key((self.key + (12 - 1)) % 12); 
    }

    pub fn increment_key(&mut self) {
        self.set_key((self.key + 1) % 12); 
    }

    pub fn get_key(&self) -> String {
        ALL_NOTES[self.key as usize].to_string()
    }

    pub fn get_key_value(&self) -> u8 {
        self.key
    }

    pub fn set_octave(&mut self, octave: u8) {
        self.octave = octave;
    }

    pub fn decrement_octave(&mut self) {
        self.set_octave((self.octave + (9 - 1)) % 9); 
    }

    pub fn increment_octave(&mut self) {
        self.set_octave((self.octave + 1) % 9); 
    }

     pub fn get_octave(&self) -> u8 {
        self.octave
    }

    pub fn get_key_notes(key : u8) -> Vec<String> {
        return MAJOR_INTERVALS
            .iter()
            .map(|&offset| ALL_NOTES[((key + offset) % 12) as usize].to_string())
            .collect()
    }

    pub fn note_add(value: u8, diff: u8) -> String {
        ChordEngine::value_to_note(value + diff).unwrap()
    }

    fn key_note_all_note_index(key_note_index: usize) -> u8 {
        MAJOR_INTERVALS[key_note_index]
    }

    pub fn get_chord_notes(&self, chord_index: i32, chord_mode: i32) -> Vec<String> {
        let mode = chord_mode_dict().get(&chord_mode).expect("unknown chord mode");
        let chord_mode_str = mode[(chord_index % 7) as usize];
        let octave_shift = 12 * (chord_index / 7) as u8;
        let root_offset = Self::key_note_all_note_index((chord_index % 7) as usize) + octave_shift;

        chord_type_dict()
            .get(chord_mode_str)
            .expect("unknown chord type")
            .iter()
            .map(|&offset| ChordEngine::note_add(self.key+12*self.octave, root_offset + offset))
            .collect()
    }

    pub fn get_chord_name(key : u8, chord_index: i32, chord_mode: i32) -> String {
        let note = ChordEngine::note_add(
            key,
            Self::key_note_all_note_index((chord_index % 7) as usize),
        );
        let mode = chord_mode_dict().get(&chord_mode).expect("unknown chord mode");
        let chord_mode_str = mode[(chord_index % 7) as usize];
        format!("{}{}", note, chord_mode_str)
    }

    pub fn value_to_freq(value: u8) -> f32 {
        440.0 * 2.0_f32.powf((value as f32 - 69.0) / 12.0)
    }

    pub fn _note_to_freq(note: &str) -> Result<f32, String> {
        Ok(Self::value_to_freq(Self::note_to_value(note)?))
    }

    pub fn value_to_note(value: u8) -> Result<String, String> {
        let value = value as i32;
        let octave = value / 12;
        let note_index = (value % 12) as usize;
        let name = ALL_NOTES[note_index];
        if octave > 10 {
            return Err("bad octave".to_string());
        }
        Ok(format!("{name}{octave}"))
    }

    pub fn note_to_value(note: &str) -> Result<u8, String> {
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

        Ok((octave * 12 + note_index as i32) as u8)
    }
}