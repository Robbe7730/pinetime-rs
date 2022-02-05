use nrf52832_hal::rtc::{self, Rtc};

use chrono::{NaiveDateTime, Duration};

pub struct Clock<RTC> {
    rtc: Rtc<RTC>,
    pub datetime: NaiveDateTime,
    prev_counter: u32,
}

impl<RTC: rtc::Instance> Clock<RTC> {
    pub fn new(rtc: Rtc<RTC>) -> Self {
        Clock {
            rtc,
            datetime: NaiveDateTime::from_timestamp(0, 0),
            prev_counter: 0,
        }
    }

    pub fn tick(&mut self) {
        let new_counter = self.rtc.get_counter();

        self.datetime += Duration::milliseconds(
            ((new_counter - self.prev_counter) * 125).into()
        );
        self.prev_counter = new_counter;
    }
}
