use crate::drivers::battery::{BatteryState, Battery};
use chrono::{DateTime, TimeZone, Utc};

pub struct DeviceState {
    battery_driver: Battery,

    pub battery: BatteryState,
    pub counter: usize,
    pub datetime: DateTime<Utc>,
}

impl DeviceState {
    pub fn new(mut battery_driver: Battery) -> DeviceState {
        let battery = battery_driver.get_state();
        DeviceState {
            battery_driver,

            battery,
            counter: 0,
            datetime: Utc.timestamp(0, 0),
        }
    }

    pub fn update_battery(&mut self) {
        self.battery = self.battery_driver.get_state();
    }
}
