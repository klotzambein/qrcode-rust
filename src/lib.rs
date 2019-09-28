//! QRCode encoder
//!
//! This crate provides a QR code and Micro QR code encoder for binary data.
//!
//!```ignore
//! extern crate qrcode;
//! extern crate image;
//!
//! use qrcode::QrCode;
//! use image::Luma;
//!
//! fn main() {
//!     // Encode some data into bits.
//!     let code = QrCode::new(b"01234567").unwrap();
//!
//!     // Render the bits into an image.
//!     let image = code.render::<Luma<u8>>().build();
//!
//!     // Save the image.
//!     image.save("/tmp/qrcode.png").unwrap();
//!
//!     // You can also render it into a string.
//!     let string = code.render()
//!         .light_color(' ')
//!         .dark_color('#')
//!         .build();
//!     println!("{}", string);
//! }
//! ```

#![no_std]

use core::ops::Index;

pub mod bits;
pub mod canvas;
mod cast;
pub mod ec;
pub mod optimize;
pub mod spec;
pub mod types;

use spec::QrSpec;
pub use types::{Color, EcLevel, QrResult, Version};

use checked_int_cast::CheckedIntCast;
use heapless::Vec;

/// The encoded QR code symbol.
#[derive(Clone)]
pub struct QrCode<V: QrSpec> {
    content: Vec<Color, V::ColorSize>,
}

impl<V: QrSpec> QrCode<V> {
    /// Constructs a new QR code for the given version and error correction
    /// level.
    ///
    ///     use qrcode::{QrCode, Version, EcLevel};
    ///
    ///     let code = QrCode::with_version(b"Some data", Version::Normal(5), EcLevel::M).unwrap();
    ///
    /// This method can also be used to generate Micro QR code.
    ///
    ///     use qrcode::{QrCode, Version, EcLevel};
    ///
    ///     let micro_code = QrCode::with_version(b"123", Version::Micro(1), EcLevel::L).unwrap();
    ///
    pub fn new<D: AsRef<[u8]>>(data: D) -> QrResult<Self> {
        let mut bits = bits::Bits::new();
        bits.push_optimal_data(data.as_ref())?;
        bits.push_terminator(V::EC_LEVEL)?;
        Self::with_bits(bits)
    }

    /// Constructs a new QR code with encoded bits.
    ///
    /// Use this method only if there are very special need to manipulate the
    /// raw bits before encoding. Some examples are:
    ///
    /// * Encode data using specific character set with ECI
    /// * Use the FNC1 modes
    /// * Avoid the optimal segmentation algorithm
    ///
    /// See the `Bits` structure for detail.
    ///
    ///     #![allow(unused_must_use)]
    ///
    ///     use qrcode::{QrCode, Version, EcLevel};
    ///     use qrcode::bits::Bits;
    ///
    ///     let mut bits = Bits::new(Version::Normal(1));
    ///     bits.push_eci_designator(9);
    ///     bits.push_byte_data(b"\xca\xfe\xe4\xe9\xea\xe1\xf2 QR");
    ///     bits.push_terminator(EcLevel::L);
    ///     let qrcode = QrCode::with_bits(bits, EcLevel::L);
    ///
    pub fn with_bits(bits: bits::Bits<V>) -> QrResult<Self> {
        let data = bits.into_bytes();
        let (data_ec, data_end) = ec::construct_codewords::<V>(&*data)?;
        let mut canvas = canvas::Canvas::<V>::new();
        canvas.draw_all_functional_patterns();
        canvas.draw_data(&data_ec[..data_end], &data_ec[data_end..]);
        let canvas = canvas.apply_best_mask();
        let mut content = Vec::new();
        content.extend(canvas.into_colors());
        Ok(Self { content })
    }

    /// Gets the maximum number of allowed erratic modules can be introduced
    /// before the data becomes corrupted. Note that errors should not be
    /// introduced to functional modules.
    pub fn max_allowed_errors(&self) -> usize {
        ec::max_allowed_errors::<V>().expect("invalid version or ec_level")
    }

    /// Checks whether a module at coordinate (x, y) is a functional module or
    /// not.
    pub fn is_functional(&self, x: usize, y: usize) -> bool {
        let x = x.as_i16_checked().expect("coordinate is too large for QR code");
        let y = y.as_i16_checked().expect("coordinate is too large for QR code");
        canvas::is_functional(V::VERSION, V::WIDTH, x, y)
    }

    // /// Converts the QR code into a human-readable string. This is mainly for
    // /// debugging only.
    // pub fn to_debug_str(&self, on_char: char, off_char: char) -> String {
    //     self.render().quiet_zone(false).dark_color(on_char).light_color(off_char).build()
    // }

