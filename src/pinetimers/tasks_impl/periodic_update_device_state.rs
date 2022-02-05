use chrono::Duration;

use rtic::mutex_prelude::TupleExt02;

use fugit::ExtU32;

pub fn periodic_update_device_state(ctx: crate::tasks::periodic_update_device_state::Context) {
    crate::tasks::periodic_update_device_state::spawn_after(1.secs()).unwrap();

    (
        ctx.shared.devicestate,
        ctx.shared.rtc,
    ).lock(|devicestate, rtc| {
        let new_counter = rtc.get_counter().try_into().unwrap();
        devicestate.datetime = devicestate.datetime + Duration::milliseconds(
            125i64 * (new_counter - devicestate.counter) as i64
        );
        devicestate.counter = new_counter;
    });

    crate::tasks::ble_update::spawn().unwrap();
    crate::tasks::redraw_screen::spawn().unwrap();
}
