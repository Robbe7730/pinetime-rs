use rtic::mutex_prelude::TupleExt02;

pub fn ble_update(ctx: crate::tasks::ble_update::Context) {
    (
        ctx.shared.bluetooth,
        ctx.shared.battery
    ).lock(|bluetooth, battery| {
        bluetooth.update_data(battery);
    })
}
