use std::collections::VecDeque;
use std::sync::Mutex;
use nannou::prelude::*;

static UPDATE_VISUAL: Mutex<Option<Box<dyn FnMut(&mut VecDeque<f32>) + Send + 'static>>> = Mutex::new(None);

pub struct Model {
    _window: Entity,
    display_buffer: VecDeque<f32>,
    update_visual: Mutex<Box<dyn FnMut(&mut VecDeque<f32>) + Send + 'static>>,
}

fn model(app: &App) -> Model {
    let _window = app.new_window().view(view).build();
    let update_visual = UPDATE_VISUAL.lock().unwrap().take().unwrap();
    Model {
        _window,
        display_buffer: VecDeque::new(),
        update_visual: Mutex::new(update_visual),
    }
}


fn update(_app: &App, model: &mut Model) {
    let mut f = model.update_visual.lock().unwrap();
    (*f)(&mut model.display_buffer);
}

fn view(app: &App, model: &Model) {
    let draw = app.draw();
    draw.background().color(BLACK);

    let win = app.window_rect();
    let t = app.time();

    draw.text("sine wave")
        .color(WHITE)
        .font_size(48)
        .font("TAY High Beams")
        .x_y(0.0, win.top() * 0.5);

    let amplitude = 80.0;
    let num_samples = model.display_buffer.len();
    // let frequency = 0.02;
    // let speed = 2.0;

    if num_samples > 0 {
        let points = model.display_buffer.iter().enumerate().map(|(i, &sample)| {
            let x = win.left() + (i as f32 / num_samples as f32) * win.w();
            let y = sample * amplitude;
            pt2(x, y)
        });

        draw.polyline()
            .weight(3.0)
            .color(WHITE)
            .points(points);
    }
}

pub fn run(update_visual: impl FnMut(&mut VecDeque<f32>) + Send + 'static) {
    *UPDATE_VISUAL.lock().unwrap() = Some(Box::new(update_visual));
    // nannou::app(model).simple_window(view).run();
    nannou::app(model).update(update).run();
}