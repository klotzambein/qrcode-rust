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

#![cfg_attr(not(test), no_std)]

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
    content: Vec<u8, V::ColorSize>,
}

impl<V: QrSpec> QrCode<V> {
    /// Constructs a new QR code for the given version and error correction
    /// level.
    ///
    ///     use qrcode::{QrCode, Version, EcLevel};
    ///     use qrcode::spec::{Version1, EcLevelL};
    ///
    ///     let code = QrCode::<Version1<EcLevelL>>::new(b"Some data");
    ///
    /// This method can also be used to generate Micro QR code.
    pub fn new<D: AsRef<[u8]>>(data: D) -> QrResult<Self> {
        let mut bits = bits::Bits::new();
        bits.push_optimal_data(data.as_ref())?;
        bits.push_terminator()?;
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
    ///     use qrcode::spec::{Version1, EcLevelL};
    ///
    ///     let mut bits = Bits::<Version1<EcLevelL>>::new();
    ///     bits.push_eci_designator(9);
    ///     bits.push_byte_data(b"\xca\xfe\xe4\xe9\xea\xe1\xf2 QR");
    ///     bits.push_terminator();
    ///     let qrcode = QrCode::with_bits(bits);
    ///
    pub fn with_bits(bits: bits::Bits<V>) -> QrResult<Self> {
        let data = bits.into_bytes();
        let (data_ec, data_end) = ec::construct_codewords::<V>(&*data)?;
        let mut canvas = canvas::Canvas::<V>::new();
        canvas.draw_all_functional_patterns();
        canvas.draw_data(&data_ec[..data_end], &data_ec[data_end..]);
        let canvas = canvas.apply_best_mask();
        let content = canvas.colors_bits();
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

    /// Converts the QR code into a human-readable string. This is mainly for
    /// debugging only.
    #[cfg(test)]
    pub fn to_debug_str(&self, dark_char: char, light_char: char) -> String {
        let mut buffer = String::with_capacity((V::WIDTH * (V::WIDTH + 1)) as usize);
        for (i, c) in self.colors().enumerate() {
            if i == V::AREA {
                break;
            }
            if i % V::WIDTH as usize == 0 {
                buffer.push('\n');
            }
            buffer.push(c.select(dark_char, light_char));
        }
        buffer
    }

    /// Converts the QR code to a vector of colors.
    pub fn colors(&self) -> impl Iterator<Item = Color> + '_ {
        struct BitIter(u8, u8);
        impl Iterator for BitIter {
            type Item = u8;
            fn next(&mut self) -> Option<u8> {
                if self.1 >= 8 {
                    None
                } else {
                    let result = self.0 >> self.1;
                    self.1 = self.1.wrapping_sub(1);
                    Some(result)
                }
            }
        }

        self.content.iter().flat_map(|b| BitIter(*b, 7)).map(Color::from_bit)
    }

    // /// Converts the QR code to a vector of colors.
    // pub fn colors(self) -> Vec<Color, V::ColorSize> {
    //     self.content
    // }
}

#[cfg(test)]
mod tests {
    use crate::{QrCode};
    use crate::spec::{Version1, EcLevelM};

    #[test]
    fn test_annex_i_qr() {
        // This uses the ISO Annex I as test vector.
        let code = QrCode::<Version1<EcLevelM>>::new(b"01234567").unwrap();
        assert_eq!(
            &*code.to_debug_str('#', '.'),
            "\n\
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

    // #[test]
    // fn test_annex_i_micro_qr() {
    //     let code = QrCode::with_version(b"01234567", Version::Micro(2), EcLevel::L).unwrap();
    //     assert_eq!(
    //         &*code.to_debug_str('#', '.'),
    //         "\
    //          #######.#.#.#\n\
    //          #.....#.###.#\n\
    //          #.###.#..##.#\n\
    //          #.###.#..####\n\
    //          #.###.#.###..\n\
    //          #.....#.#...#\n\
    //          #######..####\n\
    //          .........##..\n\
    //          ##.#....#...#\n\
    //          .##.#.#.#.#.#\n\
    //          ###..#######.\n\
    //          ...#.#....##.\n\
    //          ###.#..##.###"
    //     );
    // }
}