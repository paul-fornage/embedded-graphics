//! Iterators.

pub mod contiguous;
pub mod pixel;
pub mod raw;

use crate::{
    draw_target::DrawTarget, geometry::Point, pixelcolor::PixelColor, primitives::Rectangle, Pixel,
};

/// Extension trait for contiguous iterators.
pub trait ContiguousIteratorExt
where
    Self: Iterator + Sized,
    <Self as Iterator>::Item: PixelColor,
{
    /// Converts a contiguous iterator into a pixel iterator.
    fn into_pixels(self, bounding_box: &Rectangle) -> contiguous::IntoPixels<Self>;
}

impl<I> ContiguousIteratorExt for I
where
    I: Iterator,
    I::Item: PixelColor,
{
    fn into_pixels(self, bounding_box: &Rectangle) -> contiguous::IntoPixels<Self> {
        contiguous::IntoPixels::new(self, *bounding_box)
    }
}

/// Extension trait for pixel iterators.
#[allow(async_fn_in_trait)] // Futures may stay on the local embedded executor.
pub trait PixelIteratorExt<C>
where
    Self: Sized,
    C: PixelColor,
{
    /// Draws the pixel iterator to a draw target.
    async fn draw<D>(self, target: &mut D) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = C>;

    /// Returns a translated version of the iterator.
    fn translated(self, offset: Point) -> pixel::Translated<Self>;
}

impl<I, C> PixelIteratorExt<C> for I
where
    C: PixelColor,
    I: Iterator<Item = Pixel<C>>,
{
    async fn draw<D>(self, target: &mut D) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = C>,
    {
        target.draw_iter(self).await
    }

    fn translated(self, offset: Point) -> pixel::Translated<Self> {
        pixel::Translated::new(self, offset)
    }
}

#[cfg(test)]
mod tests {
    use crate::test_support::FutureExt as _;
    use crate::{
        geometry::Point, iterator::PixelIteratorExt, mock_display::MockDisplay,
        pixelcolor::BinaryColor, Pixel,
    };

    #[test]
    fn draw_pixel_iterator() {
        let pixels = [
            Pixel(Point::new(0, 0), BinaryColor::On),
            Pixel(Point::new(1, 0), BinaryColor::Off),
            Pixel(Point::new(2, 0), BinaryColor::On),
            Pixel(Point::new(2, 1), BinaryColor::Off),
        ];

        let mut display = MockDisplay::new();
        pixels.iter().copied().draw(&mut display).ready().unwrap();

        display.assert_pattern(&[
            "#.#", //
            "  .", //
        ]);
    }
}
