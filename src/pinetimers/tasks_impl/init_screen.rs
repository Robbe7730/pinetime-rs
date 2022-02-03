use rtic::mutex_prelude::TupleExt03;

pub fn init_screen(ctx: crate::tasks::init_screen::Context) {
    (
        ctx.shared.display,
        ctx.shared.current_screen,
        ctx.shared.devicestate
    ).lock(|display, current_screen, devicestate| {
        current_screen.draw_init(display, devicestate);
    });
}
