use rtic::mutex_prelude::TupleExt04;

pub fn ble_update(ctx: crate::tasks::ble_update::Context) {
    (
        ctx.shared.bluetooth,
        ctx.shared.battery,
        ctx.shared.clock,
        ctx.shared.mcuboot
    ).lock(|bluetooth, battery, clock, mcuboot| {
        bluetooth.update_data(battery, clock, mcuboot);
    })
}
