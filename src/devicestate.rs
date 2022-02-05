use chrono::NaiveDateTime;

pub struct DeviceState {
    pub counter: usize,
    pub datetime: NaiveDateTime,
}

impl DeviceState {
    pub fn new() -> DeviceState {
        DeviceState {
            counter: 0,
            datetime: NaiveDateTime::from_timestamp(0, 0),
        }
    }
}
