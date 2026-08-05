use hidapi::{HidApi, HidDevice};
use std::f32::consts::PI;
use std::sync::{mpsc};
use std::thread;

pub fn get_dualshock() -> Result<HidDevice, String> {
    let api = HidApi::new().expect("Failed to initialize HidApi");
    let controller = api.device_list().find(|device| {
        device.product_string()
            .unwrap_or("")
            .contains("DUALSHOCK")
    });
    if let Some(_device) = controller {
        // println!("{:04x}:{:04x}", controller.unwrap().vendor_id(), controller.unwrap().product_id());
        // println!("{:?}:{:?}", controller.unwrap().manufacturer_string(), controller.unwrap().product_string());
        return Ok(controller.unwrap().open_device(&api).unwrap());
    } else {
        return Err("No DualShock controller found".to_string())
    }
}

pub fn _print_data(controller : &HidDevice) {
    loop {
        let mut buf = [0u8; 78]; // 0x11 report is 78 bytes
        match controller.read(&mut buf) {
            Ok(len) => {
                // Print each byte with its index so you can identify offsets
                for (i, byte) in buf[..len].iter().enumerate() {
                    print!("[{}]:{:#04x} ", i, byte);
                }
                println!();
            }
            Err(e) => {
                eprintln!("Read error: {}", e);
                break;
            }
        }
    }
}

/*
dualshock mappings:
l-stick horizontal: byte 3: left == 00, right == ff
l-stick vertical: byte 4: up == 00, down == ff
r-stick horizontal: byte 5: left == 00, right == ff
r-stick vertical: byte 6: up == 00, down == ff
buttons: byte 7 (first four bits): [square, x, circle, triangle]
d-pad: byte 7 (last four bits): up is 0, clockwise until 7. None is 8
byte 8: (first four bits): [share, options, l-stick down, r-stick down]
byte 8: (last four bits): [l-bumper, r-bumper, l-trigger down, r-trigger down]
l-trigger: byte 10: none == 00, full == ff
r-trigger: byte 11: none == 00, full == ff
*/

#[derive(Default, Copy, Clone, Debug)]
pub struct DS4State {
    // Sticks: normalized -1.0 to 1.0
    pub left_stick_x: f32,
    pub left_stick_y: f32,
    pub right_stick_x: f32,
    pub right_stick_y: f32,
    // Triggers: 0.0 to 1.0
    pub l_trigger: f32,
    pub r_trigger: f32,
    // Buttons
    pub packed_button_states : u16,
    // D-pad: 0-7 clockwise from up, 8 == none
    pub dpad: i8,
}

pub const DS4_EMPTY : DS4State = DS4State {
    left_stick_x: 0.0,
    left_stick_y: 0.0,
    right_stick_x: 0.0,
    right_stick_y: 0.0,
    l_trigger: 0.0,
    r_trigger: 0.0,
    packed_button_states : 0,
    dpad: 8,
};

pub fn parse_report(buf: &[u8]) -> DS4State {
    // Normalize a raw 0-255 byte to -1.0 to 1.0
    let axis = |byte: u8| -> f32 {
        (byte as f32 - 128.0) / 128.0
    };
    // Normalize a raw 0-255 byte to 0.0 to 1.0
    let trigger = |byte: u8| -> f32 {
        byte as f32 / 255.0
    };
    if buf[0] == 0x11 {
        return DS4State {
            left_stick_x:  axis(buf[3]),
            left_stick_y:  axis(buf[4]),
            right_stick_x: axis(buf[5]),
            right_stick_y: axis(buf[6]),
            l_trigger: trigger(buf[10]),
            r_trigger: trigger(buf[11]),
            // High nibble of byte 7 + all of byte 8
            packed_button_states :  (buf[8] as u16) << 4 | ((buf[7] & 0xF0) as u16) >> 4,
            // Low nibble of byte 7
            dpad: buf[7] as i8 & 0x0F,
        }
    } else if buf[0] == 0x01 {
        return DS4State {
            left_stick_x:  axis(buf[1]),
            left_stick_y:  axis(buf[2]),
            right_stick_x: axis(buf[3]),
            right_stick_y: axis(buf[4]),
            l_trigger: trigger(buf[8]),
            r_trigger: trigger(buf[9]),
            // High nibble of byte 5 + all of byte 6
            packed_button_states :  (buf[6] as u16) << 4 | ((buf[5] & 0xF0) as u16) >> 4,
            // Low nibble of byte 7
            dpad: buf[5] as i8 & 0x0F,
        }
    }
    panic!("unknown mapping");
}

pub fn start_controller_thread(device: hidapi::HidDevice) -> mpsc::Receiver<DS4State> {

    let (sender, receiver) = mpsc::channel();

    thread::spawn(move || {
        let mut buf = [0u8; 78];
        loop {
            // _print_data(&device);
            if let Ok(_) = device.read(&mut buf) {
                let parsed = parse_report(&buf);
                // println!("{parsed:?}");
                let _ = sender.send(parsed);
            }
        }
    });

    // state // return the Arc to the caller
    receiver
}

pub fn get_left_stick_section(controller_state : &DS4State) -> i8 {
    get_vec_section(controller_state.left_stick_x, controller_state.left_stick_y)
}

pub fn get_right_stick_section(controller_state : &DS4State) -> i8 {
    get_vec_section(controller_state.right_stick_x, controller_state.right_stick_y)
}

fn get_vec_section(x : f32, y : f32) -> i8 {
    let magnitude = (x * x + y * y).sqrt();
    const THRESHOLD : f32 = 0.4;
    if magnitude < THRESHOLD { return -1 } ;
    let angle : f32 = y.atan2(x);
    let normalized_angle : f32 = if angle < 0.0 { angle + 2.0 * PI } else { angle };
    ((normalized_angle / ((2.0 * PI) / 8.0)).round() % 8.0) as i8
  }

pub fn get_button_state(controller_state : &DS4State, id : ButtonType) -> bool {
    (controller_state.packed_button_states & 1<<id as u8) != 0
}

pub fn button_events(prev: u16, current: u16) -> impl Iterator<Item = InputEvent> {
    let changed = prev ^ current;
    BUTTONS.iter().enumerate()
    .filter(move |(i, _)| changed & (1u16 << i) != 0)
    .map(move |(i, &b)| InputEvent::Button(b, current & (1u16 << i) != 0))
}

#[derive(Copy, Clone, Debug)]
pub enum  InputEvent {
    Directional(DirectionalType, i8),
    Trigger(TriggerType, f32),
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
pub enum TriggerType {
    Left,
    Right,
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

pub fn get_events(prev_state : DS4State, current_state : DS4State) -> impl Iterator<Item = InputEvent> {
    let left =  (get_left_stick_section(&prev_state) != get_left_stick_section(&current_state))
        .then(|| InputEvent::Directional(DirectionalType::Left, get_left_stick_section(&current_state)))
        .into_iter();
    let right =  (get_right_stick_section(&prev_state) != get_right_stick_section(&current_state))
        .then(|| InputEvent::Directional(DirectionalType::Right, get_right_stick_section(&current_state)))
        .into_iter();
    let dpad = (prev_state.dpad != current_state.dpad)
        .then(|| InputEvent::Directional(DirectionalType::Right, current_state.dpad))
        .into_iter();
    let buttons = button_events(prev_state.packed_button_states, current_state.packed_button_states);
    //TODO - handle continuos input events (like trigger)
    left.chain(right).chain(dpad)
        .chain(buttons)
}