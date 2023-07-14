use core::ops::Range;

use itertools::{Itertools, MinMaxResult, MinMaxResult::MinMax};

use embedded_graphics::
{
    draw_target::DrawTarget,
    geometry::Point, Drawable, Pixel,
    primitives::{Rectangle, Line, PrimitiveStyle},
    geometry::AnchorPoint::BottomLeft,
    prelude::*
};

use crate::chart::scalable::Scalable;


/// curve object that contains data to be plotted
pub struct BarChart<'a> {
    /// slice of points to be drawn
    points: &'a [Point],
    pub x_range: Range<i32>,
    pub y_range: Range<i32>,
}

impl<'a> BarChart<'a> {
    /// create new curve data with manual ranges
    pub fn new(points: &'a [Point], x_range: Option<Range<i32>>, y_range: Option<Range<i32>>) -> BarChart {
        BarChart {
            points,
            x_range: x_range.unwrap_or(match points.iter().map(|p| (p.x)).minmax() {
                MinMaxResult::NoElements => 0..0,
                MinMaxResult::OneElement(v) => v..v,
                MinMax(min, max) => min..max,
            }),
            y_range: y_range.unwrap_or(match points.iter().map(|p| (p.y)).minmax() {
                MinMaxResult::NoElements => 0..0,
                MinMaxResult::OneElement(v) => v..v,
                MinMax(min, max) => min..max,
            })
        }
    }

    /// create curve that can be drawed on specific display
    pub fn into_drawable_curve<C>(
        &self,
        top_left: &'a Point,
        bottom_right: &'a Point,
    ) -> DrawableCurve<C, impl Iterator<Item = Point> + Clone + '_>
        where
            C: PixelColor,
    {
        assert!(top_left.x < bottom_right.x);
        assert!(top_left.y < bottom_right.y);
        assert!(!self.x_range.is_empty());
        assert!(!self.y_range.is_empty());

        let it = self.points.iter().map(move |p| Point {
            x: p.x.scale_between_ranges(
                &self.x_range,
                &Range {
                    start: top_left.x,
                    end: bottom_right.x,
                },
            ),
            y: p.y.scale_between_ranges(
                &self.y_range,
                &Range {
                    start: bottom_right.y,
                    end: top_left.y,
                },
            ),
        });

        let bounds = Rectangle::new(*top_left, Size::new((bottom_right.x - top_left.x) as u32, (bottom_right.y - top_left.y) as u32));

        DrawableCurve {
            scaled_data: it,
            color: None,
            thickness: None,
            fill: None,
            bounds,
        }
    }
}

/// Drawable curve object, constructed for specific display
pub struct DrawableCurve<C, I> {
    scaled_data: I,
    color: Option<C>,
    thickness: Option<usize>,
    fill: Option<bool>,
    bounds: Rectangle,
}

/// builder methods to modify curve decoration
impl<C, I> DrawableCurve<C, I>
    where
        C: PixelColor,
        I: Iterator<Item = Point> + Clone,
{
    /// set curve color
    pub fn set_color(mut self, color: C) -> DrawableCurve<C, I> {
        self.color = Some(color);
        self
    }

    pub fn set_fill(mut self, fill: bool) -> DrawableCurve<C, I> {
        self.fill = Some(fill);
        self
    }

    /// set curve line thickness
    pub fn set_thickness(mut self, thickness: usize) -> DrawableCurve<C, I> {
        self.thickness = Some(thickness);
        self
    }
}

impl<C, I> Drawable for DrawableCurve<C, I>
    where
        C: PixelColor + Default,
        I: Iterator<Item = Point> + Clone,
{
    type Color = C;
    type Output = ();

    /// most important function - draw the curve on the display
    fn draw<D: DrawTarget<Color = C>>(
        &self,
        display: &mut D,
    ) -> Result<(), <D as DrawTarget>::Error> {
        let color = match &self.color {
            None => C::default(),
            Some(c) => *c,
        };
        let thickness = match &self.thickness {
            None => 2,
            Some(t) => *t,
        };
        let style = PrimitiveStyle::with_stroke(color, thickness as u32);

        self.scaled_data.clone().tuple_windows().try_for_each(
            |(prev, point)| -> Result<(), D::Error> {
                let top_left = Point::new(prev.x, point.y);
                let top_right = point.clone();

                let should_draw = self.bounds.anchor_point(BottomLeft).y - top_left.y > 3;

                if !should_draw {
                    return Ok(());
                }

                let bottom_right = Point::new(point.x, self.bounds.anchor_point(BottomLeft).y);
                let top_line = Line::new(top_left, top_right).into_styled(style);
                let right_vertical_line = Line::new(top_right, bottom_right).into_styled(style);
                let left_vertical_line = Line::new(top_left, prev).into_styled(style);
                top_line.draw(display)?;
                right_vertical_line.draw(display)?;
                left_vertical_line.draw(display)?;

                if self.fill.unwrap_or(false) {
                    let color = self.color.unwrap_or(C::default());
                    let area = Rectangle::with_corners(top_left, bottom_right);
                    let pixels = area.points()
                        .filter(|pos| if pos.y % 2 == 0 {pos.x % 3 == 0} else {pos.x % 3 == 1})
                        .map(|pos| Pixel(pos, color));

                    display.draw_iter(pixels)?;
                }

                Ok(())
            },
        )
    }
}
