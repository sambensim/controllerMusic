use std::sync::mpsc::Receiver;
use raylib::drawing::RaylibDrawHandle;
use raylib::prelude::*;

use crate::chord_engine::{ChordEngine, chord_type_dict};
use crate::display_engine::DisplayEngine;
use crate::input_engine::{InputEngine, InputEvent};

pub fn run(samp_channel : Receiver<f32>, controller_channel : tokio::sync::broadcast::Receiver<InputEvent>, mut input_engine : InputEngine) {

    let mut display_engine : DisplayEngine = DisplayEngine::init(controller_channel, samp_channel);

    let (mut rl, thread) = raylib::init()
        .size(DisplayEngine::WIDTH as i32, DisplayEngine::HEIGHT as i32)
        .title("Hello, World")
        .build();
    
    while !rl.window_should_close() {
        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::WHITE);
        visual_loop(d, &mut display_engine, &mut input_engine);
    }
}

fn visual_loop(mut d : RaylibDrawHandle, display_engine : &mut DisplayEngine, input_engine : &mut InputEngine) {
    input_engine.step();
    display_engine.handle_input();
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
    let center = Vector2::new(DisplayEngine::WIDTH / 12.0 * 9.0, DisplayEngine::HEIGHT / 6.0);
    let r = DisplayEngine::HEIGHT / 6.0;
    for i in 0..8 {
        let pos = center + Vector2::new(r * (2.0 * PI as f32 * i as f32 / 8.0).cos(), r * (2.0 * PI as f32 * i as f32 / 8.0).sin());
        d.draw_text(&notes[i%7], pos.x as i32, pos.y as i32, 12, Color::BLACK);
    }
}