use hidapi::HidDevice;


#[derive(Copy, Clone, Debug)]
pub enum InputEvent {
    Discrete(DiscreteType, i8),
    Continuous(ContinuousType, f32),
    Button(ButtonType, bool),
}

#[derive(Copy, Clone, Debug)]
pub enum DiscreteType {
    Left,
    Right,
    Dpad,
    TouchX,
    TouchY,
}

#[derive(Copy, Clone, Debug)]
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