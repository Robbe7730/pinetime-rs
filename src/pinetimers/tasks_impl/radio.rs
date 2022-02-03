use rtic::mutex_prelude::TupleExt02;
use rubble::time::Timer;

pub fn radio(ctx: crate::tasks::radio::Context) {
    (
        ctx.shared.ble_ll,
        ctx.shared.bluetooth,
    ).lock(|ble_ll, bluetooth| {
        if let Some(cmd) = bluetooth.recv_interrupt(ble_ll.timer().now(), ble_ll)
        {
            bluetooth.configure_receiver(cmd.radio);
            ble_ll.timer().configure_interrupt(cmd.next_update);

            if cmd.queued_work {
                crate::tasks::ble_worker::spawn().unwrap();
            }
        }
    });
}
