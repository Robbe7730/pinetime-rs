use rtic::mutex_prelude::TupleExt04;

pub fn gpiote_interrupt(ctx: crate::tasks::gpiote_interrupt::Context) {
    (
        ctx.shared.gpiote,
        ctx.shared.touchpanel,
        ctx.shared.current_screen,
        ctx.shared.devicestate
    ).lock(|gpiote, touchpanel, current_screen, devicestate| {
        if gpiote.channel0().is_event_triggered() {
            devicestate.counter += 1;
        } else if gpiote.channel1().is_event_triggered() {
            touchpanel.handle_interrupt(Some(current_screen.get_event_handler()));
        } else if gpiote.channel2().is_event_triggered() {
            devicestate.update_battery();
        } else {
            panic!("Unknown channel triggered");
        }
        gpiote.reset_events();
    });
}