    /// Converts the QR code to a vector of colors.
    pub fn to_colors(&self) -> impl Iterator<Item = Color> + '_ {
        self.content.iter().cloned()
    }

    /// Converts the QR code to a vector of colors.
    pub fn into_colors(self) -> Vec<Color, V::ColorSize> {
        self.content
    }
}

impl<V: QrSpec> Index<(usize, usize)> for QrCode<V> {
    type Output = Color;

    fn index(&self, (x, y): (usize, usize)) -> &Color {
        let index = y * V::WIDTH as usize + x;
        &self.content[index]
    }
}

/*
#[cfg(test)]
mod tests {
    use crate::{EcLevel, QrCode, Version};

    #[test]
    fn test_annex_i_qr() {
        // This uses the ISO Annex I as test vector.
        let code = QrCode::with_version(b"01234567", Version::Normal(1), EcLevel::M).unwrap();
        assert_eq!(
            &*code.to_debug_str('#', '.'),
            "\
             #######..#.##.#######\n\
             #.....#..####.#.....#\n\
             #.###.#.#.....#.###.#\n\
             #.###.#.##....#.###.#\n\
             #.###.#.#.###.#.###.#\n\
             #.....#.#...#.#.....#\n\
             #######.#.#.#.#######\n\
             ........#..##........\n\
             #.#####..#..#.#####..\n\
             ...#.#.##.#.#..#.##..\n\
             ..#...##.#.#.#..#####\n\
             ....#....#.....####..\n\
             ...######..#.#..#....\n\
             ........#.#####..##..\n\
             #######..##.#.##.....\n\
             #.....#.#.#####...#.#\n\
             #.###.#.#...#..#.##..\n\
             #.###.#.##..#..#.....\n\
             #.###.#.#.##.#..#.#..\n\
             #.....#........##.##.\n\
             #######.####.#..#.#.."
        );
    }

    #[test]
    fn test_annex_i_micro_qr() {
        let code = QrCode::with_version(b"01234567", Version::Micro(2), EcLevel::L).unwrap();
        assert_eq!(
            &*code.to_debug_str('#', '.'),
            "\
             #######.#.#.#\n\
             #.....#.###.#\n\
             #.###.#..##.#\n\
             #.###.#..####\n\
             #.###.#.###..\n\
             #.....#.#...#\n\
             #######..####\n\
             .........##..\n\
             ##.#....#...#\n\
             .##.#.#.#.#.#\n\
             ###..#######.\n\
             ...#.#....##.\n\
             ###.#..##.###"
        );
    }
}
*/

#[cfg(all(test, feature = "image"))]
mod image_tests {
    use crate::{EcLevel, QrCode, Version};
    use image::{load_from_memory, Luma, Rgb};

    #[test]
    fn test_annex_i_qr_as_image() {
        let code = QrCode::new(b"01234567").unwrap();
        let image = code.render::<Luma<u8>>().build();
        let expected = load_from_memory(include_bytes!("test_annex_i_qr_as_image.png")).unwrap().to_luma();
        assert_eq!(image.dimensions(), expected.dimensions());
        assert_eq!(image.into_raw(), expected.into_raw());
    }

    #[test]
    fn test_annex_i_micro_qr_as_image() {
        let code = QrCode::with_version(b"01234567", Version::Micro(2), EcLevel::L).unwrap();
        let image = code
            .render()
            .min_dimensions(200, 200)
            .dark_color(Rgb([128, 0, 0]))
            .light_color(Rgb([255, 255, 128]))
            .build();
        let expected = load_from_memory(include_bytes!("test_annex_i_micro_qr_as_image.png")).unwrap().to_rgb();
        assert_eq!(image.dimensions(), expected.dimensions());
        assert_eq!(image.into_raw(), expected.into_raw());
    }
}

#[cfg(all(test, feature = "svg"))]
mod svg_tests {
    use crate::render::svg::Color as SvgColor;
    use crate::{EcLevel, QrCode, Version};

    #[test]
    fn test_annex_i_qr_as_svg() {
        let code = QrCode::new(b"01234567").unwrap();
        let image = code.render::<SvgColor>().build();
        let expected = include_str!("test_annex_i_qr_as_svg.svg");
        assert_eq!(&image, expected);
    }

    #[test]
    fn test_annex_i_micro_qr_as_svg() {
        let code = QrCode::with_version(b"01234567", Version::Micro(2), EcLevel::L).unwrap();
        let image = code
            .render()
            .min_dimensions(200, 200)
            .dark_color(SvgColor("#800000"))
            .light_color(SvgColor("#ffff80"))
            .build();
        let expected = include_str!("test_annex_i_micro_qr_as_svg.svg");
        assert_eq!(&image, expected);
    }
}
