use hidapi::HidDevice;


#[derive(Copy, Clone, Debug)]
pub enum InputEvent {
    Directional(DirectionalType, i8),
    Continuous(ContinuousType, f32),
    Button(ButtonType, bool),
    None,
}

#[derive(Copy, Clone, Debug)]
pub enum DirectionalType {
    Left,
    Right,
    Dpad,
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
);

pub trait Controller: Default + Copy + Clone + Send + 'static{
    fn get_controller() -> Result<HidDevice, String>;

    fn parse_report(buf: &[u8]) -> Option<crate::intermediate_controller_state::IntermediateControllerState>;

    // fn quantize_directional(directional_type : DirectionalType, x : f32, y : f32, regions : u8) -> i8;
    
    // fn quantize_continuous(continuous_type : ContinuousType, val : f32, regions : u8) -> i8;

    // fn get_button_state(&self, button : ButtonType) -> bool;
}