use rtic::mutex_prelude::TupleExt02;

pub fn ble_timer(ctx: crate::tasks::ble_timer::Context) {
    (
        ctx.shared.ble_ll,
        ctx.shared.bluetooth
    ).lock(|ble_ll, bluetooth| {
        let timer = ble_ll.timer();
        if !timer.is_interrupt_pending() {
            return;
        }
        timer.clear_interrupt();

        let cmd = ble_ll.update_timer(bluetooth);
        bluetooth.configure_receiver(cmd.radio);

        ble_ll.timer()
            .configure_interrupt(cmd.next_update);

        if cmd.queued_work {
            crate::tasks::ble_worker::spawn().unwrap();
        }
    });
}
