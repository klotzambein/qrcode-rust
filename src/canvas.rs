//! The `canvas` module puts raw bits into the QR code canvas.
//!
//!     use qrcode::types::{Version, EcLevel};
//!     use qrcode::canvas::{Canvas, MaskPattern};
//!     use qrcode::spec::{Version1, EcLevelL};
//!
//!     let mut c = Canvas::<Version1<EcLevelL>>::new();
//!     c.draw_all_functional_patterns();
//!     c.draw_data(b"data_here", b"ec_code_here");
//!     c.apply_mask(MaskPattern::Checkerboard);

use core::cmp::max;

use crate::cast::As;
use crate::spec::QrSpec;
use crate::types::{Color, EcLevel, Version};

use heapless::Vec;

//------------------------------------------------------------------------------
//{{{ Modules

/// The color of a module (pixel) in the QR code.
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Module {
    /// The module is of functional patterns which cannot be masked, or pixels
    /// which have been masked.
    Masked(Color),

    /// The module is of data and error correction bits before masking.
    Unmasked(Color),
}

impl From<Module> for Color {
    fn from(module: Module) -> Self {
        match module {
            Module::Masked(c) | Module::Unmasked(c) => c,
        }
    }
}

impl Module {
    const EMPTY: Module = Module::Unmasked(Color::Light);

    /// Checks whether a module is dark.
    pub fn is_dark(self) -> bool {
        Color::from(self) == Color::Dark
    }

    pub fn from_bits(bits: u8) -> Module {
        match bits & 0b11 {
            0 => Module::Unmasked(Color::Light),
            1 => Module::Unmasked(Color::Dark),
            2 => Module::Masked(Color::Light),
            3 => Module::Masked(Color::Dark),
            _ => unreachable!(),
        }
    }

    pub fn from_u8(data: u8, index: u8) -> Module {
        Module::from_bits(data >> (index * 2))
    }

    pub fn to_bits(self) -> u8 {
        match self {
            Module::Unmasked(Color::Light) => 0,
            Module::Unmasked(Color::Dark) => 1,
            Module::Masked(Color::Light) => 2,
            Module::Masked(Color::Dark) => 3,
        }
    }

    pub fn write(self, target: &mut u8, index: u8) {
        let index = index * 2;
        *target &= !(0b11 << index);
        *target |= self.to_bits() << index;
    }

    pub fn from_iter<'s>(iter: impl Iterator<Item = u8> + 's, len: usize) -> impl Iterator<Item = Module> + 's {
        struct U82bitIter(u8, u8);
        impl Iterator for U82bitIter {
            type Item = u8;
            fn next(&mut self) -> Option<u8> {
                if self.1 == 8 {
                    None
                } else {
                    let result = self.0 >> self.1;
                    self.1 += 2;
                    Some(result)
                }
            }
        }

        iter.flat_map(|x| U82bitIter(x, 0)).map(|m| Module::from_bits(m)).take(len)
    }

    /// Apply a mask to the unmasked modules.
    ///
    ///     use qrcode::canvas::Module;
    ///     use qrcode::types::Color;
    ///
    ///     assert_eq!(Module::Unmasked(Color::Light).mask(true), Color::Dark);
    ///     assert_eq!(Module::Unmasked(Color::Dark).mask(true), Color::Light);
    ///     assert_eq!(Module::Unmasked(Color::Light).mask(false), Color::Light);
    ///     assert_eq!(Module::Masked(Color::Dark).mask(true), Color::Dark);
    ///     assert_eq!(Module::Masked(Color::Dark).mask(false), Color::Dark);
    ///
    pub fn mask(self, should_invert: bool) -> Color {
        match (self, should_invert) {
            (Module::Unmasked(c), true) => !c,
            (Module::Unmasked(c), false) | (Module::Masked(c), _) => c,
        }
    }
}

//}}}
//------------------------------------------------------------------------------
//{{{ Canvas

/// `Canvas` is an intermediate helper structure to render error-corrected data
/// into a QR code.
pub struct Canvas<V: QrSpec> {
    /// The modules of the QR code. Modules are arranged in left-to-right, then
    /// top-to-bottom order.
    modules: Vec<u8, V::CanvasSize>,
}

impl<V: QrSpec> Clone for Canvas<V> {
    fn clone(&self) -> Self {
        Canvas { modules: self.modules.clone() }
    }
}

impl<V: QrSpec> Canvas<V> {
    /// Constructs a new canvas big enough for a QR code of the given version.
    pub fn new() -> Self {
        let mut modules = Vec::new();
        modules.resize((V::WIDTH * V::WIDTH).as_usize() / 4 + 1, 0).unwrap();
        Self { modules }
    }

    /// Converts the canvas into a human-readable string.
    #[cfg(test)]
    pub(crate) fn to_debug_str(&self) -> String {
        let width = V::WIDTH;
        let mut res = String::with_capacity((width * (width + 1)) as usize);
        for y in 0..width {
            res.push('\n');
            for x in 0..width {
                res.push(match self.get(x, y) {
                    Module::Masked(Color::Light) => '.',
                    Module::Masked(Color::Dark) => '#',
                    Module::Unmasked(Color::Light) => '-',
                    Module::Unmasked(Color::Dark) => '*',
                });
            }
        }
        res
    }

    fn coords_to_index(&self, x: i16, y: i16) -> (usize, u8) {
        let x = if x < 0 { x + V::WIDTH } else { x }.as_usize();
        let y = if y < 0 { y + V::WIDTH } else { y }.as_usize();
        let index = y * V::WIDTH.as_usize() + x;
        (index / 4, (index % 4) as u8)
    }

    /// Obtains a module at the given coordinates. For convenience, negative
    /// coordinates will wrap around.
    pub fn get(&self, x: i16, y: i16) -> Module {
        let (index, sub_index) = self.coords_to_index(x, y);
        Module::from_u8(self.modules[index], sub_index)
    }

    /// Sets the color of a functional module at the given coordinates. For
    /// convenience, negative coordinates will wrap around.
    pub fn put(&mut self, x: i16, y: i16, color: Color) {
        let (index, sub_index) = self.coords_to_index(x, y);
        Module::Masked(color).write(&mut self.modules[index], sub_index);
    }

    /// Sets the color of a functional module at the given coordinates. For
    /// convenience, negative coordinates will wrap around.
    pub fn put_unmasked(&mut self, x: i16, y: i16, color: Color) {
        let (index, sub_index) = self.coords_to_index(x, y);
        Module::Unmasked(color).write(&mut self.modules[index], sub_index);
    }
}

#[cfg(test)]
mod basic_canvas_tests {
    use crate::canvas::{Canvas, Module};
    use crate::types::Color;
    use crate::spec::{Version1, EcLevelL};

    #[test]
    fn test_index() {
        let mut c = Canvas::<Version1<EcLevelL>>::new();

        assert_eq!(c.get(0, 4), Module::Unmasked(Color::Light));
        assert_eq!(c.get(-1, -7), Module::Unmasked(Color::Light));
        assert_eq!(c.get(21 - 1, 21 - 7), Module::Unmasked(Color::Light));

        c.put(0, 0, Color::Dark);
        c.put(-1, -7, Color::Light);
        assert_eq!(c.get(0, 0), Module::Masked(Color::Dark));
        assert_eq!(c.get(21 - 1, -7), Module::Masked(Color::Light));
        assert_eq!(c.get(-1, 21 - 7), Module::Masked(Color::Light));
    }

    #[test]
    fn test_debug_str() {
        let mut c = Canvas::<Version1<EcLevelL>>::new();

        for i in 3_i16..20 {
            for j in 3_i16..20 {
                match ((i * 3) ^ j) % 5 {
                    0 => c.put_unmasked(i, j, Color::Light),
                    1 => c.put(i, j, Color::Light),
                    2 => c.put(i, j, Color::Dark),
                    3 => c.put_unmasked(i, j, Color::Light),
                    4 => c.put_unmasked(i, j, Color::Dark),
                    _ => unreachable!(),
                };
            }
        }

        assert_eq!(
            &*c.to_debug_str(),
            "\n\
             ---------------------\n\
             ---------------------\n\
             ---------------------\n\
             -----####****....----\n\
             -----.##-..##-..#--.-\n\
             ---#*--.*-#.-*#--*.--\n\
             -----*-*-****-*-*----\n\
             ---*.-.-.----#-#-#*#-\n\
             ---.*#.*.*#.*#*#.*#*-\n\
             -----.#-#----.-.#----\n\
             ----.-*.-#--.-#*-#-.-\n\
             ---##*--*..##*--*..--\n\
             ---------------------\n\
             ---*.#.*.#**.#*#.#*#-\n\
             ---##.-##..##..-#..--\n\
             ---.--*.--#.--#*--#*-\n\
             -----.#--.**#--.#--.-\n\
             ---**--**----**--**--\n\
             ---#-*-#-*#.*-.-*-.--\n\
             ---..-...----###-###-\n\
             ---------------------"
        );
    }
}

//}}}
//------------------------------------------------------------------------------
//{{{ Finder patterns

impl<V: QrSpec> Canvas<V> {
    /// Draws a single finder pattern with the center at (x, y).
    fn draw_finder_pattern_at(&mut self, x: i16, y: i16) {
        let (dx_left, dx_right) = if x >= 0 { (-3, 4) } else { (-4, 3) };
        let (dy_top, dy_bottom) = if y >= 0 { (-3, 4) } else { (-4, 3) };
        for j in dy_top..=dy_bottom {
            for i in dx_left..=dx_right {
                self.put(x + i, y + j, {
                    #[cfg_attr(feature = "cargo-clippy", allow(match_same_arms))]
                    match (i, j) {
                        (4, _) | (_, 4) | (-4, _) | (_, -4) => Color::Light,
                        (3, _) | (_, 3) | (-3, _) | (_, -3) => Color::Dark,
                        (2, _) | (_, 2) | (-2, _) | (_, -2) => Color::Light,
                        _ => Color::Dark,
                    }
                });
            }
        }
    }

