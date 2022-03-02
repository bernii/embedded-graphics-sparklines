//! # Embedded Graphics Sparklines
//!
//! `embedded-graphics-sparklines` is an implementation of sparkline graphs
//! which can be used to effectively present data on small emedded displays.
//!
//!
//! > Sparklines are "small, high resolution graphics embedded in a context
//! > of words, numbers or images". Edward Tufte describes sparklines as
//! > "data-intense, design-simple, word-sized graphics".
//!

use embedded_graphics::pixelcolor::PixelColor;
use embedded_graphics::prelude::Primitive;
use embedded_graphics::primitives::{PrimitiveStyle, StyledDrawable};

use embedded_graphics::prelude::{DrawTarget, Point};
use embedded_graphics::primitives::Rectangle;
use embedded_graphics::Drawable;
use std::collections::VecDeque;

/// `Drawable` primitive (in sense of `embedded-graphics` lib) that is reponsible
///  for performing normalization, sample storage and drawing of the accumulated
/// data.
///
/// # Example
/// ```
/// use embedded_graphics::prelude::*;
/// use embedded_graphics::pixelcolor::BinaryColor;
/// use embedded_graphics::primitives::{Line, Rectangle};
/// use embedded_graphics::mock_display::MockDisplay;
/// use embedded_graphics_sparklines::Sparkline;
///
/// let mut display: MockDisplay<BinaryColor> = MockDisplay::new();
/// display.set_allow_overdraw(true);
///
/// let draw_fn = |lastp, p| Line::new(lastp, p);
/// let mut sparkline = Sparkline::new(
///         Rectangle::new(Point::new(0, 0), Size::new(16, 5)), // position and size of the sparkline
///         12, // max samples to store in memory (and display on graph)
///         BinaryColor::On,
///         1, // stroke width
///         draw_fn,
///     );
///
///     for n in 0..11 {
///         sparkline.add((f32::sin(n as f32) * 5_f32) as i32);
///     }
/// sparkline.draw(&mut display).unwrap();
///
/// ```
pub struct Sparkline<C, F, P>
where
    C: PixelColor,
    F: Fn(Point, Point) -> P,
    P: Primitive + StyledDrawable<PrimitiveStyle<C>, Color = C>,
{
    /// stores max_samples number of values
    pub values: VecDeque<i32>,
    bbox: Rectangle,
    /// defines the max number of values that sparkline will present
    pub max_samples: usize,
    color: C,
    stroke_width: u32,
    draw_fn: F,
}

impl<C, F, P> Sparkline<C, F, P>
where
    C: PixelColor,
    F: Fn(Point, Point) -> P,
    P: Primitive + StyledDrawable<PrimitiveStyle<C>, Color = C>,
{
    pub fn new(
        bbox: Rectangle,
        max_samples: usize,
        color: C,
        stroke_width: u32,
        draw_fn: F,
    ) -> Self {
        Self {
            values: VecDeque::with_capacity(max_samples),
            bbox,
            max_samples,
            color,
            stroke_width,
            draw_fn,
        }
    }

    pub fn add(&mut self, val: i32) {
        if self.values.len() == self.max_samples {
            self.values.pop_front();
        }
        self.values.push_back(val);
    }
}

