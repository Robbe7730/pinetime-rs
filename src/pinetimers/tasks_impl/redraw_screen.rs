use rtic::mutex_prelude::TupleExt03;

pub fn redraw_screen(ctx: crate::tasks::redraw_screen::Context) {
    (
        ctx.shared.display,
        ctx.shared.current_screen,
        ctx.shared.clock
    ).lock(|display, current_screen, clock| {
        current_screen.draw_update(display, clock);
    });
}