    /// Draws the finder patterns.
    ///
    /// The finder patterns is are 7×7 square patterns appearing at the three
    /// corners of a QR code. They allows scanner to locate the QR code and
    /// determine the orientation.
    fn draw_finder_patterns(&mut self) {
        self.draw_finder_pattern_at(3, 3);

        match V::VERSION {
            Version::Micro(_) => {}
            Version::Normal(_) => {
                self.draw_finder_pattern_at(-4, 3);
                self.draw_finder_pattern_at(3, -4);
            }
        }
    }
}

#[cfg(test)]
mod finder_pattern_tests {
    use crate::canvas::Canvas;
    use crate::spec::{Version1, EcLevelL};

    #[test]
    fn test_qr() {
        let mut c = Canvas::<Version1<EcLevelL>>::new();
        c.draw_finder_patterns();
        assert_eq!(
            &*c.to_debug_str(),
            "\n\
             #######.-----.#######\n\
             #.....#.-----.#.....#\n\
             #.###.#.-----.#.###.#\n\
             #.###.#.-----.#.###.#\n\
             #.###.#.-----.#.###.#\n\
             #.....#.-----.#.....#\n\
             #######.-----.#######\n\
             ........-----........\n\
             ---------------------\n\
             ---------------------\n\
             ---------------------\n\
             ---------------------\n\
             ---------------------\n\
             ........-------------\n\
             #######.-------------\n\
             #.....#.-------------\n\
             #.###.#.-------------\n\
             #.###.#.-------------\n\
             #.###.#.-------------\n\
             #.....#.-------------\n\
             #######.-------------"
        );
    }

    // #[test]
    // fn test_micro_qr() {
    //     let mut c = Canvas::new(Version::Micro(1), EcLevel::L);
    //     c.draw_finder_patterns();
    //     assert_eq!(
    //         &*c.to_debug_str(),
    //         "\n\
    //          #######.---\n\
    //          #.....#.---\n\
    //          #.###.#.---\n\
    //          #.###.#.---\n\
    //          #.###.#.---\n\
    //          #.....#.---\n\
    //          #######.---\n\
    //          ........---\n\
    //          -----------\n\
    //          -----------\n\
    //          -----------"
    //     );
    // }
}


//}}}
//------------------------------------------------------------------------------
//{{{ Alignment patterns

impl<V: QrSpec> Canvas<V> {
    /// Draws a alignment pattern with the center at (x, y).
    fn draw_alignment_pattern_at(&mut self, x: i16, y: i16) {
        if let Module::Masked(_) = self.get(x, y) {
            return;
        }
        for j in -2..=2 {
            for i in -2..=2 {
                self.put(
                    x + i,
                    y + j,
                    match (i, j) {
                        (2, _) | (_, 2) | (-2, _) | (_, -2) | (0, 0) => Color::Dark,
                        _ => Color::Light,
                    },
                );
            }
        }
    }

