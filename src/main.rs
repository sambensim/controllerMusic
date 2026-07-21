use nannou::prelude::*;

fn main() {
    nannou::app(model).simple_window(view).run();
}

struct Model {}

fn model(_app: &App) -> Model {
    Model {}
}

fn view(app: &App, _model: &Model, _window: Entity) {
    let draw = app.draw();
    draw.background().color(BLACK);

    draw.text("sine wave")
        .color(WHITE)
        .font_size(48)
        .font("TAY High Beams");   // the family name as a string
}