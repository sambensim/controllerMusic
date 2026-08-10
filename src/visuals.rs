use raylib::drawing::RaylibDrawHandle;
use raylib::prelude::*;

use crate::display_engine::DisplayEngine;
use crate::display_engine::Visual;

pub fn handle_visuals(mut d : RaylibDrawHandle, display_engine : &mut DisplayEngine) {
    match display_engine.current_visual {
       Visual::Setup => visual_setup(d, display_engine),
       Visual::Main => visual_main(d, display_engine),
    }
}

fn visual_setup(mut d : RaylibDrawHandle, display_engine : &mut DisplayEngine) {
}

fn visual_main(mut d : RaylibDrawHandle, display_engine : &mut DisplayEngine) {
    let points: Vec<Vector2> = display_engine.get_samples().iter().enumerate().map(|(i, samp)| Vector2 { x: DisplayEngine::WIDTH - (i as f32)/(DisplayEngine::SAMPLE_CAPACITY as f32)*DisplayEngine::WIDTH, y: *samp*(DisplayEngine::HEIGHT/4.0) + DisplayEngine::HEIGHT/2.0}).collect();
    d.draw_line_strip(&points,Color::BLACK);

    let text = format!(
        "playing: {}\nselected: {}\nkey: {}\noctave: {}",
        &display_engine.current_chord,
        &display_engine.selected_chord,
        &display_engine.get_key(),
        &display_engine.get_octave(),
    );
    d.draw_text(&text, 12, 12, 20, Color::BLACK);

    let notes = display_engine.get_key_notes();
    let center = Vector2::new(DisplayEngine::WIDTH / 12.0 * 5.0, DisplayEngine::HEIGHT / 6.0);
    let r = DisplayEngine::HEIGHT / 8.0;
    for i in 0..8 {
        let pos = center + Vector2::new(r * (2.0 * PI as f32 * i as f32 / 8.0).cos(), r * (2.0 * PI as f32 * i as f32 / 8.0).sin());
        d.draw_text(&notes[i%7], pos.x as i32, pos.y as i32, 12, Color::BLACK);
    }

     let modes = ["flip7", "add9", "sus4", "sus2", "dim", "aug", "flip", "dom7"];
    let center = Vector2::new(DisplayEngine::WIDTH / 12.0 * 9.0, DisplayEngine::HEIGHT / 6.0);
    let r = DisplayEngine::HEIGHT / 8.0;
    for i in 0..8 {
        let pos = center + Vector2::new(r * (2.0 * PI as f32 * i as f32 / 8.0).cos(), r * (2.0 * PI as f32 * i as f32 / 8.0).sin());
        d.draw_text(modes[i], pos.x as i32, pos.y as i32, 12, Color::BLACK);
    }
}