impl<C, F, P> Drawable for Sparkline<C, F, P>
where
    C: PixelColor,
    F: Fn(Point, Point) -> P,
    P: Primitive + StyledDrawable<PrimitiveStyle<C>, Color = C>,
{
    type Color = C;
    type Output = ();

    fn draw<D>(&self, target: &mut D) -> Result<Self::Output, D::Error>
    where
        D: DrawTarget<Color = Self::Color>,
    {
        let mut slope: f32 = self.bbox.size.height as f32 - self.stroke_width as f32;

        // find min and max in a single pass
        let (min, max): (&i32, &i32) =
            self.values
                .iter()
                .fold((&i32::MAX, &i32::MIN), |mut acc, val| {
                    if val < acc.0 {
                        acc.0 = val;
                    }
                    if val > acc.1 {
                        acc.1 = val;
                    }
                    acc
                });

        // slope mod
        if max != min {
            slope /= (max - min) as f32;
        }

        let px_per_seg = (self.bbox.size.width - 1) as f32 / (self.values.len() - 1) as f32;
        let mut lastp = Point::new(0, 0);

        for (i, val) in self.values.iter().enumerate() {
            let scaled_val = self.bbox.top_left.y as f32 + self.bbox.size.height as f32
                - ((val - min) as f32 * slope)
                - self.stroke_width as f32 / 2f32;

            let p = Point::new(
                (i as f32 * px_per_seg) as i32 + self.bbox.top_left.x,
                scaled_val as i32,
            );

            // skip first point as it goes from zero
            if i > 0 {
                // draw using supplied closure drawing function
                (self.draw_fn)(lastp, p)
                    .into_styled(PrimitiveStyle::with_stroke(self.color, self.stroke_width))
                    .draw(target)?;
            }
            lastp = p;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use embedded_graphics::mock_display::MockDisplay;
    use embedded_graphics::pixelcolor::BinaryColor;
    use embedded_graphics::prelude::Size;
    use embedded_graphics::primitives::{Circle, Line};

    use super::*;

    enum DrawSignal {
        Sin,
        Lin,
    }

    fn generate_sparkline(
        max_samples: usize,
        stroke_width: u32,
        draw_signal: DrawSignal,
    ) -> Sparkline<BinaryColor, impl Fn(Point, Point) -> Line, Line> {
        let draw_fn = |lastp, p| Line::new(lastp, p);
        let mut sparkline = Sparkline::new(
            Rectangle::new(Point::new(0, 0), Size::new(16, 5)), // position and size of the sparkline
            max_samples, // max samples to store in memory (and display on graph)
            BinaryColor::On,
            stroke_width, // stroke size
            draw_fn,
        );

        for n in 0..11 {
            let val = match &draw_signal {
                DrawSignal::Sin => (f32::sin(n as f32) * 5_f32) as i32,
                DrawSignal::Lin => n,
            };
            sparkline.add(val);
        }

        sparkline
    }

    #[test]
    fn draws_sin() {
        let mut display: MockDisplay<BinaryColor> = MockDisplay::new();
        display.set_allow_overdraw(true);

        let sparkline = generate_sparkline(32, 1, DrawSignal::Sin);
        sparkline.draw(&mut display).unwrap();

        display.assert_pattern(&[
            " ###        #   ",
            "#  #      ## #  ",
            "#   #    #    # ",
            "     #   #     #",
            "      ###       ",
        ]);
    }

    #[test]
    fn draws_sin_10_samples() {
        let mut display: MockDisplay<BinaryColor> = MockDisplay::new();
        display.set_allow_overdraw(true);

        let sparkline = generate_sparkline(10, 1, DrawSignal::Sin);
        sparkline.draw(&mut display).unwrap();

        display.assert_pattern(&[
            "##         ##   ",
            "  #       #  #  ",
            "   #     #    # ",
            "    #   #      #",
            "     ###        ",
        ]);
        assert_eq!(10, sparkline.values.len());
    }

    #[test]
    fn draws_line_using_stroke_width() {
        let mut display: MockDisplay<BinaryColor> = MockDisplay::new();
        display.set_allow_overdraw(true);

        let sparkline_1 = generate_sparkline(20, 1, DrawSignal::Lin);
        sparkline_1.draw(&mut display).unwrap();

        display.assert_pattern(&[
            "             ###",
            "          ###   ",
            "      ####      ",
            "   ###          ",
            "###             ",
        ]);

        display = MockDisplay::new();
        display.set_allow_overdraw(true);
        let sparkline_2 = generate_sparkline(20, 2, DrawSignal::Lin);
        sparkline_2.draw(&mut display).unwrap();

        display.assert_pattern(&[
            "          ######",
            "      ##########",
            " #########      ",
            "######          ",
            "#               ",
        ]);

        display = MockDisplay::new();
        display.set_allow_overdraw(true);
        display.set_allow_out_of_bounds_drawing(true);
        let sparkline_3 = generate_sparkline(20, 3, DrawSignal::Lin);
        sparkline_3.draw(&mut display).unwrap();

        display.assert_pattern(&[
            "            ####",
            "   #############",
            "################",
            "############    ",
            "####            ",
        ]);
    }

    #[test]
    fn draws_in_bounding_box() {
        let mut display: MockDisplay<BinaryColor> = MockDisplay::new();
        display.set_allow_overdraw(true);

        let draw_fn = |lastp, p| Line::new(lastp, p);
        let mut sparkline = Sparkline::new(
            Rectangle::new(Point::new(5, 4), Size::new(16, 5)), // position and size of the sparkline
            32, // max samples to store in memory (and display on graph)
            BinaryColor::On,
            1, // stroke size
            draw_fn,
        );

        for n in 0..11 {
            sparkline.add(n)
        }

        sparkline.draw(&mut display).unwrap();

        display.assert_pattern(&[
            "                     ",
            "                     ",
            "                     ",
            "                     ",
            "                  ###",
            "               ###   ",
            "           ####      ",
            "        ###          ",
            "     ###             ",
        ]);
    }

    #[test]
    fn uses_drawing_function() {
        let mut display: MockDisplay<BinaryColor> = MockDisplay::new();
        // display.set_allow_overdraw(true);

        let draw_fn = |lastp: Point, _p: Point| Circle::new(Point::new(lastp.x, lastp.y), 3);
        let mut sparkline = Sparkline::new(
            Rectangle::new(Point::new(0, 0), Size::new(26, 9)), // position and size of the sparkline
            32, // max samples to store in memory (and display on graph)
            BinaryColor::On,
            1, // stroke size
            draw_fn,
        );

        for n in 0..6 {
            sparkline.add(n)
        }

        sparkline.draw(&mut display).unwrap();

        display.assert_pattern(&[
            "                       ",
            "                       ",
            "                     # ",
            "                #   # #",
            "               # #   # ",
            "           #    #      ",
            "      #   # #          ",
            "     # #   #           ",
            " #    #                ",
            "# #                    ",
            " #                     ",
        ]);
    }
}
