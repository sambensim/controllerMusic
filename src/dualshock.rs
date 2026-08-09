use hidapi::{HidApi, HidDevice};

use crate::intermediate_controller_state::IntermediateControllerState;

#[derive(Default, Copy, Clone, Debug)]
pub struct DS4 {}
//     // Sticks: normalized -1.0 to 1.0
//     pub left_stick_x: f32,
//     pub left_stick_y: f32,
//     pub right_stick_x: f32,
//     pub right_stick_y: f32,
//     // Triggers: 0.0 to 1.0
//     pub l_trigger: f32,
//     pub r_trigger: f32,
//     // Buttons
//     pub packed_button_states : u16,
//     // D-pad: 0-7 clockwise from up, 8 == none
//     pub dpad: i8,
// }

impl crate::controller::Controller for DS4 {
    fn get_controller() -> Result<HidDevice, String> {
        let api = HidApi::new().expect("Failed to initialize HidApi");
        let controller = api.device_list().find(|device| {
            device.product_string()
                .unwrap_or("")
                .contains("DUALSHOCK")
        });
        if let Some(_device) = controller {
            // println!("{:04x}:{:04x}", controller.unwrap().vendor_id(), controller.unwrap().product_id());
            // println!("{:?}:{:?}", controller.unwrap().manufacturer_string(), controller.unwrap().product_string());
            let out = controller.unwrap().open_device(&api).unwrap();
            let _ = Self::enable_full_bt_reports(&out); //TODO - handle error
            return Ok(out)
        } else {
            return Err("No DualShock controller found".to_string())
        }
    }

    fn parse_report(buf: &[u8]) -> Option<IntermediateControllerState> {
        // _print_data(buf);
        // Normalize a raw 0-255 byte to -1.0 to 1.0
        let axis = |byte: u8| -> f32 {
            (byte as f32 - 128.0) / 128.0
        };
        // Normalize a raw 0-255 byte to 0.0 to 1.0
        let trigger = |byte: u8| -> f32 {
            byte as f32 / 255.0
        };
        if buf[0] == 0x11 {
            if (buf[1] >> 7) & 1 == 0 {
                return None; //audio only output (microphone?)
            }
            return Some(IntermediateControllerState {
                left_stick_x:  axis(buf[3]),
                left_stick_y:  axis(buf[4]),
                right_stick_x: axis(buf[5]),
                right_stick_y: axis(buf[6]),
                l_trigger: trigger(buf[10]),
                r_trigger: trigger(buf[11]),
                // High nibble of byte 7 + all of byte 8
                packed_button_states :   (if buf[37]&(1<<7) == 0 {1} else {0}) << 14 | ((buf[9]&3) as u16) << 12 | (buf[8] as u16) << 4 | ((buf[7] & 0xF0) as u16) >> 4,
                // Low nibble of byte 7
                dpad: buf[7] as i8 & 0x0F,
                touchpad_x : ((buf[38] as u16) | (((buf[39] as u16) & 7_u16) << 8_u16)) as f32 / 2048.0,
                touchpad_y : (((buf[40] as u16) << 8_u16) | ((buf[39] as u16) & (15_u16<<4))) as f32 / 16384.0,
            })
        } else if buf[0] == 0x01 {
            return Some(IntermediateControllerState {
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
                touchpad_x : -1.0,
                touchpad_y : -1.0,
            })
        }
        panic!("unknown mapping");
    }
}

impl DS4 {
    fn enable_full_bt_reports(device: &hidapi::HidDevice) -> hidapi::HidResult<()> { //AI
        // Request the BT calibration feature report (0x05).
        // This is what tells the DS4 to switch from the short 0x01 reports
        // to the extended 0x11–0x19 reports. You only need to call this once
        let mut buf = [0u8; 41]; // ReportFeatureInCalibrateBT is 41 bytes
        buf[0] = 0x05;           // report ID
        device.get_feature_report(&mut buf)?;
        // You can parse the calibration data out of buf here if you want it
        Ok(())
    }
}

pub fn _print_data(data : &[u8]) {
    // Print each byte with its index so you can identify offsets
    // for i in 0..data[35] {
    //     let start : usize = 36 + (i*9) as usize;
    //     if data[start+1]&(1<<7) == 0 {
    //         let x : u16 = (data[start+2] as u16) | (((data[start+3] as u16) & 7_u16) << 8_u16);
    //         let y : u16 = ((data[start+4] as u16) << 8_u16) | ((data[start+3] as u16) & (15_u16<<4));
    //         // print!("{x}")
    //         print!("finger {}: ({x}, {y})\n", data[start+1])
    //     }
    //     if data[start+5]&(1<<7) == 0 {
    //         let x : u16 = (data[start+6] as u16) | (((data[start+7] as u16) & 7_u16) << 8_u16);
    //         let y : u16 = ((data[start+8] as u16) << 8_u16) | ((data[start+7] as u16) & (15_u16<<4));
    //         // print!("{x}")
    //         // print!("finger {}: ({x}, {y})\n", data[start+5])
    //     }
        // println!()
    println!("{}", data[9]&(1<<1))
    // for (i, byte) in data.iter().enumerate() {
    //     print!("[{}]:{:#04x} ", i, byte);
    // }
    // println!();
    // println!();
    // println!();
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
ps button: byte 9 bit 0
touchpad button: byte 9 bit 1
*/
