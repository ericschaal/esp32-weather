use std::ops::{Add, Div, Mul, Range, Sub};

pub trait Scalable<T>
    where
        T: Copy + Add<Output = T> + Sub<Output = T> + Mul<Output = T> + Div<Output = T>,
{
    fn scale_between_ranges(&self, input_range: &Range<T>, output_range: &Range<T>) -> T;
}

impl<T> Scalable<T> for T
    where
        T: Copy + Add<Output = T> + Sub<Output = T> + Mul<Output = T> + Div<Output = T>,
{
    fn scale_between_ranges(&self, input_range: &Range<T>, output_range: &Range<T>) -> T {
        (*self - input_range.start) * (output_range.end - output_range.start)
            / (input_range.end - input_range.start)
            + output_range.start
    }
}