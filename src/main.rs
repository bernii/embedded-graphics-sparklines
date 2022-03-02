//! # Example with Display Simulator: Sparklines
//!
//! A simple example showing basic behaviour of `sparklines` with semi-randomly
//! generated data.
//! There are five phases where we generate various data that is then beeing
//! displayed with sparkline, each one has 50 samples:
//! - first, a sine wave is generated
//! - second, there's a linear grpwth function
//! - third and fourth uses an exponential up and down functions
//! - in the last phase random samples are generated

use std::{thread, time::Duration};

use embedded_graphics::{
    mono_font::{ascii::FONT_5X7, ascii::FONT_6X10, MonoTextStyle},
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::{Circle, PrimitiveStyleBuilder, Rectangle, StrokeAlignment},
    text::{Alignment, Text},
};
#[cfg(feature = "build-binary")]
use embedded_graphics_simulator::{
    BinaryColorTheme, OutputSettingsBuilder, SimulatorDisplay, SimulatorEvent, Window,
};

use embedded_graphics::primitives::Line;
use embedded_graphics_sparklines::Sparkline;

// 25Hz updates
const UPDATE_FREQ: Duration = Duration::from_millis(1000 / 25);

#[cfg(feature = "build-binary")]
use rand::Rng;

fn main() -> Result<(), std::convert::Infallible> {
    // Create a new simulator display with 128x64 pixels.
    let mut display: SimulatorDisplay<BinaryColor> = SimulatorDisplay::new(Size::new(240, 135));

    // define bonding rectangle that defines the position
    // and size of the sparkline
    let bbox = Rectangle::new(Point::new(0, 26), Size::new(240, 90));

    // * defining the drawing function     *
    // * comment #2 to see dotted function *

    // draw function 1 - dots / circles
    let _draw_fn = |lastp: Point, _p: Point| Circle::new(Point::new(lastp.x, lastp.y), 2);

    // draw function 2 - lines
    let draw_fn = |lastp, p| Line::new(lastp, p);

    // create sparkline object
    let mut sparkline = Sparkline::new(
        bbox, // position and size of the sparkline
        32,   // max samples to store in memory (and display on graph)
        BinaryColor::On,
        1, // stroke size
        draw_fn,
    );

    // add 9 bootstrap values
    sparkline.add(22);
    sparkline.add(32);
    sparkline.add(12);
    sparkline.add(12);
    sparkline.add(12);
    sparkline.add(12);
    sparkline.add(22);
    sparkline.add(22);
    sparkline.add(22);

    let output_settings = OutputSettingsBuilder::new()
        .theme(BinaryColorTheme::OledBlue)
        .build();
    let mut window = Window::new("Sparkline", &output_settings);

    let mut i = 0;
    'running: loop {
        display.clear(BinaryColor::Off)?;

        let border_stroke = PrimitiveStyleBuilder::new()
            .stroke_color(BinaryColor::On)
            .stroke_width(3)
            .stroke_alignment(StrokeAlignment::Inside)
            .build();

        // Draw a 3px wide outline around the display.
        display
            .bounding_box()
            .into_styled(border_stroke)
            .draw(&mut display)?;

        // samples, 50 each:
        // sin
        // linear
        // exponential up
        // exponential down
        // random after that :-)
        let (val, phase) = match i {
            0..=50 => ((f32::sin(i as f32) * 22_f32) as i32, "sin"),
            51..=100 => (i, "lin"),
            101..=150 => ((i - 100).pow(2), "exp"),
            151..=200 => ((200 - i).pow(2), "-exp"),
            _ => (rand::thread_rng().gen_range(0..100), "rnd"),
        };

        sparkline.add(val);
        sparkline.draw(&mut display)?;

        Text::with_alignment(
            &format!(
                "samples # {}/{}",
                sparkline.values.len(),
                sparkline.max_samples
            ),
            display.bounding_box().bottom_right().unwrap() - Point::new(4, 4),
            MonoTextStyle::new(&FONT_6X10, BinaryColor::On),
            Alignment::Right,
        )
        .draw(&mut display)?;

        Text::with_alignment(
            &format!("iteration # {}", i),
            display.bounding_box().top_left + Point::new(4, 14),
            MonoTextStyle::new(&FONT_6X10, BinaryColor::On),
            Alignment::Left,
        )
        .draw(&mut display)?;

        Text::with_alignment(
            &format!("(val={})", val),
            display.bounding_box().top_left + Point::new(4, 22),
            MonoTextStyle::new(&FONT_5X7, BinaryColor::On),
            Alignment::Left,
        )
        .draw(&mut display)?;

        Text::with_alignment(
            &format!("phase: {}", phase),
            display.bounding_box().top_left + Point::new(display.size().width as i32 - 4, 22),
            MonoTextStyle::new(&FONT_5X7, BinaryColor::On),
            Alignment::Right,
        )
        .draw(&mut display)?;

        window.update(&display);

        if window.events().any(|e| e == SimulatorEvent::Quit) {
            break 'running ();
        }

        thread::sleep(UPDATE_FREQ);
        i += 1;
    }

    Ok(())
}
