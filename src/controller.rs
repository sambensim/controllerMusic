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

pub fn print_data(controller : HidDevice) {
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