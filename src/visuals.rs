use std::sync::mpsc::Receiver;
use raylib::drawing::RaylibDrawHandle;
use raylib::prelude::*;

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
    let points: Vec<Vector2> = display_engine.get_samples().iter().enumerate().map(|(i, samp)| Vector2 { x: DisplayEngine::WIDTH - (i as f32)/(DisplayEngine::SAMPLE_CAPACITY as f32)*DisplayEngine::WIDTH, y: *samp*(DisplayEngine::HEIGHT/4.0) + DisplayEngine::HEIGHT/2.0}).collect();
    d.draw_line_strip(&points,Color::BLACK);

    let text = display_engine.get_chord();
    d.draw_text(&text, 12, 12, 20, Color::BLACK);
}