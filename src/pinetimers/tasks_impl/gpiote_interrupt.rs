use rtic::mutex_prelude::TupleExt03;

pub fn gpiote_interrupt(ctx: crate::tasks::gpiote_interrupt::Context) {
    (
        ctx.shared.gpiote,
        ctx.shared.touchpanel,
        ctx.shared.current_screen,
    ).lock(|gpiote, touchpanel, current_screen| {
        if gpiote.channel0().is_event_triggered() {
            // Button was pressed
            crate::tasks::reboot::spawn().unwrap();
        } else if gpiote.channel1().is_event_triggered() {
            touchpanel.handle_interrupt(Some(current_screen.get_event_handler()));
        } else if gpiote.channel2().is_event_triggered() {
            // Battery state changed
        } else {
            panic!("Unknown channel triggered");
        }
        gpiote.reset_events();
    });
}
