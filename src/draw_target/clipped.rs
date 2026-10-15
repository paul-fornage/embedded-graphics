use crate::{
    draw_target::DrawTarget, geometry::Dimensions, iterator::contiguous::Cropped,
    primitives::Rectangle, transform::Transform, Pixel,
};

/// Clipped draw target.
///
/// Created by calling [`clipped`] on any [`DrawTarget`].
/// See the [`clipped`] method documentation for more.
///
/// [`clipped`]: crate::draw_target::DrawTargetExt::clipped
#[derive(Debug)]
pub struct Clipped<'a, T>
where
    T: DrawTarget,
{
    parent: &'a mut T,
    clip_area: Rectangle,
}

impl<'a, T> Clipped<'a, T>
where
    T: DrawTarget,
{
    pub(super) fn new(parent: &'a mut T, clip_area: &Rectangle) -> Self {
        let clip_area = clip_area.intersection(&parent.bounding_box());

        Self { parent, clip_area }
    }
}

impl<T> DrawTarget for Clipped<'_, T>
where
    T: DrawTarget,
{
    type Color = T::Color;
    type Error = T::Error;

    async fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        let pixels = pixels
            .into_iter()
            .filter(|Pixel(p, _)| self.clip_area.contains(*p));

        self.parent.draw_iter(pixels).await
    }

    async fn fill_contiguous<I>(&mut self, area: &Rectangle, colors: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Self::Color>,
    {
        let intersection = self.bounding_box().intersection(area);

        if &intersection == area {
            self.parent.fill_contiguous(area, colors).await
        } else {
            let crop_area = intersection.translate(-area.top_left);
            let cropped = Cropped::new(colors.into_iter(), area.size, &crop_area);
            self.parent.fill_contiguous(&intersection, cropped).await
        }
    }

    async fn fill_solid(
        &mut self,
        area: &Rectangle,
        color: Self::Color,
    ) -> Result<(), Self::Error> {
        let area = area.intersection(&self.clip_area);

        self.parent.fill_solid(&area, color).await
    }
}

impl<T> Dimensions for Clipped<'_, T>
where
    T: DrawTarget,
{
    fn bounding_box(&self) -> Rectangle {
        self.clip_area
    }
}

#[cfg(test)]
mod tests {
    use crate::test_support::FutureExt as _;
    use crate::{
        draw_target::{DrawTarget, DrawTargetExt},
        geometry::Dimensions,
        geometry::{Point, Size},
        mock_display::MockDisplay,
        pixelcolor::BinaryColor,
        primitives::{Primitive, PrimitiveStyle, Rectangle},
        Drawable, Pixel,
    };

    #[test]
    fn draw_iter() {
        let mut display = MockDisplay::new();

        let area = Rectangle::new(Point::new(2, 1), Size::new(2, 4));
        let mut clipped = display.clipped(&area);

        let pixels = [
            Pixel(Point::new(0, 1), BinaryColor::On),
            Pixel(Point::new(1, 1), BinaryColor::On),
            Pixel(Point::new(2, 1), BinaryColor::On),
            Pixel(Point::new(3, 1), BinaryColor::On),
            Pixel(Point::new(4, 1), BinaryColor::On),
            Pixel(Point::new(2, 0), BinaryColor::Off),
            Pixel(Point::new(2, 2), BinaryColor::Off),
            Pixel(Point::new(2, 3), BinaryColor::Off),
            Pixel(Point::new(2, 4), BinaryColor::Off),
            Pixel(Point::new(2, 5), BinaryColor::Off),
        ];
        clipped.draw_iter(pixels.iter().copied()).ready().unwrap();

        display.assert_pattern(&[
            "    ", //
            "  ##", //
            "  . ", //
            "  . ", //
            "  . ", //
        ]);
    }

    #[test]
    fn fill_contiguous() {
        let mut display = MockDisplay::new();

        let area = Rectangle::new(Point::new(3, 2), Size::new(2, 3));
        let mut clipped = display.clipped(&area);

        let colors = [
            1, 1, 1, 1, 1, //
            0, 0, 0, 0, 1, //
            0, 1, 0, 1, 1, //
            1, 0, 1, 0, 1, //
        ];
        let area = Rectangle::new(Point::new(1, 2), Size::new(5, 4));
        clipped
            .fill_contiguous(&area, colors.iter().map(|c| BinaryColor::from(*c != 0)))
            .ready()
            .unwrap();

        display.assert_pattern(&[
            "     ", //
            "     ", //
            "   ##", //
            "   ..", //
            "   .#", //
        ]);
    }

    #[test]
    fn fill_solid() {
        let mut display = MockDisplay::new();

        let area = Rectangle::new(Point::new(3, 2), Size::new(4, 2));
        let mut clipped = display.clipped(&area);

        let area = Rectangle::new(Point::new(2, 1), Size::new(6, 4));
        clipped.fill_solid(&area, BinaryColor::On).ready().unwrap();

        display.assert_pattern(&[
            "       ", //
            "       ", //
            "   ####", //
            "   ####", //
        ]);
    }

    #[test]
    fn clear() {
        let mut display = MockDisplay::new();

        let area = Rectangle::new(Point::new(1, 3), Size::new(3, 4));
        let mut clipped = display.clipped(&area);
        clipped.clear(BinaryColor::On).ready().unwrap();

        let mut expected = MockDisplay::new();
        area.into_styled(PrimitiveStyle::with_fill(BinaryColor::On))
            .draw(&mut expected)
            .ready()
            .unwrap();

        display.assert_eq(&expected);
    }

    #[test]
    fn bounding_box() {
        let mut display: MockDisplay<BinaryColor> = MockDisplay::new();

        let area = Rectangle::new(Point::new(1, 3), Size::new(2, 4));
        let clipped = display.clipped(&area);

        assert_eq!(clipped.bounding_box(), area);
    }

    #[test]
    fn bounding_box_is_clipped() {
        let mut display: MockDisplay<BinaryColor> = MockDisplay::new();
        let display_bb = display.bounding_box();

        let top_left = Point::new(10, 20);
        let size = Size::new(1000, 1000);
        let area = Rectangle::new(top_left, size);
        let clipped = display.clipped(&area);

        let expected_size = display_bb.size - Size::new(top_left.x as u32, top_left.y as u32);

        assert_eq!(
            clipped.bounding_box(),
            Rectangle::new(top_left, expected_size),
        );
    }
}
