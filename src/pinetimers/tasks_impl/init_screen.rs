use rtic::mutex_prelude::TupleExt03;

pub fn init_screen(ctx: crate::tasks::init_screen::Context) {
    (
        ctx.shared.display,
        ctx.shared.current_screen,
        ctx.shared.clock
    ).lock(|display, current_screen, clock| {
        current_screen.draw_init(display, clock);
    });
}
