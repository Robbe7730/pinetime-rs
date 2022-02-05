use rtic::Mutex;

use chrono::NaiveDateTime;

pub fn set_time(mut ctx: crate::tasks::set_time::Context, time: NaiveDateTime) {
    ctx.shared.clock.lock(|clock| {
        clock.datetime = time;
    });
}
