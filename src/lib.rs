use embedded_graphics::pixelcolor::PixelColor;
use embedded_graphics::prelude::Primitive;
use embedded_graphics::primitives::{PrimitiveStyle, StyledDrawable};

use embedded_graphics::prelude::{DrawTarget, Point};
use embedded_graphics::primitives::Rectangle;
use embedded_graphics::Drawable;
use std::collections::VecDeque;

pub struct Sparkline<C, F, P>
where
    C: PixelColor,
    F: Fn(Point, Point) -> P,
    P: Primitive + StyledDrawable<PrimitiveStyle<C>, Color = C>,
{
    pub values: VecDeque<i32>,
    bbox: Rectangle,
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

        let (min, max): (&i32, &i32) =
            self.values
                .iter()
                .fold((&i32::MAX, &i32::MIN), |mut acc, val| {
                    if val < &acc.0 {
                        acc.0 = val;
                    }
                    if val > &acc.1 {
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

            let p = Point::new((i as f32 * px_per_seg) as i32, scaled_val as i32);

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
