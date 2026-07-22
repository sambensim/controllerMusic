use hidapi::{HidApi, HidDevice};

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

pub fn _print_data(controller : HidDevice) {
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

#[derive(Default)]
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
    pub square: bool,
    pub cross: bool,
    pub circle: bool,
    pub triangle: bool,
    pub l_bumper: bool,
    pub r_bumper: bool,
    pub l_trigger_btn: bool,
    pub r_trigger_btn: bool,
    pub share: bool,
    pub options: bool,
    pub l_stick_btn: bool,
    pub r_stick_btn: bool,

    // D-pad: 0-7 clockwise from up, 8 == none
    pub dpad: u8,
}

pub fn parse_report(buf: &[u8]) -> DS4State {
    // Normalize a raw 0-255 byte to -1.0 to 1.0
    let axis = |byte: u8| -> f32 {
        (byte as f32 - 128.0) / 128.0
    };

    // Normalize a raw 0-255 byte to 0.0 to 1.0
    let trigger = |byte: u8| -> f32 {
        byte as f32 / 255.0
    };

    let buttons = buf[7];
    let byte8 = buf[8];

    DS4State {
        left_stick_x:  axis(buf[3]),
        left_stick_y:  axis(buf[4]),
        right_stick_x: axis(buf[5]),
        right_stick_y: axis(buf[6]),

        l_trigger: trigger(buf[10]),
        r_trigger: trigger(buf[11]),

        // High nibble of byte 7
        square:   buttons & 0b00010000 != 0,
        cross:    buttons & 0b00100000 != 0,
        circle:   buttons & 0b01000000 != 0,
        triangle: buttons & 0b10000000 != 0,

        // Low nibble of byte 7
        dpad: buttons & 0x0F,

        // High nibble of byte 8
        share:       byte8 & 0b00010000 != 0,
        options:     byte8 & 0b00100000 != 0,
        l_stick_btn: byte8 & 0b01000000 != 0,
        r_stick_btn: byte8 & 0b10000000 != 0,

        // Low nibble of byte 8
        l_bumper:     byte8 & 0b00000001 != 0,
        r_bumper:     byte8 & 0b00000010 != 0,
        l_trigger_btn: byte8 & 0b00000100 != 0,
        r_trigger_btn: byte8 & 0b00001000 != 0,
    }
}

use std::f32::consts::PI;
use std::sync::{Arc, Mutex};
use std::thread;

pub fn start_controller_thread(device: hidapi::HidDevice) -> Arc<Mutex<DS4State>> {
    let state = Arc::new(Mutex::new(DS4State::default()));
    let state_clone = Arc::clone(&state);

    thread::spawn(move || {
        let mut buf = [0u8; 78];
        loop {
            if let Ok(_) = device.read(&mut buf) {
                let parsed = parse_report(&buf);
                *state_clone.lock().unwrap() = parsed;
            }
        }
    });

    state // return the Arc to the caller
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