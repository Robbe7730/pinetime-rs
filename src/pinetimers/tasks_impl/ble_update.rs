use rtic::mutex_prelude::TupleExt03;

pub fn ble_update(ctx: crate::tasks::ble_update::Context) {
    (
        ctx.shared.bluetooth,
        ctx.shared.battery,
        ctx.shared.clock
    ).lock(|bluetooth, battery, clock| {
        bluetooth.update_data(battery, clock);
    })
}
