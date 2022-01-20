mod display;

use display::get_display;

fn main() {
    get_display()
        .poweron()
        .poweroff();
}
