use std::f32::consts::PI;

use fast_srgb8::{f32_to_srgb8, srgb8_to_f32};
use image::RgbImage;

use crate::{base83, Component};

// fn srgb8_to_f32(value: u8) -> f32 {
//     let v = value as f32 / 255.;
//     if (v <= 0.04045) {
//         v / 12.92
//     } else {
//         f32::powf((v + 0.055) / 1.055, 2.4)
//     }
// }

// fn f32_to_srgb8(value: f32) -> u8 {
//     let v = f32::max(0., f32::min(1., value));
//     if (v <= 0.0031308) {
//         f32::trunc(v * 12.92 * 255. + 0.5) as u8
//     } else {
//         f32::trunc((1.055 * f32::powf(v, 1. / 2.4) - 0.055) * 255. + 0.5) as u8
//     }
// }

#[inline]
fn basis_fn(
    component_x: usize,
    component_y: usize,
    point_x: u32,
    point_y: u32,
    width: f32,
    height: f32,
) -> f32 {
    let normalizer = if component_x == 0 && component_y == 0 {
        1
    } else {
        2
    } as f32;

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
    let (mut r, mut g, mut b) = (0., 0., 0.);

    let dimensions = image.dimensions();
    let width = dimensions.0;
    let height = dimensions.1;

    let mut i = 0;

    image.enumerate_pixels().for_each(|(x, y, pixel)| {
        i += 1;
        // let pixel = image.get_pixel(x, y);
        let basis = basis_fn(component_x, component_y, x, y, width as f32, height as f32);

        r += basis * srgb8_to_f32(pixel[0]);
        g += basis * srgb8_to_f32(pixel[1]);
        b += basis * srgb8_to_f32(pixel[2]);
    });

    let scale = 1. / (width * height) as f32;

    (r * scale, g * scale, b * scale)
}

#[inline]
fn encode_dc((r, g, b): (f32, f32, f32)) -> usize {
    let r = f32_to_srgb8(r) as usize;
    let g = f32_to_srgb8(g) as usize;
    let b = f32_to_srgb8(b) as usize;

    (r << 16) + (g << 8) + b
}

fn encode_ac((r, g, b): (f32, f32, f32), max_value: f32) -> usize {
    #[inline]
    fn quantize(value: f32, max_value: f32) -> usize {
        let value = value / max_value;
        f32::floor(f32::max(
            0.0,
            f32::min(
                18.0,
                f32::floor(value.abs().sqrt().copysign(value) * 9. + 9.5),
            ),
        )) as usize
    }

    quantize(r, max_value) * 19 * 19 + quantize(g, max_value) * 19 + quantize(b, max_value)
}

pub fn encode(component: Component, image: &RgbImage) -> String {
    let component_x = component.x() as usize;
    let component_y = component.y() as usize;

    let factors = (0..component_y)
        .into_iter()
        .flat_map(|y| (0..component_x).map(move |x| (x, y)))
        .fold(
            Vec::with_capacity(component_x * component_y),
            |mut acc, (x, y)| {
                acc.push(multiply_basis_function(image, x, y));
                acc
            },
        );

    let dc = factors[0];
    let ac = &factors[1..];

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
            .map(|(r, g, b)| r.max(*g).max(*b))
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

#[cfg(test)]
mod test_encode {
    use super::*;

    #[test]
    fn test_pic1() {
        let bytes = include_bytes!("../resources/tests/pic1.png");

        let image = image::load_from_memory(bytes).unwrap();
        let buffer = image.into_rgb8();

        let output = encode(Component::try_new(1, 1).unwrap(), &buffer);

        assert_eq!(output, "00JHjm");
    }
}
