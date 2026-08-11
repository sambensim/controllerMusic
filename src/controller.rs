use hidapi::HidDevice;

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum InputSource {
    Discrete(DiscreteType),
    Continuous(ContinuousType),
    Button(ButtonType),
}

#[derive(Copy, Clone, Debug)]
pub enum InputEvent {
    Discrete(DiscreteType, i8, u8),
    Continuous(ContinuousType, f32),
    Button(ButtonType, bool),
}

impl InputEvent {
    pub fn split(&self) -> (InputSource, f32) {
        match self {
            InputEvent::Discrete(t, v, r) => (InputSource::Discrete(*t), (*v as f32) / (*r as f32)),
            InputEvent::Continuous(t, v) => (InputSource::Continuous(*t), *v),
            InputEvent::Button(t, b) => (InputSource::Button(*t), if *b {1.0} else {0.0}),
        }
    }
}

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum DiscreteType {
    Left,
    Right,
    Dpad,
    TouchX,
    TouchY,
}

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum ContinuousType {
    LeftTrigger,
    RightTrigger,
}

macro_rules! buttons { //AI
    ($($name:ident),* $(,)?) => {
        #[repr(u8)]
        #[derive(Copy, Clone, Debug, PartialEq, Eq)]
        pub enum ButtonType { $($name),* }

        pub const BUTTONS: &[ButtonType] = &[$(ButtonType::$name),*];
    };
}

buttons!(
    Square, Cross, Circle, Triangle,
    LBumper, RBumper, LTriggerBtn, RTriggerBtn,
    Share, Options, LStickBtn, RStickBtn,
    PS, TouchpadClick, Touch
);

pub trait Controller: Default + Copy + Clone + Send + 'static{
    fn get_controller() -> Result<HidDevice, String>;

    fn parse_report(buf: &[u8]) -> Option<crate::intermediate_controller_state::IntermediateControllerState>;
}