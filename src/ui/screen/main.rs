use crate::drivers::bluetooth::Bluetooth;
use crate::ui::screen::Screen;
use crate::drivers::touchpanel::{TouchPanelEventHandler, TouchPoint};
use crate::drivers::display::DisplaySupported;
use crate::drivers::clock::Clock;
use crate::pinetimers::ConnectedRtc;
use crate::drivers::mcuboot::MCUBoot;

use embedded_graphics::prelude::{DrawTarget, Point, Drawable, Transform};
use embedded_graphics::pixelcolor::RgbColor;
use embedded_graphics::primitives::{Circle, Line, Primitive, PrimitiveStyleBuilder};
use embedded_graphics::text::{Text, Baseline};
use embedded_graphics::mono_font::ascii::FONT_10X20;
use embedded_graphics::mono_font::MonoTextStyle;

use core::marker::PhantomData;
use core::fmt::Debug;
use core::f64::consts::PI;

use libm::{cos, sin};

use chrono::Timelike;

use alloc::sync::Arc;
use alloc::vec::Vec;
use alloc::{vec, format};

#[derive(Debug)]
pub struct ScreenMain<COLOR> {
    event_handler: Arc<ScreenMainEventHandler>,
    hands: Vec<Line>,
    _marker: PhantomData<COLOR>
}

#[derive(Debug)]
pub struct ScreenMainEventHandler {}

impl TouchPanelEventHandler for ScreenMainEventHandler {
    fn on_slide_up(&self, _p: TouchPoint) {
    }
}

impl<DISPLAY, COLOR> ScreenMain<DISPLAY>
where
    DISPLAY: DisplaySupported<COLOR> + DrawTarget<Color = COLOR> + Debug,
    <DISPLAY as DrawTarget>::Error: Debug,
    COLOR: RgbColor
{
    fn get_hand(
        &self,
        angle: f64,
        radius: f64,
        center: Point,
    ) -> Line {
        let point = Point::new(
            (sin(angle) * radius) as i32,
            -(cos(angle) * radius) as i32
        );

        Line::new(point, Point::new(0, 0))
            .translate(center)
    }
}

impl<DISPLAY, COLOR> Screen<DISPLAY> for ScreenMain<DISPLAY>
where
    DISPLAY: DisplaySupported<COLOR> + DrawTarget<Color = COLOR> + Send + Debug,
    <DISPLAY as DrawTarget>::Error: Debug,
    COLOR: RgbColor
{
    fn new() -> ScreenMain<DISPLAY> {
        ScreenMain {
            event_handler: Arc::new(ScreenMainEventHandler {}),
            hands: Vec::new(),
            _marker: PhantomData,
        }
    }

    fn get_event_handler(&self) -> Arc<dyn TouchPanelEventHandler> {
        return self.event_handler.clone();
    }

    fn draw_init(&mut self, display: &mut DISPLAY, _clock: &Clock<ConnectedRtc>, _: &MCUBoot, _: &Bluetooth) {
        let clock_center = Point::new(120, 120);
        let clock_radius = 90;

        display.clear(COLOR::BLACK).unwrap();

        Circle::with_center(clock_center, clock_radius * 2)
            .into_styled(
                PrimitiveStyleBuilder::new()
                    .stroke_color(COLOR::WHITE)
                    .stroke_width(2)
                    .build()
            )
            .draw(display)
            .unwrap();
    }

    fn draw_update(&mut self, display: &mut DISPLAY, clock: &Clock<ConnectedRtc>, _: &MCUBoot, bluetooth: &Bluetooth) {
        let clock_center = Point::new(120, 120);
        let clock_radius = 90;

        let clear_style = PrimitiveStyleBuilder::new()
            .stroke_color(COLOR::BLACK)
            .stroke_width(2)
            .build();

        self.hands.iter().for_each(|hand| {
            hand.into_styled(clear_style)
                .draw(display)
                .unwrap();
        });

        self.hands = vec![
            self.get_hand(
                (clock.datetime.time().second() as f64) * (PI / 30.0),
                clock_radius as f64,
                clock_center
            ),
            self.get_hand(
                (clock.datetime.time().minute() as f64) * (PI / 30.0),
                clock_radius as f64,
                clock_center
            ),
            self.get_hand(
                (clock.datetime.time().hour() as f64) * (PI / 6.0),
                clock_radius as f64 * 0.75,
                clock_center
            ),
        ];

        let hand_style = PrimitiveStyleBuilder::new()
            .stroke_color(COLOR::WHITE)
            .stroke_width(2)
            .build();

        self.hands.iter().for_each(|hand| {
            hand.into_styled(hand_style)
                .draw(display)
                .unwrap();
        });

        let text_style = MonoTextStyle::new(&FONT_10X20, COLOR::WHITE);
        Text::with_baseline(
            &format!("{:?}", bluetooth.phy.get_state()),
            Point::new(0, 0),
            text_style,
            Baseline::Top
        )
            .draw(display)
            .unwrap();
    }
}
