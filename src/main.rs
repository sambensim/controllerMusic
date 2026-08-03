mod controller;
mod visuals;
mod sound;
mod music_theory;
mod synths;

use raylib::prelude::*;
use std::{collections::VecDeque, sync::mpsc::{self, Receiver}};

use crate::{controller::DS4State, music_theory::ChordEngine};


struct DisplayEngine {
    controller_state : DS4State,
    controller_channel : Receiver<DS4State>,
    chord_engine : music_theory::ChordEngine
}

impl DisplayEngine {
    fn get_state(&mut self) -> DS4State {
        let new_state = self.controller_channel.try_recv();
        if !new_state.is_err() {
            self.controller_state = new_state.unwrap();
        };
        self.controller_state
    }

    fn get_chord(&mut self) -> String {
        let state = self.get_state();
        let loct = controller::get_left_stick_section(&state);
        if loct == -1 {
            return "none".to_string();
        }
        let roct = controller::get_right_stick_section(&state);
        self.chord_engine.get_chord_name(loct as i32, roct as i32)
    }
}

fn main() {
    let dualshock = controller::get_dualshock().unwrap();
    let (controller_channel,  controller_channel2) = controller::start_controller_thread(dualshock);
    let samp_channel = sound::do_sound(controller_channel);
    const WIDTH : f32 = 640_f32;
    const HEIGHT : f32 = 480_f32;

    let (mut rl, thread) = raylib::init()
        .size(WIDTH as i32, HEIGHT as i32)
        .title("Hello, World")
        .build();

    let mut display_engine : DisplayEngine = DisplayEngine {
        controller_state : controller_channel2.recv().unwrap(),
        controller_channel : controller_channel2,
        chord_engine : music_theory::ChordEngine::new(0, 4),
    };



    const CAPACITY : usize = 1000;
    let mut draw_points : VecDeque<f32> = VecDeque::from([0.0_f32; CAPACITY]);
    
    while !rl.window_should_close() {
        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::WHITE);

        let mut samp = samp_channel.try_recv();
        while !samp.is_err() {
            draw_points.push_front(samp.unwrap());
            samp = samp_channel.try_recv();
        }
        draw_points.truncate(CAPACITY);
        let points: Vec<Vector2> = draw_points.iter().enumerate().map(|(i, samp)| Vector2 { x: WIDTH - (i as f32)/(CAPACITY as f32)*WIDTH, y: *samp*(HEIGHT/4.0) +HEIGHT/2.0}).collect();
        d.draw_line_strip(&points,Color::BLACK);

        let text = display_engine.get_chord();
        d.draw_text(&text, 12, 12, 20, Color::BLACK);
    }
}