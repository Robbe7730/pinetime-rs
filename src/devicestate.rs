use crate::drivers::battery::{BatteryState, Battery};

pub struct DeviceState {
    battery_driver: Battery,

    pub battery: BatteryState,
    pub counter: usize,
}

impl DeviceState {
    pub fn new(mut battery_driver: Battery) -> DeviceState {
        let battery = battery_driver.get_state();
        DeviceState {
            battery_driver,

            battery,
            counter: 0,
        }
    }

    pub fn update_battery(&mut self) {
        self.battery = self.battery_driver.get_state();
    }
}