    /// Draws the alignment patterns.
    ///
    /// The alignment patterns are 5×5 square patterns inside the QR code symbol
    /// to help the scanner create the square grid.
    fn draw_alignment_patterns(&mut self) {
        match V::VERSION {
            Version::Micro(_) | Version::Normal(1) => {}
            Version::Normal(2..=6) => self.draw_alignment_pattern_at(-7, -7),
            Version::Normal(a) => {
                let positions = ALIGNMENT_PATTERN_POSITIONS[(a - 7).as_usize()];
                for x in positions.iter() {
                    for y in positions.iter() {
                        self.draw_alignment_pattern_at(*x, *y);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod alignment_pattern_tests {
    use crate::canvas::Canvas;
    use crate::spec::{Version1, Version3, Version7, EcLevelL};

    #[test]
    fn test_draw_alignment_patterns_1() {
        let mut c = Canvas::<Version1<EcLevelL>>::new();
        c.draw_finder_patterns();
        c.draw_alignment_patterns();
        assert_eq!(
            &*c.to_debug_str(),
            "\n\
             #######.-----.#######\n\
             #.....#.-----.#.....#\n\
             #.###.#.-----.#.###.#\n\
             #.###.#.-----.#.###.#\n\
             #.###.#.-----.#.###.#\n\
             #.....#.-----.#.....#\n\
             #######.-----.#######\n\
             ........-----........\n\
             ---------------------\n\
             ---------------------\n\
             ---------------------\n\
             ---------------------\n\
             ---------------------\n\
             ........-------------\n\
             #######.-------------\n\
             #.....#.-------------\n\
             #.###.#.-------------\n\
             #.###.#.-------------\n\
             #.###.#.-------------\n\
             #.....#.-------------\n\
             #######.-------------"
        );
    }

    #[test]
    fn test_draw_alignment_patterns_3() {
        let mut c = Canvas::<Version3<EcLevelL>>::new();
        c.draw_finder_patterns();
        c.draw_alignment_patterns();
        assert_eq!(
            &*c.to_debug_str(),
            "\n\
             #######.-------------.#######\n\
             #.....#.-------------.#.....#\n\
             #.###.#.-------------.#.###.#\n\
             #.###.#.-------------.#.###.#\n\
             #.###.#.-------------.#.###.#\n\
             #.....#.-------------.#.....#\n\
             #######.-------------.#######\n\
             ........-------------........\n\
             -----------------------------\n\
             -----------------------------\n\
             -----------------------------\n\
             -----------------------------\n\
             -----------------------------\n\
             -----------------------------\n\
             -----------------------------\n\
             -----------------------------\n\
             -----------------------------\n\
             -----------------------------\n\
             -----------------------------\n\
             -----------------------------\n\
             --------------------#####----\n\
             ........------------#...#----\n\
             #######.------------#.#.#----\n\
             #.....#.------------#...#----\n\
             #.###.#.------------#####----\n\
             #.###.#.---------------------\n\
             #.###.#.---------------------\n\
             #.....#.---------------------\n\
             #######.---------------------"
        );
    }

    #[test]
    fn test_draw_alignment_patterns_7() {
        let mut c = Canvas::<Version7<EcLevelL>>::new();
        c.draw_finder_patterns();
        c.draw_alignment_patterns();
        assert_eq!(
            &*c.to_debug_str(),
            "\n\
             #######.-----------------------------.#######\n\
             #.....#.-----------------------------.#.....#\n\
             #.###.#.-----------------------------.#.###.#\n\
             #.###.#.-----------------------------.#.###.#\n\
             #.###.#.------------#####------------.#.###.#\n\
             #.....#.------------#...#------------.#.....#\n\
             #######.------------#.#.#------------.#######\n\
             ........------------#...#------------........\n\
             --------------------#####--------------------\n\
             ---------------------------------------------\n\
             ---------------------------------------------\n\
             ---------------------------------------------\n\
             ---------------------------------------------\n\
             ---------------------------------------------\n\
             ---------------------------------------------\n\
             ---------------------------------------------\n\
             ---------------------------------------------\n\
             ---------------------------------------------\n\
             ---------------------------------------------\n\
             ---------------------------------------------\n\
             ----#####-----------#####-----------#####----\n\
             ----#...#-----------#...#-----------#...#----\n\
             ----#.#.#-----------#.#.#-----------#.#.#----\n\
             ----#...#-----------#...#-----------#...#----\n\
             ----#####-----------#####-----------#####----\n\
             ---------------------------------------------\n\
             ---------------------------------------------\n\
             ---------------------------------------------\n\
             ---------------------------------------------\n\
             ---------------------------------------------\n\
             ---------------------------------------------\n\
             ---------------------------------------------\n\
             ---------------------------------------------\n\
             ---------------------------------------------\n\
             ---------------------------------------------\n\
             ---------------------------------------------\n\
             --------------------#####-----------#####----\n\
             ........------------#...#-----------#...#----\n\
             #######.------------#.#.#-----------#.#.#----\n\
             #.....#.------------#...#-----------#...#----\n\
             #.###.#.------------#####-----------#####----\n\
             #.###.#.-------------------------------------\n\
             #.###.#.-------------------------------------\n\
             #.....#.-------------------------------------\n\
             #######.-------------------------------------"
        );
    }
}


/// `ALIGNMENT_PATTERN_POSITIONS` describes the x- and y-coordinates of the
/// center of the alignment patterns. Since the QR code is symmetric, only one
/// coordinate is needed.
static ALIGNMENT_PATTERN_POSITIONS: [&'static [i16]; 34] = [
    &[6, 22, 38],
    &[6, 24, 42],
    &[6, 26, 46],
    &[6, 28, 50],
    &[6, 30, 54],
    &[6, 32, 58],
    &[6, 34, 62],
    &[6, 26, 46, 66],
    &[6, 26, 48, 70],
    &[6, 26, 50, 74],
    &[6, 30, 54, 78],
    &[6, 30, 56, 82],
    &[6, 30, 58, 86],
    &[6, 34, 62, 90],
    &[6, 28, 50, 72, 94],
    &[6, 26, 50, 74, 98],
    &[6, 30, 54, 78, 102],
    &[6, 28, 54, 80, 106],
    &[6, 32, 58, 84, 110],
    &[6, 30, 58, 86, 114],
    &[6, 34, 62, 90, 118],
    &[6, 26, 50, 74, 98, 122],
    &[6, 30, 54, 78, 102, 126],
    &[6, 26, 52, 78, 104, 130],
    &[6, 30, 56, 82, 108, 134],
    &[6, 34, 60, 86, 112, 138],
    &[6, 30, 58, 86, 114, 142],
    &[6, 34, 62, 90, 118, 146],
    &[6, 30, 54, 78, 102, 126, 150],
    &[6, 24, 50, 76, 102, 128, 154],
    &[6, 28, 54, 80, 106, 132, 158],
    &[6, 32, 58, 84, 110, 136, 162],
    &[6, 26, 54, 82, 110, 138, 166],
    &[6, 30, 58, 86, 114, 142, 170],
];
//}}}
//------------------------------------------------------------------------------
//{{{ Timing patterns

impl<V: QrSpec> Canvas<V> {
    /// Draws a line from (x1, y1) to (x2, y2), inclusively.
    ///
    /// The line must be either horizontal or vertical, i.e.
    /// `x1 == x2 || y1 == y2`. Additionally, the first coordinates must be less
    /// then the second ones.
    ///
    /// On even coordinates, `color_even` will be plotted; on odd coordinates,
    /// `color_odd` will be plotted instead. Thus the timing pattern can be
    /// drawn using this method.
    ///
    fn draw_line(&mut self, x1: i16, y1: i16, x2: i16, y2: i16, color_even: Color, color_odd: Color) {
        debug_assert!(x1 == x2 || y1 == y2);

        if y1 == y2 {
            // Horizontal line.
            for x in x1..=x2 {
                self.put(x, y1, if x % 2 == 0 { color_even } else { color_odd });
            }
        } else {
            // Vertical line.
            for y in y1..=y2 {
                self.put(x1, y, if y % 2 == 0 { color_even } else { color_odd });
            }
        }
    }

    /// Draws the timing patterns.
    ///
    /// The timing patterns are checkerboard-colored lines near the edge of the QR
    /// code symbol, to establish the fine-grained module coordinates when
    /// scanning.
    fn draw_timing_patterns(&mut self) {
        let width = V::WIDTH;
        let (y, x1, x2) = match V::VERSION {
            Version::Micro(_) => (0, 8, width - 1),
            Version::Normal(_) => (6, 8, width - 9),
        };
        self.draw_line(x1, y, x2, y, Color::Dark, Color::Light);
        self.draw_line(y, x1, y, x2, Color::Dark, Color::Light);
    }
}

#[cfg(test)]
mod timing_pattern_tests {
    use crate::canvas::Canvas;
    use crate::spec::{Version1, EcLevelL};

    #[test]
    fn test_draw_timing_patterns_qr() {
        let mut c = Canvas::<Version1<EcLevelL>>::new();
        c.draw_timing_patterns();
        assert_eq!(
            &*c.to_debug_str(),
            "\n\
             ---------------------\n\
             ---------------------\n\
             ---------------------\n\
             ---------------------\n\
             ---------------------\n\
             ---------------------\n\
             --------#.#.#--------\n\
             ---------------------\n\
             ------#--------------\n\
             ------.--------------\n\
             ------#--------------\n\
             ------.--------------\n\
             ------#--------------\n\
             ---------------------\n\
             ---------------------\n\
             ---------------------\n\
             ---------------------\n\
             ---------------------\n\
             ---------------------\n\
             ---------------------\n\
             ---------------------"
        );
    }

    // #[test]
    // fn test_draw_timing_patterns_micro_qr() {
    //     let mut c = Canvas::new(Version::Micro(1), EcLevel::L);
    //     c.draw_timing_patterns();
    //     assert_eq!(
    //         &*c.to_debug_str(),
    //         "\n\
    //          --------#.#\n\
    //          -----------\n\
    //          -----------\n\
    //          -----------\n\
    //          -----------\n\
    //          -----------\n\
    //          -----------\n\
    //          -----------\n\
    //          #----------\n\
    //          .----------\n\
    //          #----------"
    //     );
    // }
}


//}}}
//------------------------------------------------------------------------------
//{{{ Format info & Version info

impl<V: QrSpec> Canvas<V> {
    /// Draws a big-endian integer onto the canvas with the given coordinates.
    ///
    /// The 1 bits will be plotted with `on_color` and the 0 bits with
    /// `off_color`. The coordinates will be extracted from the `coords`
    /// iterator. It will start from the most significant bits first, so
    /// *trailing* zeros will be ignored.
    fn draw_number(&mut self, number: u32, bits: u32, on_color: Color, off_color: Color, coords: &[(i16, i16)]) {
        let mut mask = 1 << (bits - 1);
        for &(x, y) in coords {
            let color = if (mask & number) == 0 { off_color } else { on_color };
            self.put(x, y, color);
            mask >>= 1;
        }
    }

    /// Draws the format info patterns for an encoded number.
    fn draw_format_info_patterns_with_number(&mut self, format_info: u16) {
        let format_info = u32::from(format_info);
        match V::VERSION {
            Version::Micro(_) => {
                self.draw_number(format_info, 15, Color::Dark, Color::Light, &FORMAT_INFO_COORDS_MICRO_QR);
            }
            Version::Normal(_) => {
                self.draw_number(format_info, 15, Color::Dark, Color::Light, &FORMAT_INFO_COORDS_QR_MAIN);
                self.draw_number(format_info, 15, Color::Dark, Color::Light, &FORMAT_INFO_COORDS_QR_SIDE);
                self.put(8, -8, Color::Dark); // Dark module.
            }
        }
    }

    /// Reserves area to put in the format information.
    fn draw_reserved_format_info_patterns(&mut self) {
        self.draw_format_info_patterns_with_number(0);
    }

    /// Draws the version information patterns.
    fn draw_version_info_patterns(&mut self) {
        match V::VERSION {
            Version::Micro(_) | Version::Normal(1..=6) => {
                return;
            }
            Version::Normal(a) => {
                let version_info = VERSION_INFOS[(a - 7).as_usize()];
                self.draw_number(version_info, 18, Color::Dark, Color::Light, &VERSION_INFO_COORDS_BL);
                self.draw_number(version_info, 18, Color::Dark, Color::Light, &VERSION_INFO_COORDS_TR);
            }
        }
    }
}

#[cfg(test)]
mod draw_version_info_tests {
    use crate::canvas::Canvas;
    use crate::spec::{Version1, Version7, EcLevelL};

    // #[test]
    // fn test_draw_number() {
    //     let mut c = Canvas::new(Version::Micro(1), EcLevel::L);
    //     c.draw_number(0b10101101, 8, Color::Dark, Color::Light, &[(0, 0), (0, -1), (-2, -2), (-2, 0)]);
    //     assert_eq!(
    //         &*c.to_debug_str(),
    //         "\n\
    //          #--------.-\n\
    //          -----------\n\
    //          -----------\n\
    //          -----------\n\
    //          -----------\n\
    //          -----------\n\
    //          -----------\n\
    //          -----------\n\
    //          -----------\n\
    //          ---------#-\n\
    //          .----------"
    //     );
    // }

    #[test]
    fn test_draw_version_info_1() {
        let mut c = Canvas::<Version1<EcLevelL>>::new();
        c.draw_version_info_patterns();
        assert_eq!(
            &*c.to_debug_str(),
            "\n\
             ---------------------\n\
             ---------------------\n\
             ---------------------\n\
             ---------------------\n\
             ---------------------\n\
             ---------------------\n\
             ---------------------\n\
             ---------------------\n\
             ---------------------\n\
             ---------------------\n\
             ---------------------\n\
             ---------------------\n\
             ---------------------\n\
             ---------------------\n\
             ---------------------\n\
             ---------------------\n\
             ---------------------\n\
             ---------------------\n\
             ---------------------\n\
             ---------------------\n\
             ---------------------"
        );
    }

    #[test]
    fn test_draw_version_info_7() {
        let mut c = Canvas::<Version7<EcLevelL>>::new();
        c.draw_version_info_patterns();

        assert_eq!(
            &*c.to_debug_str(),
            "\n\
             ----------------------------------..#--------\n\
             ----------------------------------.#.--------\n\
             ----------------------------------.#.--------\n\
             ----------------------------------.##--------\n\
             ----------------------------------###--------\n\
             ----------------------------------...--------\n\
             ---------------------------------------------\n\
             ---------------------------------------------\n\
             ---------------------------------------------\n\
             ---------------------------------------------\n\
             ---------------------------------------------\n\
             ---------------------------------------------\n\
             ---------------------------------------------\n\
             ---------------------------------------------\n\
             ---------------------------------------------\n\
             ---------------------------------------------\n\
             ---------------------------------------------\n\
             ---------------------------------------------\n\
             ---------------------------------------------\n\
             ---------------------------------------------\n\
             ---------------------------------------------\n\
             ---------------------------------------------\n\
             ---------------------------------------------\n\
             ---------------------------------------------\n\
             ---------------------------------------------\n\
             ---------------------------------------------\n\
             ---------------------------------------------\n\
             ---------------------------------------------\n\
             ---------------------------------------------\n\
             ---------------------------------------------\n\
             ---------------------------------------------\n\
             ---------------------------------------------\n\
             ---------------------------------------------\n\
             ---------------------------------------------\n\
             ....#.---------------------------------------\n\
             .####.---------------------------------------\n\
             #..##.---------------------------------------\n\
             ---------------------------------------------\n\
             ---------------------------------------------\n\
             ---------------------------------------------\n\
             ---------------------------------------------\n\
             ---------------------------------------------\n\
             ---------------------------------------------\n\
             ---------------------------------------------\n\
             ---------------------------------------------"
        );
    }

    #[test]
    fn test_draw_reserved_format_info_patterns_qr() {
        let mut c = Canvas::<Version1<EcLevelL>>::new();
        c.draw_reserved_format_info_patterns();
        assert_eq!(
            &*c.to_debug_str(),
            "\n\
             --------.------------\n\
             --------.------------\n\
             --------.------------\n\
             --------.------------\n\
             --------.------------\n\
             --------.------------\n\
             ---------------------\n\
             --------.------------\n\
             ......-..----........\n\
             ---------------------\n\
             ---------------------\n\
             ---------------------\n\
             ---------------------\n\
             --------#------------\n\
             --------.------------\n\
             --------.------------\n\
             --------.------------\n\
             --------.------------\n\
             --------.------------\n\
             --------.------------\n\
             --------.------------"
        );
    }

    // #[test]
    // fn test_draw_reserved_format_info_patterns_micro_qr() {
    //     let mut c = Canvas::new(Version::Micro(1), EcLevel::L);
    //     c.draw_reserved_format_info_patterns();
    //     assert_eq!(
    //         &*c.to_debug_str(),
    //         "\n\
    //          -----------\n\
    //          --------.--\n\
    //          --------.--\n\
    //          --------.--\n\
    //          --------.--\n\
    //          --------.--\n\
    //          --------.--\n\
    //          --------.--\n\
    //          -........--\n\
    //          -----------\n\
    //          -----------"
    //     );
    // }
}


static VERSION_INFO_COORDS_BL: [(i16, i16); 18] = [
    (5, -9),
    (5, -10),
    (5, -11),
    (4, -9),
    (4, -10),
    (4, -11),
    (3, -9),
    (3, -10),
    (3, -11),
    (2, -9),
    (2, -10),
    (2, -11),
    (1, -9),
    (1, -10),
    (1, -11),
    (0, -9),
    (0, -10),
    (0, -11),
];

static VERSION_INFO_COORDS_TR: [(i16, i16); 18] = [
    (-9, 5),
    (-10, 5),
    (-11, 5),
    (-9, 4),
    (-10, 4),
    (-11, 4),
    (-9, 3),
    (-10, 3),
    (-11, 3),
    (-9, 2),
    (-10, 2),
    (-11, 2),
    (-9, 1),
    (-10, 1),
    (-11, 1),
    (-9, 0),
    (-10, 0),
    (-11, 0),
];

static FORMAT_INFO_COORDS_QR_MAIN: [(i16, i16); 15] = [
    (0, 8),
    (1, 8),
    (2, 8),
    (3, 8),
    (4, 8),
    (5, 8),
    (7, 8),
    (8, 8),
    (8, 7),
    (8, 5),
    (8, 4),
    (8, 3),
    (8, 2),
    (8, 1),
    (8, 0),
];

static FORMAT_INFO_COORDS_QR_SIDE: [(i16, i16); 15] = [
    (8, -1),
    (8, -2),
    (8, -3),
    (8, -4),
    (8, -5),
    (8, -6),
    (8, -7),
    (-8, 8),
    (-7, 8),
    (-6, 8),
    (-5, 8),
    (-4, 8),
    (-3, 8),
    (-2, 8),
    (-1, 8),
];

static FORMAT_INFO_COORDS_MICRO_QR: [(i16, i16); 15] = [
    (1, 8),
    (2, 8),
    (3, 8),
    (4, 8),
    (5, 8),
    (6, 8),
    (7, 8),
    (8, 8),
    (8, 7),
    (8, 6),
    (8, 5),
    (8, 4),
    (8, 3),
    (8, 2),
    (8, 1),
];

static VERSION_INFOS: [u32; 34] = [
    0x07c94, 0x085bc, 0x09a99, 0x0a4d3, 0x0bbf6, 0x0c762, 0x0d847, 0x0e60d, 0x0f928, 0x10b78, 0x1145d, 0x12a17,
    0x13532, 0x149a6, 0x15683, 0x168c9, 0x177ec, 0x18ec4, 0x191e1, 0x1afab, 0x1b08e, 0x1cc1a, 0x1d33f, 0x1ed75,
    0x1f250, 0x209d5, 0x216f0, 0x228ba, 0x2379f, 0x24b0b, 0x2542e, 0x26a64, 0x27541, 0x28c69,
];

//}}}
//------------------------------------------------------------------------------
//{{{ All functional patterns before data placement

impl<V: QrSpec> Canvas<V> {
    /// Draw all functional patterns, before data placement.
    ///
    /// All functional patterns (e.g. the finder pattern) *except* the format
    /// info pattern will be filled in. The format info pattern will be filled
    /// with light modules instead. Data bits can then put in the empty modules.
    /// with `.draw_data()`.
    pub fn draw_all_functional_patterns(&mut self) {
        self.draw_finder_patterns();
        self.draw_alignment_patterns();
        self.draw_reserved_format_info_patterns();
        self.draw_timing_patterns();
        self.draw_version_info_patterns();
    }
}

/// Gets whether the module at the given coordinates represents a functional
/// module.
pub fn is_functional(version: Version, width: i16, x: i16, y: i16) -> bool {
    debug_assert!(width == version.width());

    let x = if x < 0 { x + width } else { x };
    let y = if y < 0 { y + width } else { y };

    match version {
        Version::Micro(_) => x == 0 || y == 0 || (x < 9 && y < 9),
        Version::Normal(a) => {
            let non_alignment_test = x == 6 || y == 6 || // Timing patterns
                    (x < 9 && y < 9) ||                  // Top-left finder pattern
                    (x < 9 && y >= width-8) ||           // Bottom-left finder pattern
                    (x >= width-8 && y < 9); // Top-right finder pattern
            if non_alignment_test {
                true
            } else if a == 1 {
                false
            } else if 2 <= a && a <= 6 {
                (width - 7 - x).abs() <= 2 && (width - 7 - y).abs() <= 2
            } else {
                let positions = ALIGNMENT_PATTERN_POSITIONS[(a - 7).as_usize()];
                let last = positions.len() - 1;
                for (i, align_x) in positions.iter().enumerate() {
                    for (j, align_y) in positions.iter().enumerate() {
                        if i == 0 && (j == 0 || j == last) || (i == last && j == 0) {
                            continue;
                        }
                        if (*align_x - x).abs() <= 2 && (*align_y - y).abs() <= 2 {
                            return true;
                        }
                    }
                }
                false
            }
        }
    }
}

#[cfg(test)]
mod all_functional_patterns_tests {
    use crate::canvas::{is_functional, Canvas};
    use crate::types::{Version};
    use crate::spec::{Version2, EcLevelL};

    #[test]
    fn test_all_functional_patterns_qr() {
        let mut c = Canvas::<Version2<EcLevelL>>::new();
        c.draw_all_functional_patterns();
        assert_eq!(
            &*c.to_debug_str(),
            "\n\
             #######..--------.#######\n\
             #.....#..--------.#.....#\n\
             #.###.#..--------.#.###.#\n\
             #.###.#..--------.#.###.#\n\
             #.###.#..--------.#.###.#\n\
             #.....#..--------.#.....#\n\
             #######.#.#.#.#.#.#######\n\
             .........--------........\n\
             ......#..--------........\n\
             ------.------------------\n\
             ------#------------------\n\
             ------.------------------\n\
             ------#------------------\n\
             ------.------------------\n\
             ------#------------------\n\
             ------.------------------\n\
             ------#---------#####----\n\
             ........#-------#...#----\n\
             #######..-------#.#.#----\n\
             #.....#..-------#...#----\n\
             #.###.#..-------#####----\n\
             #.###.#..----------------\n\
             #.###.#..----------------\n\
             #.....#..----------------\n\
             #######..----------------"
        );
    }

    // #[test]
    // fn test_all_functional_patterns_micro_qr() {
    //     let mut c = Canvas::new(Version::Micro(1), EcLevel::L);
    //     c.draw_all_functional_patterns();
    //     assert_eq!(
    //         &*c.to_debug_str(),
    //         "\n\
    //          #######.#.#\n\
    //          #.....#..--\n\
    //          #.###.#..--\n\
    //          #.###.#..--\n\
    //          #.###.#..--\n\
    //          #.....#..--\n\
    //          #######..--\n\
    //          .........--\n\
    //          #........--\n\
    //          .----------\n\
    //          #----------"
    //     );
    // }

    #[test]
    fn test_is_functional_qr_1() {
        let version = Version::Normal(1);
        assert!(is_functional(version, version.width(), 0, 0));
        assert!(is_functional(version, version.width(), 10, 6));
        assert!(!is_functional(version, version.width(), 10, 5));
        assert!(!is_functional(version, version.width(), 14, 14));
        assert!(is_functional(version, version.width(), 6, 11));
        assert!(!is_functional(version, version.width(), 4, 11));
        assert!(is_functional(version, version.width(), 4, 13));
        assert!(is_functional(version, version.width(), 17, 7));
        assert!(!is_functional(version, version.width(), 17, 17));
    }

    #[test]
    fn test_is_functional_qr_3() {
        let version = Version::Normal(3);
        assert!(is_functional(version, version.width(), 0, 0));
        assert!(!is_functional(version, version.width(), 25, 24));
        assert!(is_functional(version, version.width(), 24, 24));
        assert!(!is_functional(version, version.width(), 9, 25));
        assert!(!is_functional(version, version.width(), 20, 0));
        assert!(is_functional(version, version.width(), 21, 0));
    }

    #[test]
    fn test_is_functional_qr_7() {
        let version = Version::Normal(7);
        assert!(is_functional(version, version.width(), 21, 4));
        assert!(is_functional(version, version.width(), 7, 21));
        assert!(is_functional(version, version.width(), 22, 22));
        assert!(is_functional(version, version.width(), 8, 8));
        assert!(!is_functional(version, version.width(), 19, 5));
        assert!(!is_functional(version, version.width(), 36, 3));
        assert!(!is_functional(version, version.width(), 4, 36));
        assert!(is_functional(version, version.width(), 38, 38));
    }

    #[test]
    fn test_is_functional_micro() {
        let version = Version::Micro(1);
        assert!(is_functional(version, version.width(), 8, 0));
        assert!(is_functional(version, version.width(), 10, 0));
        assert!(!is_functional(version, version.width(), 10, 1));
        assert!(is_functional(version, version.width(), 8, 8));
        assert!(is_functional(version, version.width(), 0, 9));
        assert!(!is_functional(version, version.width(), 1, 9));
    }
}


//}}}
//------------------------------------------------------------------------------
//{{{ Data placement iterator

struct DataModuleIter {
    x: i16,
    y: i16,
    width: i16,
    timing_pattern_column: i16,
}

impl DataModuleIter {
    fn new(version: Version) -> Self {
        let width = version.width();
        Self {
            x: width - 1,
            y: width - 1,
            width,
            timing_pattern_column: match version {
                Version::Micro(_) => 0,
                Version::Normal(_) => 6,
            },
        }
    }
}

impl Iterator for DataModuleIter {
    type Item = (i16, i16);

    fn next(&mut self) -> Option<(i16, i16)> {
        let adjusted_ref_col = if self.x <= self.timing_pattern_column { self.x + 1 } else { self.x };
        if adjusted_ref_col <= 0 {
            return None;
        }

        let res = (self.x, self.y);
        let column_type = (self.width - adjusted_ref_col) % 4;

        match column_type {
            2 if self.y > 0 => {
                self.y -= 1;
                self.x += 1;
            }
            0 if self.y < self.width - 1 => {
                self.y += 1;
                self.x += 1;
            }
            0 | 2 if self.x == self.timing_pattern_column + 1 => {
                self.x -= 2;
            }
            _ => {
                self.x -= 1;
            }
        }

        Some(res)
    }
}

#[cfg(test)]
#[cfg_attr(rustfmt, rustfmt_skip)] // skip to prevent file becoming too long.
mod data_iter_tests {
    use crate::canvas::DataModuleIter;
    use crate::types::Version;

    #[test]
    fn test_qr() {
        let res = DataModuleIter::new(Version::Normal(1)).collect::<Vec<(i16, i16)>>();
        assert_eq!(res, vec![
            (20, 20), (19, 20), (20, 19), (19, 19), (20, 18), (19, 18),
            (20, 17), (19, 17), (20, 16), (19, 16), (20, 15), (19, 15),
            (20, 14), (19, 14), (20, 13), (19, 13), (20, 12), (19, 12),
            (20, 11), (19, 11), (20, 10), (19, 10), (20, 9), (19, 9),
            (20, 8), (19, 8), (20, 7), (19, 7), (20, 6), (19, 6),
            (20, 5), (19, 5), (20, 4), (19, 4), (20, 3), (19, 3),
            (20, 2), (19, 2), (20, 1), (19, 1), (20, 0), (19, 0),

            (18, 0), (17, 0), (18, 1), (17, 1), (18, 2), (17, 2),
            (18, 3), (17, 3), (18, 4), (17, 4), (18, 5), (17, 5),
            (18, 6), (17, 6), (18, 7), (17, 7), (18, 8), (17, 8),
            (18, 9), (17, 9), (18, 10), (17, 10), (18, 11), (17, 11),
            (18, 12), (17, 12), (18, 13), (17, 13), (18, 14), (17, 14),
            (18, 15), (17, 15), (18, 16), (17, 16), (18, 17), (17, 17),
            (18, 18), (17, 18), (18, 19), (17, 19), (18, 20), (17, 20),

            (16, 20), (15, 20), (16, 19), (15, 19), (16, 18), (15, 18),
            (16, 17), (15, 17), (16, 16), (15, 16), (16, 15), (15, 15),
            (16, 14), (15, 14), (16, 13), (15, 13), (16, 12), (15, 12),
            (16, 11), (15, 11), (16, 10), (15, 10), (16, 9), (15, 9),
            (16, 8), (15, 8), (16, 7), (15, 7), (16, 6), (15, 6),
            (16, 5), (15, 5), (16, 4), (15, 4), (16, 3), (15, 3),
            (16, 2), (15, 2), (16, 1), (15, 1), (16, 0), (15, 0),

            (14, 0), (13, 0), (14, 1), (13, 1), (14, 2), (13, 2),
            (14, 3), (13, 3), (14, 4), (13, 4), (14, 5), (13, 5),
            (14, 6), (13, 6), (14, 7), (13, 7), (14, 8), (13, 8),
            (14, 9), (13, 9), (14, 10), (13, 10), (14, 11), (13, 11),
            (14, 12), (13, 12), (14, 13), (13, 13), (14, 14), (13, 14),
            (14, 15), (13, 15), (14, 16), (13, 16), (14, 17), (13, 17),
            (14, 18), (13, 18), (14, 19), (13, 19), (14, 20), (13, 20),

            (12, 20), (11, 20), (12, 19), (11, 19), (12, 18), (11, 18),
            (12, 17), (11, 17), (12, 16), (11, 16), (12, 15), (11, 15),
            (12, 14), (11, 14), (12, 13), (11, 13), (12, 12), (11, 12),
            (12, 11), (11, 11), (12, 10), (11, 10), (12, 9), (11, 9),
            (12, 8), (11, 8), (12, 7), (11, 7), (12, 6), (11, 6),
            (12, 5), (11, 5), (12, 4), (11, 4), (12, 3), (11, 3),
            (12, 2), (11, 2), (12, 1), (11, 1), (12, 0), (11, 0),

            (10, 0), (9, 0), (10, 1), (9, 1), (10, 2), (9, 2),
            (10, 3), (9, 3), (10, 4), (9, 4), (10, 5), (9, 5),
            (10, 6), (9, 6), (10, 7), (9, 7), (10, 8), (9, 8),
            (10, 9), (9, 9), (10, 10), (9, 10), (10, 11), (9, 11),
            (10, 12), (9, 12), (10, 13), (9, 13), (10, 14), (9, 14),
            (10, 15), (9, 15), (10, 16), (9, 16), (10, 17), (9, 17),
            (10, 18), (9, 18), (10, 19), (9, 19), (10, 20), (9, 20),

            (8, 20), (7, 20), (8, 19), (7, 19), (8, 18), (7, 18),
            (8, 17), (7, 17), (8, 16), (7, 16), (8, 15), (7, 15),
            (8, 14), (7, 14), (8, 13), (7, 13), (8, 12), (7, 12),
            (8, 11), (7, 11), (8, 10), (7, 10), (8, 9), (7, 9),
            (8, 8), (7, 8), (8, 7), (7, 7), (8, 6), (7, 6),
            (8, 5), (7, 5), (8, 4), (7, 4), (8, 3), (7, 3),
            (8, 2), (7, 2), (8, 1), (7, 1), (8, 0), (7, 0),

            (5, 0), (4, 0), (5, 1), (4, 1), (5, 2), (4, 2),
            (5, 3), (4, 3), (5, 4), (4, 4), (5, 5), (4, 5),
            (5, 6), (4, 6), (5, 7), (4, 7), (5, 8), (4, 8),
            (5, 9), (4, 9), (5, 10), (4, 10), (5, 11), (4, 11),
            (5, 12), (4, 12), (5, 13), (4, 13), (5, 14), (4, 14),
            (5, 15), (4, 15), (5, 16), (4, 16), (5, 17), (4, 17),
            (5, 18), (4, 18), (5, 19), (4, 19), (5, 20), (4, 20),

            (3, 20), (2, 20), (3, 19), (2, 19), (3, 18), (2, 18),
            (3, 17), (2, 17), (3, 16), (2, 16), (3, 15), (2, 15),
            (3, 14), (2, 14), (3, 13), (2, 13), (3, 12), (2, 12),
            (3, 11), (2, 11), (3, 10), (2, 10), (3, 9), (2, 9),
            (3, 8), (2, 8), (3, 7), (2, 7), (3, 6), (2, 6),
            (3, 5), (2, 5), (3, 4), (2, 4), (3, 3), (2, 3),
            (3, 2), (2, 2), (3, 1), (2, 1), (3, 0), (2, 0),

            (1, 0), (0, 0), (1, 1), (0, 1), (1, 2), (0, 2),
            (1, 3), (0, 3), (1, 4), (0, 4), (1, 5), (0, 5),
            (1, 6), (0, 6), (1, 7), (0, 7), (1, 8), (0, 8),
            (1, 9), (0, 9), (1, 10), (0, 10), (1, 11), (0, 11),
            (1, 12), (0, 12), (1, 13), (0, 13), (1, 14), (0, 14),
            (1, 15), (0, 15), (1, 16), (0, 16), (1, 17), (0, 17),
            (1, 18), (0, 18), (1, 19), (0, 19), (1, 20), (0, 20),
        ]);
    }

    #[test]
    fn test_micro_qr() {
        let res = DataModuleIter::new(Version::Micro(1)).collect::<Vec<(i16, i16)>>();
        assert_eq!(res, vec![
            (10, 10), (9, 10), (10, 9), (9, 9), (10, 8), (9, 8),
            (10, 7), (9, 7), (10, 6), (9, 6), (10, 5), (9, 5),
            (10, 4), (9, 4), (10, 3), (9, 3), (10, 2), (9, 2),
            (10, 1), (9, 1), (10, 0), (9, 0),

            (8, 0), (7, 0), (8, 1), (7, 1), (8, 2), (7, 2),
            (8, 3), (7, 3), (8, 4), (7, 4), (8, 5), (7, 5),
            (8, 6), (7, 6), (8, 7), (7, 7), (8, 8), (7, 8),
            (8, 9), (7, 9), (8, 10), (7, 10),

            (6, 10), (5, 10), (6, 9), (5, 9), (6, 8), (5, 8),
            (6, 7), (5, 7), (6, 6), (5, 6), (6, 5), (5, 5),
            (6, 4), (5, 4), (6, 3), (5, 3), (6, 2), (5, 2),
            (6, 1), (5, 1), (6, 0), (5, 0),

            (4, 0), (3, 0), (4, 1), (3, 1), (4, 2), (3, 2),
            (4, 3), (3, 3), (4, 4), (3, 4), (4, 5), (3, 5),
            (4, 6), (3, 6), (4, 7), (3, 7), (4, 8), (3, 8),
            (4, 9), (3, 9), (4, 10), (3, 10),

            (2, 10), (1, 10), (2, 9), (1, 9), (2, 8), (1, 8),
            (2, 7), (1, 7), (2, 6), (1, 6), (2, 5), (1, 5),
            (2, 4), (1, 4), (2, 3), (1, 3), (2, 2), (1, 2),
            (2, 1), (1, 1), (2, 0), (1, 0),
        ]);
    }

    #[test]
    fn test_micro_qr_2() {
        let res = DataModuleIter::new(Version::Micro(2)).collect::<Vec<(i16, i16)>>();
        assert_eq!(res, vec![
            (12, 12), (11, 12), (12, 11), (11, 11), (12, 10), (11, 10),
            (12, 9), (11, 9), (12, 8), (11, 8), (12, 7), (11, 7),
            (12, 6), (11, 6), (12, 5), (11, 5), (12, 4), (11, 4),
            (12, 3), (11, 3), (12, 2), (11, 2), (12, 1), (11, 1),
            (12, 0), (11, 0),

            (10, 0), (9, 0), (10, 1), (9, 1), (10, 2), (9, 2),
            (10, 3), (9, 3), (10, 4), (9, 4), (10, 5), (9, 5),
            (10, 6), (9, 6), (10, 7), (9, 7), (10, 8), (9, 8),
            (10, 9), (9, 9), (10, 10), (9, 10), (10, 11), (9, 11),
            (10, 12), (9, 12),

            (8, 12), (7, 12), (8, 11), (7, 11), (8, 10), (7, 10),
            (8, 9), (7, 9), (8, 8), (7, 8), (8, 7), (7, 7),
            (8, 6), (7, 6), (8, 5), (7, 5), (8, 4), (7, 4),
            (8, 3), (7, 3), (8, 2), (7, 2), (8, 1), (7, 1),
            (8, 0), (7, 0),

            (6, 0), (5, 0), (6, 1), (5, 1), (6, 2), (5, 2),
            (6, 3), (5, 3), (6, 4), (5, 4), (6, 5), (5, 5),
            (6, 6), (5, 6), (6, 7), (5, 7), (6, 8), (5, 8),
            (6, 9), (5, 9), (6, 10), (5, 10), (6, 11), (5, 11),
            (6, 12), (5, 12),

            (4, 12), (3, 12), (4, 11), (3, 11), (4, 10), (3, 10),
            (4, 9), (3, 9), (4, 8), (3, 8), (4, 7), (3, 7),
            (4, 6), (3, 6), (4, 5), (3, 5), (4, 4), (3, 4),
            (4, 3), (3, 3), (4, 2), (3, 2), (4, 1), (3, 1),
            (4, 0), (3, 0),

            (2, 0), (1, 0), (2, 1), (1, 1), (2, 2), (1, 2),
            (2, 3), (1, 3), (2, 4), (1, 4), (2, 5), (1, 5),
            (2, 6), (1, 6), (2, 7), (1, 7), (2, 8), (1, 8),
            (2, 9), (1, 9), (2, 10), (1, 10), (2, 11), (1, 11),
            (2, 12), (1, 12),
        ]);
    }
}


//}}}
//------------------------------------------------------------------------------
//{{{ Data placement

impl<V: QrSpec> Canvas<V> {
    fn draw_codewords<I>(&mut self, codewords: &[u8], is_half_codeword_at_end: bool, coords: &mut I)
    where
        I: Iterator<Item = (i16, i16)>,
    {
        let length = codewords.len();
        let last_word = if is_half_codeword_at_end { length - 1 } else { length };
        for (i, b) in codewords.iter().enumerate() {
            let bits_end = if i == last_word { 4 } else { 0 };
            'outside: for j in (bits_end..=7).rev() {
                let color = if (*b & (1 << j)) == 0 { Color::Light } else { Color::Dark };
                while let Some((x, y)) = coords.next() {
                    let r = self.get(x, y);
                    if let Module::Unmasked(_) = r {
                        self.put_unmasked(x, y, color);
                        continue 'outside;
                    }
                }
                return;
            }
        }
    }

    /// Draws the encoded data and error correction codes to the empty modules.
    pub fn draw_data(&mut self, data: &[u8], ec: &[u8]) {
        let is_half_codeword_at_end = match (V::VERSION, V::EC_LEVEL) {
            (Version::Micro(1), EcLevel::L) | (Version::Micro(3), EcLevel::M) => true,
            _ => false,
        };

        let mut coords = DataModuleIter::new(V::VERSION);
        self.draw_codewords(data, is_half_codeword_at_end, &mut coords);
        self.draw_codewords(ec, false, &mut coords);
    }
}


#[cfg(test)]
mod draw_codewords_test {
    use crate::canvas::Canvas;
    use crate::spec::{Version2, EcLevelL};

    // #[test]
    // fn test_micro_qr_1() {
    //    let mut c = Canvas::new(Version::Micro(1), EcLevel::L);
    //    c.draw_all_functional_patterns();
    //    c.draw_data(b"\x6e\x5d\xe2", b"\x2b\x63");
    //    assert_eq!(
    //        &*c.to_debug_str(),
    //        "\n\
    //         #######.#.#\n\
    //         #.....#..-*\n\
    //         #.###.#..**\n\
    //         #.###.#..*-\n\
    //         #.###.#..**\n\
    //         #.....#..*-\n\
    //         #######..*-\n\
    //         .........-*\n\
    //         #........**\n\
    //         .***-**---*\n\
    //         #---*-*-**-"
    //    );
    // }

    #[test]
    fn test_qr_2() {
        let mut c = Canvas::<Version2<EcLevelL>>::new();
        c.draw_all_functional_patterns();
        
        c.draw_data(
            b"\x92I$\x92I$\x92I$\x92I$\x92I$\x92I$\x92I$\x92I$\
              \x92I$\x92I$\x92I$\x92I$\x92I$\x92I$\x92I$",
            b"",
        );

        assert_eq!(
            &*c.to_debug_str(),
            "\n\
             #######..--*---*-.#######\n\
             #.....#..-*-*-*-*.#.....#\n\
             #.###.#..*---*---.#.###.#\n\
             #.###.#..--*---*-.#.###.#\n\
             #.###.#..-*-*-*-*.#.###.#\n\
             #.....#..*---*---.#.....#\n\
             #######.#.#.#.#.#.#######\n\
             .........--*---*-........\n\
             ......#..-*-*-*-*........\n\
             --*-*-.-**---*---*--**--*\n\
             -*-*--#----*---*---------\n\
             *----*.*--*-*-*-*-**--**-\n\
             --*-*-#-**---*---*--**--*\n\
             -*-*--.----*---*---------\n\
             *----*#*--*-*-*-*-**--**-\n\
             --*-*-.-**---*---*--**--*\n\
             -*-*--#----*---*#####----\n\
             ........#-*-*-*-#...#-**-\n\
             #######..*---*--#.#.#*--*\n\
             #.....#..--*---*#...#----\n\
             #.###.#..-*-*-*-#####-**-\n\
             #.###.#..*---*--*----*--*\n\
             #.###.#..--*------**-----\n\
             #.....#..-*-*-**-*--*-**-\n\
             #######..*---*--*----*--*"
        );
    }
}

//}}}
//------------------------------------------------------------------------------
//{{{ Masking

/// The mask patterns. Since QR code and Micro QR code do not use the same
/// pattern number, we name them according to their shape instead of the number.
#[derive(Debug, Copy, Clone)]
pub enum MaskPattern {
    /// QR code pattern 000: `(x + y) % 2 == 0`.
    Checkerboard = 0b000,

    /// QR code pattern 001: `y % 2 == 0`.
    HorizontalLines = 0b001,

    /// QR code pattern 010: `x % 3 == 0`.
    VerticalLines = 0b010,

    /// QR code pattern 011: `(x + y) % 3 == 0`.
    DiagonalLines = 0b011,

    /// QR code pattern 100: `((x/3) + (y/2)) % 2 == 0`.
    LargeCheckerboard = 0b100,

    /// QR code pattern 101: `(x*y)%2 + (x*y)%3 == 0`.
    Fields = 0b101,

    /// QR code pattern 110: `((x*y)%2 + (x*y)%3) % 2 == 0`.
    Diamonds = 0b110,

    /// QR code pattern 111: `((x+y)%2 + (x*y)%3) % 2 == 0`.
    Meadow = 0b111,
}

mod mask_functions {
    pub fn checkerboard(x: i16, y: i16) -> bool {
        (x + y) % 2 == 0
    }
    pub fn horizontal_lines(_: i16, y: i16) -> bool {
        y % 2 == 0
    }
    pub fn vertical_lines(x: i16, _: i16) -> bool {
        x % 3 == 0
    }
    pub fn diagonal_lines(x: i16, y: i16) -> bool {
        (x + y) % 3 == 0
    }
    pub fn large_checkerboard(x: i16, y: i16) -> bool {
        ((y / 2) + (x / 3)) % 2 == 0
    }
    pub fn fields(x: i16, y: i16) -> bool {
        (x * y) % 2 + (x * y) % 3 == 0
    }
    pub fn diamonds(x: i16, y: i16) -> bool {
        ((x * y) % 2 + (x * y) % 3) % 2 == 0
    }
    pub fn meadow(x: i16, y: i16) -> bool {
        ((x + y) % 2 + (x * y) % 3) % 2 == 0
    }
}

fn get_mask_function(pattern: MaskPattern) -> fn(i16, i16) -> bool {
    match pattern {
        MaskPattern::Checkerboard => mask_functions::checkerboard,
        MaskPattern::HorizontalLines => mask_functions::horizontal_lines,
        MaskPattern::VerticalLines => mask_functions::vertical_lines,
        MaskPattern::DiagonalLines => mask_functions::diagonal_lines,
        MaskPattern::LargeCheckerboard => mask_functions::large_checkerboard,
        MaskPattern::Fields => mask_functions::fields,
        MaskPattern::Diamonds => mask_functions::diamonds,
        MaskPattern::Meadow => mask_functions::meadow,
    }
}

impl<V: QrSpec> Canvas<V> {
    /// Applies a mask to the canvas. This method will also draw the format info
    /// patterns.
    pub fn apply_mask(&mut self, pattern: MaskPattern) {
        let mask_fn = get_mask_function(pattern);
        for x in 0..V::WIDTH {
            for y in 0..V::WIDTH {
                let module = self.get(x, y);
                self.put(x, y, module.mask(mask_fn(x, y)));
            }
        }

        self.draw_format_info_patterns(pattern);
    }

    /// Draws the format information to encode the error correction level and
    /// mask pattern.
    ///
    /// If the error correction level or mask pattern is not supported in the
    /// current QR code version, this method will fail.
    fn draw_format_info_patterns(&mut self, pattern: MaskPattern) {
        let format_number = match V::VERSION {
            Version::Normal(_) => {
                let simple_format_number = ((V::EC_LEVEL as usize) ^ 1) << 3 | (pattern as usize);
                FORMAT_INFOS_QR[simple_format_number]
            }
            Version::Micro(a) => {
                let micro_pattern_number = match pattern {
                    MaskPattern::HorizontalLines => 0b00,
                    MaskPattern::LargeCheckerboard => 0b01,
                    MaskPattern::Diamonds => 0b10,
                    MaskPattern::Meadow => 0b11,
                    _ => panic!("Unsupported mask pattern in Micro QR code"),
                };
                let symbol_number = match (a, V::EC_LEVEL) {
                    (1, EcLevel::L) => 0b000,
                    (2, EcLevel::L) => 0b001,
                    (2, EcLevel::M) => 0b010,
                    (3, EcLevel::L) => 0b011,
                    (3, EcLevel::M) => 0b100,
                    (4, EcLevel::L) => 0b101,
                    (4, EcLevel::M) => 0b110,
                    (4, EcLevel::Q) => 0b111,
                    _ => panic!("Unsupported version/ec_level combination in Micro QR code"),
                };
                let simple_format_number = symbol_number << 2 | micro_pattern_number;
                FORMAT_INFOS_MICRO_QR[simple_format_number]
            }
        };
        self.draw_format_info_patterns_with_number(format_number);
    }
}

#[cfg(test)]
mod mask_tests {
    use crate::canvas::{Canvas, MaskPattern};
    use crate::spec::{Version1, EcLevelL};

    #[test]
    fn test_apply_mask_qr() {
        let mut c = Canvas::<Version1<EcLevelL>>::new();
        c.draw_all_functional_patterns();
        c.apply_mask(MaskPattern::Checkerboard);

        assert_eq!(
            &*c.to_debug_str(),
            "\n\
             #######...#.#.#######\n\
             #.....#..#.#..#.....#\n\
             #.###.#.#.#.#.#.###.#\n\
             #.###.#..#.#..#.###.#\n\
             #.###.#...#.#.#.###.#\n\
             #.....#..#.#..#.....#\n\
             #######.#.#.#.#######\n\
             ........##.#.........\n\
             ###.#####.#.###...#..\n\
             .#.#.#.#.#.#.#.#.#.#.\n\
             #.#.#.#.#.#.#.#.#.#.#\n\
             .#.#.#.#.#.#.#.#.#.#.\n\
             #.#.#.#.#.#.#.#.#.#.#\n\
             ........##.#.#.#.#.#.\n\
             #######.#.#.#.#.#.#.#\n\
             #.....#.##.#.#.#.#.#.\n\
             #.###.#.#.#.#.#.#.#.#\n\
             #.###.#..#.#.#.#.#.#.\n\
             #.###.#.#.#.#.#.#.#.#\n\
             #.....#.##.#.#.#.#.#.\n\
             #######.#.#.#.#.#.#.#"
        );
    }

    #[test]
    fn test_draw_format_info_patterns_qr() {
        let mut c = Canvas::<Version1<EcLevelL>>::new();
        c.draw_format_info_patterns(MaskPattern::LargeCheckerboard);
        assert_eq!(
            &*c.to_debug_str(),
            "\n\
             --------#------------\n\
             --------#------------\n\
             --------#------------\n\
             --------#------------\n\
             --------.------------\n\
             --------#------------\n\
             ---------------------\n\
             --------.------------\n\
             ##..##-..----..#.####\n\
             ---------------------\n\
             ---------------------\n\
             ---------------------\n\
             ---------------------\n\
             --------#------------\n\
             --------.------------\n\
             --------#------------\n\
             --------#------------\n\
             --------.------------\n\
             --------.------------\n\
             --------#------------\n\
             --------#------------"
        );
    }

    // #[test]
    // fn test_draw_format_info_patterns_micro_qr() {
    //     let mut c = Canvas::new(Version::Micro(2), EcLevel::L);
    //     c.draw_format_info_patterns(MaskPattern::LargeCheckerboard);
    //     assert_eq!(
    //         &*c.to_debug_str(),
    //         "\n\
    //          -------------\n\
    //          --------#----\n\
    //          --------.----\n\
    //          --------.----\n\
    //          --------#----\n\
    //          --------#----\n\
    //          --------.----\n\
    //          --------.----\n\
    //          -#.#....#----\n\
    //          -------------\n\
    //          -------------\n\
    //          -------------\n\
    //          -------------"
    //     );
    // }
}


static FORMAT_INFOS_QR: [u16; 32] = [
    0x5412, 0x5125, 0x5e7c, 0x5b4b, 0x45f9, 0x40ce, 0x4f97, 0x4aa0, 0x77c4, 0x72f3, 0x7daa, 0x789d, 0x662f, 0x6318,
    0x6c41, 0x6976, 0x1689, 0x13be, 0x1ce7, 0x19d0, 0x0762, 0x0255, 0x0d0c, 0x083b, 0x355f, 0x3068, 0x3f31, 0x3a06,
    0x24b4, 0x2183, 0x2eda, 0x2bed,
];

static FORMAT_INFOS_MICRO_QR: [u16; 32] = [
    0x4445, 0x4172, 0x4e2b, 0x4b1c, 0x55ae, 0x5099, 0x5fc0, 0x5af7, 0x6793, 0x62a4, 0x6dfd, 0x68ca, 0x7678, 0x734f,
    0x7c16, 0x7921, 0x06de, 0x03e9, 0x0cb0, 0x0987, 0x1735, 0x1202, 0x1d5b, 0x186c, 0x2508, 0x203f, 0x2f66, 0x2a51,
    0x34e3, 0x31d4, 0x3e8d, 0x3bba,
];

//}}}
//------------------------------------------------------------------------------
//{{{ Penalty score

impl<V: QrSpec> Canvas<V> {
    /// Compute the penalty score for having too many adjacent modules with the
    /// same color.
    ///
    /// Every 5+N adjacent modules in the same column/row having the same color
    /// will contribute 3+N points.
    fn compute_adjacent_penalty_score(&self, is_horizontal: bool) -> u16 {
        let mut total_score = 0;

        for i in 0..V::WIDTH {
            let map_fn = |j| if is_horizontal { self.get(j, i) } else { self.get(i, j) };

            let colors = (0..V::WIDTH).map(map_fn).chain(Some(Module::EMPTY).into_iter());
            let mut last_color = Module::EMPTY;
            let mut consecutive_len = 1_u16;

            for color in colors {
                if color == last_color {
                    consecutive_len += 1;
                } else {
                    last_color = color;
                    if consecutive_len >= 5 {
                        total_score += consecutive_len - 2;
                    }
                    consecutive_len = 1;
                }
            }
        }

        total_score
    }

    /// Compute the penalty score for having too many rectangles with the same
    /// color.
    ///
    /// Every 2×2 blocks (with overlapping counted) having the same color will
    /// contribute 3 points.
    fn compute_block_penalty_score(&self) -> u16 {
        let mut total_score = 0;

        for i in 0..V::WIDTH - 1 {
            for j in 0..V::WIDTH - 1 {
                let this = self.get(i, j);
                let right = self.get(i + 1, j);
                let bottom = self.get(i, j + 1);
                let bottom_right = self.get(i + 1, j + 1);
                if this == right && right == bottom && bottom == bottom_right {
                    total_score += 3;
                }
            }
        }

        total_score
    }

    /// Compute the penalty score for having a pattern similar to the finder
    /// pattern in the wrong place.
    ///
    /// Every pattern that looks like `#.###.#....` in any orientation will add
    /// 40 points.
    fn compute_finder_penalty_score(&self, is_horizontal: bool) -> u16 {
        static PATTERN: [Color; 7] =
            [Color::Dark, Color::Light, Color::Dark, Color::Dark, Color::Dark, Color::Light, Color::Dark];

        let mut total_score = 0;

        for i in 0..V::WIDTH {
            for j in 0..V::WIDTH - 6 {
                let get_h = |k| self.get(k, i).into();
                let get_v = |k| self.get(i, k).into();
                let get: &dyn Fn(i16) -> Color = if is_horizontal { &get_h } else { &get_v };

                if (j..(j + 7)).map(&*get).ne(PATTERN.iter().cloned()) {
                    continue;
                }

                let check = |k| 0 <= k && k < V::WIDTH && get(k) != Color::Light;
                if !((j - 4)..j).any(&check) || !((j + 7)..(j + 11)).any(&check) {
                    total_score += 40;
                }
            }
        }

        total_score - 360
    }

    /// Compute the penalty score for having an unbalanced dark/light ratio.
    ///
    /// The score is given linearly by the deviation from a 50% ratio of dark
    /// modules. The highest possible score is 100.
    ///
    /// Note that this algorithm differs slightly from the standard we do not
    /// round the result every 5%, but the difference should be negligible and
    /// should not affect which mask is chosen.
    fn compute_balance_penalty_score(&self) -> u16 {
        let dark_modules = Module::from_iter(self.modules.iter().copied(), V::AREA).filter(|m| m.is_dark()).count();
        let total_modules = V::AREA;
        let ratio = dark_modules * 200 / total_modules;
        if ratio >= 100 { ratio - 100 } else { 100 - ratio }.as_u16()
    }

    /// Compute the penalty score for having too many light modules on the sides.
    ///
    /// This penalty score is exclusive to Micro QR code.
    ///
    /// Note that the standard gives the formula for *efficiency* score, which
    /// has the inverse meaning of this method, but it is very easy to convert
    /// between the two (this score is (16×width − standard-score)).
    fn compute_light_side_penalty_score(&self) -> u16 {
        let h = (1..V::WIDTH).filter(|j| !self.get(*j, -1).is_dark()).count();
        let v = (1..V::WIDTH).filter(|j| !self.get(-1, *j).is_dark()).count();

        (h + v + 15 * max(h, v)).as_u16()
    }

    /// Compute the total penalty scores. A QR code having higher points is less
    /// desirable.
    fn compute_total_penalty_scores(&self) -> u16 {
        match V::VERSION {
            Version::Normal(_) => {
                let s1_a = self.compute_adjacent_penalty_score(true);
                let s1_b = self.compute_adjacent_penalty_score(false);
                let s2 = self.compute_block_penalty_score();
                let s3_a = self.compute_finder_penalty_score(true);
                let s3_b = self.compute_finder_penalty_score(false);
                let s4 = self.compute_balance_penalty_score();
                s1_a + s1_b + s2 + s3_a + s3_b + s4
            }
            Version::Micro(_) => self.compute_light_side_penalty_score(),
        }
    }
}


#[cfg(test)]
mod penalty_tests {
    use crate::canvas::{Canvas, MaskPattern};
    use crate::spec::{Version1, EcLevelQ};

    fn create_test_canvas() -> Canvas<Version1<EcLevelQ>> {
        let mut c = Canvas::new();
        c.draw_all_functional_patterns();
        c.draw_data(
            b"\x20\x5b\x0b\x78\xd1\x72\xdc\x4d\x43\x40\xec\x11\x00",
            b"\xa8\x48\x16\x52\xd9\x36\x9c\x00\x2e\x0f\xb4\x7a\x10",
        );
        c.apply_mask(MaskPattern::Checkerboard);
        c
    }

    #[test]
    fn check_penalty_canvas() {
        let c = create_test_canvas();
        assert_eq!(
            &*c.to_debug_str(),
            "\n\
             #######.##....#######\n\
             #.....#.#..#..#.....#\n\
             #.###.#.#..##.#.###.#\n\
             #.###.#.#.....#.###.#\n\
             #.###.#.#.#...#.###.#\n\
             #.....#...#...#.....#\n\
             #######.#.#.#.#######\n\
             ........#............\n\
             .##.#.##....#.#.#####\n\
             .#......####....#...#\n\
             ..##.###.##...#.##...\n\
             .##.##.#..##.#.#.###.\n\
             #...#.#.#.###.###.#.#\n\
             ........##.#..#...#.#\n\
             #######.#.#....#.##..\n\
             #.....#..#.##.##.#...\n\
             #.###.#.#.#...#######\n\
             #.###.#..#.#.#.#...#.\n\
             #.###.#.#...####.#..#\n\
             #.....#.#.##.#...#.##\n\
             #######.....####....#"
        );
    }

    #[test]
    fn test_penalty_score_adjacent() {
        let c = create_test_canvas();
        assert_eq!(c.compute_adjacent_penalty_score(true), 88);
        assert_eq!(c.compute_adjacent_penalty_score(false), 92);
    }

    #[test]
    fn test_penalty_score_block() {
        let c = create_test_canvas();
        assert_eq!(c.compute_block_penalty_score(), 90);
    }

    #[test]
    fn test_penalty_score_finder() {
        let c = create_test_canvas();
        assert_eq!(c.compute_finder_penalty_score(true), 0);
        assert_eq!(c.compute_finder_penalty_score(false), 40);
    }

    #[test]
    fn test_penalty_score_balance() {
        let c = create_test_canvas();
        assert_eq!(c.compute_balance_penalty_score(), 2);
    }

    // #[test]
    // fn test_penalty_score_light_sides() {
    //     static HORIZONTAL_SIDE: [Color; 17] = [
    //         Color::Dark,
    //         Color::Light,
    //         Color::Light,
    //         Color::Dark,
    //         Color::Dark,
    //         Color::Dark,
    //         Color::Light,
    //         Color::Light,
    //         Color::Dark,
    //         Color::Light,
    //         Color::Dark,
    //         Color::Light,
    //         Color::Light,
    //         Color::Dark,
    //         Color::Light,
    //         Color::Light,
    //         Color::Light,
    //     ];
    //     static VERTICAL_SIDE: [Color; 17] = [
    //         Color::Dark,
    //         Color::Dark,
    //         Color::Dark,
    //         Color::Light,
    //         Color::Light,
    //         Color::Dark,
    //         Color::Dark,
    //         Color::Light,
    //         Color::Dark,
    //         Color::Light,
    //         Color::Dark,
    //         Color::Light,
    //         Color::Dark,
    //         Color::Light,
    //         Color::Light,
    //         Color::Dark,
    //         Color::Light,
    //     ];

    //     let mut c = Canvas::new(Version::Micro(4), EcLevel::Q);
    //     for i in 0_i16..17 {
    //         c.put(i, -1, HORIZONTAL_SIDE[i as usize]);
    //         c.put(-1, i, VERTICAL_SIDE[i as usize]);
    //     }

    //     assert_eq!(c.compute_light_side_penalty_score(), 168);
    // }
}

//}}}
//------------------------------------------------------------------------------
//{{{ Select mask with lowest penalty score

static ALL_PATTERNS_QR: [MaskPattern; 8] = [
    MaskPattern::Checkerboard,
    MaskPattern::HorizontalLines,
    MaskPattern::VerticalLines,
    MaskPattern::DiagonalLines,
    MaskPattern::LargeCheckerboard,
    MaskPattern::Fields,
    MaskPattern::Diamonds,
    MaskPattern::Meadow,
];

static ALL_PATTERNS_MICRO_QR: [MaskPattern; 4] =
    [MaskPattern::HorizontalLines, MaskPattern::LargeCheckerboard, MaskPattern::Diamonds, MaskPattern::Meadow];

impl<V: QrSpec> Canvas<V> {
    /// Construct a new canvas and apply the best masking that gives the lowest
    /// penalty score.
    pub fn apply_best_mask(&self) -> Canvas<V> {
        match V::VERSION {
            Version::Normal(_) => ALL_PATTERNS_QR.iter(),
            Version::Micro(_) => ALL_PATTERNS_MICRO_QR.iter(),
        }
        .map(|ptn| {
            let mut c: Canvas<V> = Canvas::clone(self);
            c.apply_mask(*ptn);
            c
        })
        .min_by_key(Self::compute_total_penalty_scores)
        .expect("at least one pattern")
    }

    /// Convert the modules into a vector of colors.
    pub fn colors(&self) -> impl Iterator<Item = Color> + '_ {
        Module::from_iter(self.modules.iter().copied(), V::AREA).map(Color::from)
    }

    /// Convert the modules into a vector of colors.
    pub fn color_bits(&self) -> Vec<u8, V::ColorSize> {
        let mut result = Vec::new();
        let mut buf = 0_u8;
        for (i, color) in self.colors().enumerate() {
            buf <<= 1;
            if let Color::Dark = color {
                buf |= 0b1
            }
            if i % 8 == 7 {
                result.push(buf).unwrap();
                buf = 0;
            }
        }
        result.push(buf).unwrap();
        result
    }

    /// Convert the modules into a vector of colors.
    pub fn color_line_bits(&self) -> Vec<u8, V::ColorSize> {
        let mut result = Vec::new();
        let mut buf = 0_u8;
        let mut i = 0;
        for color in self.colors() {
            buf <<= 1;
            if let Color::Dark = color {
                buf |= 0b1
            }
            
            i += 1;
            if i % 8 == 0 {
                result.push(buf).unwrap();
                buf = 0;
            }
            if i == V::WIDTH {
                result.push(buf).unwrap();
                buf = 0;
                i = 0;
            }
        }
        result
    }
}

//}}}
//------------------------------------------------------------------------------
