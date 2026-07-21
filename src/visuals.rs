use nannou::prelude::*;

struct Model {}

fn main() {
    nannou::app(model)
        .simple_window(view)
        .run();
}

fn model(_app: &App) -> Model {
    Model {}
}

fn view(_app: &App, _model: &Model, _window: Entity) {
}