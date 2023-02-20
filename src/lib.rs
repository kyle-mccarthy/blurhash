#![deny(clippy::all)]

use std::collections::VecDeque;
use std::f32::consts::PI;

use fast_srgb8::srgb8_to_f32;
use image::RgbImage;

pub mod base83;
pub mod error;

pub use error::{Error, Result};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Component {
    x: u8,
    y: u8,
}

impl Component {
    pub fn try_new(x: u8, y: u8) -> Result<Self> {
        if !(1..=9).contains(&x) {
            return Err(Error::ComponentOutOfBounds(x));
        }

        if !(1..=9).contains(&y) {
            return Err(Error::ComponentOutOfBounds(y));
        }

        Ok(Self { x, y })
    }

    #[inline]
    pub fn x(&self) -> u8 {
        self.x
    }

    #[inline]
    pub fn y(&self) -> u8 {
        self.y
    }
}

#[inline]
fn basis_fn(
    component_x: usize,
    component_y: usize,
    point_x: u32,
    point_y: u32,
    width: f32,
    height: f32,
) -> f32 {
    let normalizer = if point_x == 0 && point_y == 0 { 1 } else { 2 } as f32;

    normalizer
        * f32::cos((PI * component_x as f32 * point_x as f32) / width)
        * f32::cos((PI * component_y as f32 * point_y as f32) / height)
}

#[inline]
fn multiply_basis_function(
    image: &RgbImage,
    component_x: usize,
    component_y: usize,
) -> (f32, f32, f32) {
    let (mut r, mut b, mut g) = (0., 0., 0.);

    let dimensions = image.dimensions();
    let width = dimensions.0 as f32;
    let height = dimensions.1 as f32;

    image.enumerate_pixels().for_each(|(x, y, pixel)| {
        let basis = basis_fn(component_x, component_y, x, y, width, height);

        r += basis * srgb8_to_f32(pixel[0]);
        g += basis * srgb8_to_f32(pixel[1]);
        b += basis * srgb8_to_f32(pixel[2]);
    });

    let scale = 1. / width * height;

    (r * scale, b * scale, g * scale)
}

fn encode_dc((r, g, b): (f32, f32, f32)) -> usize {
    let srgb = fast_srgb8::f32x4_to_srgb8([r, g, b, 0.]);
    let (r, g, b) = (srgb[0] as usize, srgb[1] as usize, srgb[2] as usize);
    (r << 16) + (g << 8) + b
}

fn encode_ac((r, b, g): (f32, f32, f32), max_value: f32) -> usize {
    #[inline]
    fn quantize(value: f32, max_value: f32) -> usize {
        let value = value / max_value;
        f32::floor(f32::max(
            0.0,
            f32::min(
                18.0,
                f32::floor(value.signum() * value.abs().sqrt() * 9. + 9.5),
            ),
        )) as usize
    }

    quantize(r, max_value) * 19 * 19 + quantize(g, max_value) * 19 + quantize(b, max_value)
}

pub fn encode(component: Component, image: &RgbImage) -> String {
    let component_x = component.x() as usize;
    let component_y = component.y() as usize;

    let mut factors = (0..component_y)
        .into_iter()
        .flat_map(|y| (0..component_x).map(move |x| (x, y)))
        .fold(
            VecDeque::with_capacity(component_x * component_y),
            |mut acc, (x, y)| {
                acc.push_back(multiply_basis_function(image, x, y));
                acc
            },
        );

    // SAFETY: we can unwrap here since there are x_component * y_component number of factors
    let dc = factors.pop_front().expect("DC component is None");
    let ac = factors.make_contiguous();

    let mut hash = String::with_capacity(ac.len() + 2);

    let size_flag = component_x - 1 + (component_y - 1) * 9;
    hash.push(base83::encode_char(size_flag));

    let max_value = if ac.is_empty() {
        hash.push(base83::encode_char(0));
        1.
    } else {
        // SAFETY: we know that ac isn't empty so we can safely unwrap
        let max_value = ac
            .iter()
            .map(|(r, b, g)| r.max(*b).max(*g))
            .reduce(f32::max)
            .unwrap();

        let quantized_max_value = f32::floor(f32::max(
            0.,
            f32::min(82., f32::floor(max_value * 166. - 0.5)),
        ));
        hash.push(base83::encode_char(quantized_max_value as usize));

        (quantized_max_value + 1.0) / 166.0
    };

    base83::encode_into(&mut hash, encode_dc(dc), 4);

    (*ac)
        .iter()
        .for_each(|factor| base83::encode_into(&mut hash, encode_ac(*factor, max_value), 2));

    hash
}
