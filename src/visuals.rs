use nannou::prelude::*;

struct Model {
}

fn main() {
    nannou::app(model).simple_window(view).run();
}

fn model(_app: &App) -> Model {
    Model {}
}

fn view(app: &App, _model: &Model, _window: Entity) {
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
    let frequency = 0.02;
    let speed = 2.0;

    let points = (0..=win.w() as i32).map(|i| {
        let x = win.left() + i as f32;
        // let y = (x * frequency).sin() * amplitude * (t * speed).sin();
        let y = (x * frequency + t * speed).sin() * amplitude;
        pt2(x, y)
    });

    draw.polyline()
        .weight(3.0)
        .color(WHITE)
        .points(points);
}