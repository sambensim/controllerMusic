use hidapi::{DeviceInfo, HidApi};

pub fn get_dualshock() -> Result<DeviceInfo, String> {
    let api = HidApi::new().expect("Failed to initialize HidApi");
    let controller = api.device_list().find(|device| {
        device.product_string()
            .unwrap_or("")
            .contains("DUALSHOCK")
    });
    if let Some(_device) = controller {
        // println!("{:04x}:{:04x}", controller.unwrap().vendor_id(), controller.unwrap().product_id());
        // println!("{:?}:{:?}", controller.unwrap().manufacturer_string(), controller.unwrap().product_string());
        return Ok(controller.unwrap().clone());
    } else {
        return Err("No DualShock controller found".to_string())
    }
}