/*! This module contains all definitions of QRCode versions and error correction
levels. MicroQR is currently not included.

# Generation code
This code is used to generate the macro code below.
```ignore
use std::fmt::{Display, Formatter, Result as FmtResult};

/// `DATA_BYTES_PER_BLOCK` provides the number of codewords (bytes) used for
/// real data per block in each version.
///
/// This is a copy of ISO/IEC 18004:2006, §6.5.1, Table 9 (The value "k" of the
/// 7th column, followed by the 6th column).
///
/// Every entry is a 4-tuple. Take `DATA_BYTES_PER_BLOCK[39][3] == (15, 20, 16, 61)`
/// as an example, this means in version 40 with correction level H, there are
/// 20 blocks with 15 bytes in size, and 61 blocks with 16 bytes in size.
///
/// (byte size block 1, block count 1, byte size block 2, block count 2,
/// ec bytes per block)
static DATA_BYTES_PER_BLOCK: [[(usize, usize, usize, usize, usize); 4]; 44] = [
    // Normal versions.
    [(19, 1, 0, 0, 7), (16, 1, 0, 0, 10), (13, 1, 0, 0, 13), (9, 1, 0, 0, 17)],   // 1
    [(34, 1, 0, 0, 10), (28, 1, 0, 0, 16), (22, 1, 0, 0, 22), (16, 1, 0, 0, 28)], // 2
    [(55, 1, 0, 0, 15), (44, 1, 0, 0, 26), (17, 2, 0, 0, 18), (13, 2, 0, 0, 22)], // 3
    [(80, 1, 0, 0, 20), (32, 2, 0, 0, 18), (24, 2, 0, 0, 26), (9, 4, 0, 0, 16)],  // 4
    [(108, 1, 0, 0, 26), (43, 2, 0, 0, 24), (15, 2, 16, 2, 18), (11, 2, 12, 2, 22)], // 5
    [(68, 2, 0, 0, 18), (27, 4, 0, 0, 16), (19, 4, 0, 0, 24), (15, 4, 0, 0, 28)], // 6
    [(78, 2, 0, 0, 20), (31, 4, 0, 0, 18), (14, 2, 15, 4, 18), (13, 4, 14, 1, 26)], // 7
    [(97, 2, 0, 0, 24), (38, 2, 39, 2, 22), (18, 4, 19, 2, 22), (14, 4, 15, 2, 26)], // 8
    [(116, 2, 0, 0, 30), (36, 3, 37, 2, 22), (16, 4, 17, 4, 20), (12, 4, 13, 4, 24)], // 9
    [(68, 2, 69, 2, 18), (43, 4, 44, 1, 26), (19, 6, 20, 2, 24), (15, 6, 16, 2, 28)], // 10
    [(81, 4, 0, 0, 20), (50, 1, 51, 4, 30), (22, 4, 23, 4, 28), (12, 3, 13, 8, 24)], // 11
    [(92, 2, 93, 2, 24), (36, 6, 37, 2, 22), (20, 4, 21, 6, 26), (14, 7, 15, 4, 28)], // 12
    [(107, 4, 0, 0, 26), (37, 8, 38, 1, 22), (20, 8, 21, 4, 24), (11, 12, 12, 4, 22)], // 13
    [(115, 3, 116, 1, 30), (40, 4, 41, 5, 24), (16, 11, 17, 5, 20), (12, 11, 13, 5, 24)], // 14
    [(87, 5, 88, 1, 22), (41, 5, 42, 5, 24), (24, 5, 25, 7, 30), (12, 11, 13, 7, 24)], // 15
    [(98, 5, 99, 1, 24), (45, 7, 46, 3, 28), (19, 15, 20, 2, 24), (15, 3, 16, 13, 30)], // 16
    [(107, 1, 108, 5, 28), (46, 10, 47, 1, 28), (22, 1, 23, 15, 28), (14, 2, 15, 17, 28)], // 17
    [(120, 5, 121, 1, 30), (43, 9, 44, 4, 26), (22, 17, 23, 1, 28), (14, 2, 15, 19, 28)], // 18
    [(113, 3, 114, 4, 28), (44, 3, 45, 11, 26), (21, 17, 22, 4, 26), (13, 9, 14, 16, 26)], // 19
    [(107, 3, 108, 5, 28), (41, 3, 42, 13, 26), (24, 15, 25, 5, 30), (15, 15, 16, 10, 28)], // 20
    [(116, 4, 117, 4, 28), (42, 17, 0, 0, 26), (22, 17, 23, 6, 28), (16, 19, 17, 6, 30)], // 21
    [(111, 2, 112, 7, 28), (46, 17, 0, 0, 28), (24, 7, 25, 16, 30), (13, 34, 0, 0, 24)], // 22
    [(121, 4, 122, 5, 30), (47, 4, 48, 14, 28), (24, 11, 25, 14, 30), (15, 16, 16, 14, 30)], // 23
    [(117, 6, 118, 4, 30), (45, 6, 46, 14, 28), (24, 11, 25, 16, 30), (16, 30, 17, 2, 30)], // 24
    [(106, 8, 107, 4, 26), (47, 8, 48, 13, 28), (24, 7, 25, 22, 30), (15, 22, 16, 13, 30)], // 25
    [(114, 10, 115, 2, 28), (46, 19, 47, 4, 28), (22, 28, 23, 6, 28), (16, 33, 17, 4, 30)], // 26
    [(122, 8, 123, 4, 30), (45, 22, 46, 3, 28), (23, 8, 24, 26, 30), (15, 12, 16, 28, 30)], // 27
    [(117, 3, 118, 10, 30), (45, 3, 46, 23, 28), (24, 4, 25, 31, 30), (15, 11, 16, 31, 30)], // 28
    [(116, 7, 117, 7, 30), (45, 21, 46, 7, 28), (23, 1, 24, 37, 30), (15, 19, 16, 26, 30)], // 29
    [(115, 5, 116, 10, 30), (47, 19, 48, 10, 28), (24, 15, 25, 25, 30), (15, 23, 16, 25, 30)], // 30
    [(115, 13, 116, 3, 30), (46, 2, 47, 29, 28), (24, 42, 25, 1, 30), (15, 23, 16, 28, 30)], // 31
    [(115, 17, 0, 0, 30), (46, 10, 47, 23, 28), (24, 10, 25, 35, 30), (15, 19, 16, 35, 30)], // 32
    [(115, 17, 116, 1, 30), (46, 14, 47, 21, 28), (24, 29, 25, 19, 30), (15, 11, 16, 46, 30)], // 33
    [(115, 13, 116, 6, 30), (46, 14, 47, 23, 28), (24, 44, 25, 7, 30), (16, 59, 17, 1, 30)], // 34
    [(121, 12, 122, 7, 30), (47, 12, 48, 26, 28), (24, 39, 25, 14, 30), (15, 22, 16, 41, 30)], // 35
    [(121, 6, 122, 14, 30), (47, 6, 48, 34, 28), (24, 46, 25, 10, 30), (15, 2, 16, 64, 30)], // 36
    [(122, 17, 123, 4, 30), (46, 29, 47, 14, 28), (24, 49, 25, 10, 30), (15, 24, 16, 46, 30)], // 37
    [(122, 4, 123, 18, 30), (46, 13, 47, 32, 28), (24, 48, 25, 14, 30), (15, 42, 16, 32, 30)], // 38
    [(117, 20, 118, 4, 30), (47, 40, 48, 7, 28), (24, 43, 25, 22, 30), (15, 10, 16, 67, 30)], // 39
    [(118, 19, 119, 6, 30), (47, 18, 48, 31, 28), (24, 34, 25, 34, 30), (15, 20, 16, 61, 30)], // 40
    // Micro versions.
    [(3, 1, 0, 0, 2), (0, 0, 0, 0, 0), (0, 0, 0, 0, 0), (0, 0, 0, 0, 0)],      // M1
    [(5, 1, 0, 0, 5), (4, 1, 0, 0, 6), (0, 0, 0, 0, 0), (0, 0, 0, 0, 0)],      // M2
    [(11, 1, 0, 0, 6), (9, 1, 0, 0, 8), (0, 0, 0, 0, 0), (0, 0, 0, 0, 0)],     // M3
    [(16, 1, 0, 0, 8), (14, 1, 0, 0, 10), (10, 1, 0, 0, 14), (0, 0, 0, 0, 0)], // M4
];

fn main() {
    println!("// --------------------------------------------------------");
    println!("// -- Generated -- Generated --  Generated --  Generated --");
    println!("// --------------------------------------------------------\n");
    println!("spec_normal! {{");
    for i in 0..40 {
        let (s1, c1, s2, c2, ec) = DATA_BYTES_PER_BLOCK[i][0];
        let total_size = ec * (c1 + c2) + s1 * c1 + s2 * c2;
        let width = i * 4 + 21;
        println!(
            "  Version{}, {}, {}, {}, {} => [",
            i + 1,
            TypeNum(total_size),
            TypeNum(width * width / 8 + 1),
            TypeNum(width * width / 4 + 1),
            i + 1
        );
        for l in 0..4 {
            let (s1, c1, s2, c2, ec) = DATA_BYTES_PER_BLOCK[i][l];

            // Check that we always end up having the same total size.
            assert_eq!(total_size, ec * (c1 + c2) + s1 * c1 + s2 * c2);

            let ec_gen_buffer_size = s1.max(s2) + ec;
            let ec_blocks_size = ec * (c1 + c2);
            let bits_size = s1 * c1 + s2 * c2;

            println!(
                "    {{ {}, {}, {}, {}, {}, {}, {}, {} }},",
                TypeNum(ec_gen_buffer_size),
                TypeNum(ec_blocks_size),
                TypeNum(bits_size),
                s1,
                c1,
                s2,
                c2,
                ec
            );
        }
        println!("  ]{}", if i < 39 { "," } else { "" });
    }
    println!("}}");
}

pub struct TypeNum(usize);

impl Display for TypeNum {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let TypeNum(x) = *self;
        if x <= 1024 {
            write!(f, "U{}", x)
        } else {
            let mut buf = String::new();
            let mut i = 0;
            let mut y = x;
            while y != 0 {
                y = y >> 1;
                i += 1;
                buf.push_str("UInt<");
            }
            buf.push_str("UTerm");
            y = x;
            while i > 0 {
                i -= 1;
                if y & (1 << i) == 0 {
                    buf.push_str(", B0>");
                } else {
                    buf.push_str(", B1>");
                }
            }
            f.write_str(&buf)
        }
    }
}
```
*/

use crate::types::{EcLevel, Version};
use core::marker::PhantomData;
use heapless::consts::*;
use heapless::ArrayLength;
use typenum::{UInt, UTerm, B0, B1};

pub trait QrSpec {
    /// EC_BYTES_PER_BLOCK * (BLOCK_1_COUNT + BLOCK_2_COUNT) + BLOCK_1_COUNT *
    /// BLOCK_1_SIZE + BLOCK_2_COUNT * BLOCK_2_SIZE
    type TotalSize: ArrayLength<u8>;
    /// MAX(BLOCK_1_SIZE, BLOCK_2_SIZE) + EC_BYTES_PER_BLOCK
    type ECGenBufferSize: ArrayLength<u8>;
    /// EC_BYTES_PER_BLOCK * (BLOCK_1_COUNT + BLOCK_2_COUNT)
    type ECBlocksSize: ArrayLength<u8>;
    /// WIDTH * WIDTH / 4 + 1
    type CanvasSize: ArrayLength<u8>;
    /// WIDTH * WIDTH / 8 + 1
    type ColorSize: ArrayLength<u8>;
    /// BLOCK_1_COUNT * BLOCK_1_SIZE + BLOCK_2_COUNT * BLOCK_2_SIZE
    type BitsSize: ArrayLength<u8>;

    const WIDTH: i16;
    const BLOCK_1_SIZE: usize;
    const BLOCK_1_COUNT: usize;
    const BLOCK_2_SIZE: usize;
    const BLOCK_2_COUNT: usize;
    const EC_BYTES_PER_BLOCK: usize;
    const VERSION: Version;
    const EC_LEVEL: EcLevel;
    const AREA: usize = (Self::WIDTH * Self::WIDTH) as usize;
}

pub trait EcLvl {
    const EC_LEVEL: EcLevel;
}

pub struct EcLevelL;
impl EcLvl for EcLevelL {
    const EC_LEVEL: EcLevel = EcLevel::L;
}
pub struct EcLevelM;
impl EcLvl for EcLevelM {
    const EC_LEVEL: EcLevel = EcLevel::M;
}
pub struct EcLevelQ;
impl EcLvl for EcLevelQ {
    const EC_LEVEL: EcLevel = EcLevel::Q;
}
pub struct EcLevelH;
impl EcLvl for EcLevelH {
    const EC_LEVEL: EcLevel = EcLevel::H;
}

macro_rules! spec_normal_level {
    ($name:ident, $level:ty, $ec_level:expr, $total_size:ty, $ec_gen_buffer_size:ty, $ec_blocks_size:ty, $color_size:ty, $canvas_size:ty, $bits_size:ty, $version_num:expr, $block_1_size:expr, $block_1_count:expr, $block_2_size:expr, $block_2_count:expr, $ec_bytes_per_block:expr) => {
        impl QrSpec for $name<$level> {
            type TotalSize = $total_size;
            type ECGenBufferSize = $ec_gen_buffer_size;
            type ECBlocksSize = $ec_blocks_size;
            type CanvasSize = $canvas_size;
            type ColorSize = $color_size;
            type BitsSize = $bits_size;

            const WIDTH: i16 = $version_num * 4 + 17;
            const BLOCK_1_SIZE: usize = $block_1_size;
            const BLOCK_1_COUNT: usize = $block_1_count;
            const BLOCK_2_SIZE: usize = $block_2_size;
            const BLOCK_2_COUNT: usize = $block_2_count;
            const EC_BYTES_PER_BLOCK: usize = $ec_bytes_per_block;
            const VERSION: Version = Version::Normal($version_num);
            const EC_LEVEL: EcLevel = $ec_level;
        }
    };
}

macro_rules! spec_normal {
   {$($name:ident, $total_size:ty, $color_size:ty, $canvas_size:ty, $version_num:expr => [
       { $ec_gen_buffer_size_l:ty, $ec_blocks_size_l:ty, $bits_size_l:ty, $block_1_size_l:expr, $block_1_count_l:expr, $block_2_size_l:expr, $block_2_count_l:expr, $ec_bytes_per_block_l:expr },
       { $ec_gen_buffer_size_m:ty, $ec_blocks_size_m:ty, $bits_size_m:ty, $block_1_size_m:expr, $block_1_count_m:expr, $block_2_size_m:expr, $block_2_count_m:expr, $ec_bytes_per_block_m:expr },
       { $ec_gen_buffer_size_q:ty, $ec_blocks_size_q:ty, $bits_size_q:ty, $block_1_size_q:expr, $block_1_count_q:expr, $block_2_size_q:expr, $block_2_count_q:expr, $ec_bytes_per_block_q:expr },
       { $ec_gen_buffer_size_h:ty, $ec_blocks_size_h:ty, $bits_size_h:ty, $block_1_size_h:expr, $block_1_count_h:expr, $block_2_size_h:expr, $block_2_count_h:expr, $ec_bytes_per_block_h:expr },
   ]),*} => {$(
       pub struct $name<L: EcLvl>(PhantomData<L>);
       spec_normal_level!($name, EcLevelL, EcLevel::L, $total_size, $ec_gen_buffer_size_l, $ec_blocks_size_l, $color_size, $canvas_size, $bits_size_l, $version_num, $block_1_size_l, $block_1_count_l, $block_2_size_l, $block_2_count_l, $ec_bytes_per_block_l);
       spec_normal_level!($name, EcLevelM, EcLevel::M, $total_size, $ec_gen_buffer_size_m, $ec_blocks_size_m, $color_size, $canvas_size, $bits_size_m, $version_num, $block_1_size_m, $block_1_count_m, $block_2_size_m, $block_2_count_m, $ec_bytes_per_block_m);
       spec_normal_level!($name, EcLevelQ, EcLevel::Q, $total_size, $ec_gen_buffer_size_q, $ec_blocks_size_q, $color_size, $canvas_size, $bits_size_q, $version_num, $block_1_size_q, $block_1_count_q, $block_2_size_q, $block_2_count_q, $ec_bytes_per_block_q);
       spec_normal_level!($name, EcLevelH, EcLevel::H, $total_size, $ec_gen_buffer_size_h, $ec_blocks_size_h, $color_size, $canvas_size, $bits_size_h, $version_num, $block_1_size_h, $block_1_count_h, $block_2_size_h, $block_2_count_h, $ec_bytes_per_block_h);
   )*};
}

// --------------------------------------------------------
// -- Generated -- Generated --  Generated --  Generated --
// --------------------------------------------------------

spec_normal! {
  Version1, U26, U56, U111, 1 => [
    { U26, U7, U19, 19, 1, 0, 0, 7 },
    { U26, U10, U16, 16, 1, 0, 0, 10 },
    { U26, U13, U13, 13, 1, 0, 0, 13 },
    { U26, U17, U9, 9, 1, 0, 0, 17 },
  ],
  Version2, U44, U79, U157, 2 => [
    { U44, U10, U34, 34, 1, 0, 0, 10 },
    { U44, U16, U28, 28, 1, 0, 0, 16 },
    { U44, U22, U22, 22, 1, 0, 0, 22 },
    { U44, U28, U16, 16, 1, 0, 0, 28 },
  ],
  Version3, U70, U106, U211, 3 => [
    { U70, U15, U55, 55, 1, 0, 0, 15 },
    { U70, U26, U44, 44, 1, 0, 0, 26 },
    { U35, U36, U34, 17, 2, 0, 0, 18 },
    { U35, U44, U26, 13, 2, 0, 0, 22 },
  ],
  Version4, U100, U137, U273, 4 => [
    { U100, U20, U80, 80, 1, 0, 0, 20 },
    { U50, U36, U64, 32, 2, 0, 0, 18 },
    { U50, U52, U48, 24, 2, 0, 0, 26 },
    { U25, U64, U36, 9, 4, 0, 0, 16 },
  ],
  Version5, U134, U172, U343, 5 => [
    { U134, U26, U108, 108, 1, 0, 0, 26 },
    { U67, U48, U86, 43, 2, 0, 0, 24 },
    { U34, U72, U62, 15, 2, 16, 2, 18 },
    { U34, U88, U46, 11, 2, 12, 2, 22 },
  ],
  Version6, U172, U211, U421, 6 => [
    { U86, U36, U136, 68, 2, 0, 0, 18 },
    { U43, U64, U108, 27, 4, 0, 0, 16 },
    { U43, U96, U76, 19, 4, 0, 0, 24 },
    { U43, U112, U60, 15, 4, 0, 0, 28 },
  ],
  Version7, U196, U254, U507, 7 => [
    { U98, U40, U156, 78, 2, 0, 0, 20 },
    { U49, U72, U124, 31, 4, 0, 0, 18 },
    { U33, U108, U88, 14, 2, 15, 4, 18 },
    { U40, U130, U66, 13, 4, 14, 1, 26 },
  ],
  Version8, U242, U301, U601, 8 => [
    { U121, U48, U194, 97, 2, 0, 0, 24 },
    { U61, U88, U154, 38, 2, 39, 2, 22 },
    { U41, U132, U110, 18, 4, 19, 2, 22 },
    { U41, U156, U86, 14, 4, 15, 2, 26 },
  ],
  Version9, U292, U352, U703, 9 => [
    { U146, U60, U232, 116, 2, 0, 0, 30 },
    { U59, U110, U182, 36, 3, 37, 2, 22 },
    { U37, U160, U132, 16, 4, 17, 4, 20 },
    { U37, U192, U100, 12, 4, 13, 4, 24 },
  ],
  Version10, U346, U407, U813, 10 => [
    { U87, U72, U274, 68, 2, 69, 2, 18 },
    { U70, U130, U216, 43, 4, 44, 1, 26 },
    { U44, U192, U154, 19, 6, 20, 2, 24 },
    { U44, U224, U122, 15, 6, 16, 2, 28 },
  ],
  Version11, U404, U466, U931, 11 => [
    { U101, U80, U324, 81, 4, 0, 0, 20 },
    { U81, U150, U254, 50, 1, 51, 4, 30 },
    { U51, U224, U180, 22, 4, 23, 4, 28 },
    { U37, U264, U140, 12, 3, 13, 8, 24 },
  ],
  Version12, U466, U529, UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, B1>, B0>, B0>, B0>, B0>, B1>, B0>, B0>, B0>, B0>, B1>, 12 => [
    { U117, U96, U370, 92, 2, 93, 2, 24 },
    { U59, U176, U290, 36, 6, 37, 2, 22 },
    { U47, U260, U206, 20, 4, 21, 6, 26 },
    { U43, U308, U158, 14, 7, 15, 4, 28 },
  ],
  Version13, U532, U596, UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, B1>, B0>, B0>, B1>, B0>, B1>, B0>, B0>, B1>, B1>, B1>, 13 => [
    { U133, U104, U428, 107, 4, 0, 0, 26 },
    { U60, U198, U334, 37, 8, 38, 1, 22 },
    { U45, U288, U244, 20, 8, 21, 4, 24 },
    { U34, U352, U180, 11, 12, 12, 4, 22 },
  ],
  Version14, U581, U667, UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, B1>, B0>, B1>, B0>, B0>, B1>, B1>, B0>, B1>, B0>, B1>, 14 => [
    { U146, U120, U461, 115, 3, 116, 1, 30 },
    { U65, U216, U365, 40, 4, 41, 5, 24 },
    { U37, U320, U261, 16, 11, 17, 5, 20 },
    { U37, U384, U197, 12, 11, 13, 5, 24 },
  ],
  Version15, U655, U742, UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, B1>, B0>, B1>, B1>, B1>, B0>, B0>, B1>, B0>, B1>, B1>, 15 => [
    { U110, U132, U523, 87, 5, 88, 1, 22 },
    { U66, U240, U415, 41, 5, 42, 5, 24 },
    { U55, U360, U295, 24, 5, 25, 7, 30 },
    { U37, U432, U223, 12, 11, 13, 7, 24 },
  ],
  Version16, U733, U821, UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, B1>, B1>, B0>, B0>, B1>, B1>, B0>, B1>, B0>, B0>, B1>, 16 => [
    { U123, U144, U589, 98, 5, 99, 1, 24 },
    { U74, U280, U453, 45, 7, 46, 3, 28 },
    { U44, U408, U325, 19, 15, 20, 2, 24 },
    { U46, U480, U253, 15, 3, 16, 13, 30 },
  ],
  Version17, U815, U904, UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, B1>, B1>, B1>, B0>, B0>, B0>, B0>, B1>, B1>, B1>, B1>, 17 => [
    { U136, U168, U647, 107, 1, 108, 5, 28 },
    { U75, U308, U507, 46, 10, 47, 1, 28 },
    { U51, U448, U367, 22, 1, 23, 15, 28 },
    { U43, U532, U283, 14, 2, 15, 17, 28 },
  ],
  Version18, U901, U991, UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, B1>, B1>, B1>, B1>, B0>, B1>, B1>, B1>, B1>, B0>, B1>, 18 => [
    { U151, U180, U721, 120, 5, 121, 1, 30 },
    { U70, U338, U563, 43, 9, 44, 4, 26 },
    { U51, U504, U397, 22, 17, 23, 1, 28 },
    { U43, U588, U313, 14, 2, 15, 19, 28 },
  ],
  Version19, U991, UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, B1>, B0>, B0>, B0>, B0>, B1>, B1>, B1>, B0>, B1>, B0>, UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, B1>, B0>, B0>, B0>, B0>, B1>, B1>, B1>, B0>, B0>, B1>, B1>, 19 => [
    { U142, U196, U795, 113, 3, 114, 4, 28 },
    { U71, U364, U627, 44, 3, 45, 11, 26 },
    { U48, U546, U445, 21, 17, 22, 4, 26 },
    { U40, U650, U341, 13, 9, 14, 16, 26 },
  ],
  Version20, UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, B1>, B0>, B0>, B0>, B0>, B1>, B1>, B1>, B1>, B0>, B1>, UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, B1>, B0>, B0>, B1>, B0>, B0>, B1>, B1>, B0>, B0>, B1>, UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, B1>, B0>, B0>, B1>, B0>, B0>, B1>, B1>, B0>, B0>, B0>, B1>, 20 => [
    { U136, U224, U861, 107, 3, 108, 5, 28 },
    { U68, U416, U669, 41, 3, 42, 13, 26 },
    { U55, U600, U485, 24, 15, 25, 5, 30 },
    { U44, U700, U385, 15, 15, 16, 10, 28 },
  ],
  Version21, UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, B1>, B0>, B0>, B1>, B0>, B0>, B0>, B0>, B1>, B0>, B0>, UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, B1>, B0>, B0>, B1>, B1>, B1>, B1>, B1>, B1>, B0>, B0>, UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, B1>, B0>, B0>, B1>, B1>, B1>, B1>, B1>, B0>, B1>, B1>, B1>, 21 => [
    { U145, U224, U932, 116, 4, 117, 4, 28 },
    { U68, U442, U714, 42, 17, 0, 0, 26 },
    { U51, U644, U512, 22, 17, 23, 6, 28 },
    { U47, U750, U406, 16, 19, 17, 6, 30 },
  ],
  Version22, UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, B1>, B0>, B0>, B1>, B1>, B1>, B0>, B1>, B0>, B1>, B0>, UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, B1>, B0>, B1>, B0>, B1>, B1>, B0>, B0>, B0>, B1>, B1>, UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, B1>, B0>, B1>, B0>, B1>, B1>, B0>, B0>, B0>, B1>, B0>, B1>, 22 => [
    { U140, U252, U1006, 111, 2, 112, 7, 28 },
    { U74, U476, U782, 46, 17, 0, 0, 28 },
    { U55, U690, U568, 24, 7, 25, 16, 30 },
    { U37, U816, U442, 13, 34, 0, 0, 24 },
  ],
  Version23, UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, B1>, B0>, B1>, B0>, B1>, B0>, B1>, B0>, B1>, B0>, B0>, UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, B1>, B0>, B1>, B1>, B1>, B0>, B0>, B1>, B1>, B1>, B0>, UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, B1>, B0>, B1>, B1>, B1>, B0>, B0>, B1>, B1>, B0>, B1>, B1>, 23 => [
    { U152, U270, UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, B1>, B0>, B0>, B0>, B1>, B0>, B0>, B0>, B1>, B1>, B0>, 121, 4, 122, 5, 30 },
    { U76, U504, U860, 47, 4, 48, 14, 28 },
    { U55, U750, U614, 24, 11, 25, 14, 30 },
    { U46, U900, U464, 15, 16, 16, 14, 30 },
  ],
  Version24, UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, B1>, B0>, B1>, B1>, B1>, B0>, B0>, B0>, B0>, B1>, B0>, UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, B1>, B1>, B0>, B0>, B0>, B1>, B1>, B1>, B1>, B0>, B1>, UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, B1>, B1>, B0>, B0>, B0>, B1>, B1>, B1>, B1>, B0>, B0>, B1>, 24 => [
    { U148, U300, UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, B1>, B0>, B0>, B1>, B0>, B0>, B1>, B0>, B1>, B1>, B0>, 117, 6, 118, 4, 30 },
    { U74, U560, U914, 45, 6, 46, 14, 28 },
    { U55, U810, U664, 24, 11, 25, 16, 30 },
    { U47, U960, U514, 16, 30, 17, 2, 30 },
  ],
  Version25, UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, B1>, B1>, B0>, B0>, B0>, B1>, B1>, B0>, B1>, B0>, B0>, UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, B1>, B1>, B0>, B1>, B0>, B1>, B1>, B0>, B0>, B0>, B0>, UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, B1>, B1>, B0>, B1>, B0>, B1>, B0>, B1>, B1>, B1>, B1>, B1>, 25 => [
    { U133, U312, UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, B1>, B0>, B0>, B1>, B1>, B1>, B1>, B1>, B1>, B0>, B0>, 106, 8, 107, 4, 26 },
    { U76, U588, U1000, 47, 8, 48, 13, 28 },
    { U55, U870, U718, 24, 7, 25, 22, 30 },
    { U46, UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, B1>, B0>, B0>, B0>, B0>, B0>, B1>, B1>, B0>, B1>, B0>, U538, 15, 22, 16, 13, 30 },
  ],
  Version26, UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, B1>, B1>, B0>, B1>, B0>, B1>, B0>, B1>, B0>, B1>, B0>, UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, B1>, B1>, B1>, B0>, B0>, B1>, B0>, B0>, B1>, B1>, B1>, UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, B1>, B1>, B1>, B0>, B0>, B1>, B0>, B0>, B1>, B1>, B0>, B1>, 26 => [
    { U143, U336, UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, B1>, B0>, B1>, B0>, B1>, B0>, B1>, B1>, B0>, B1>, B0>, 114, 10, 115, 2, 28 },
    { U75, U644, UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, B1>, B0>, B0>, B0>, B0>, B1>, B0>, B0>, B1>, B1>, B0>, 46, 19, 47, 4, 28 },
    { U51, U952, U754, 22, 28, 23, 6, 28 },
    { U47, UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, B1>, B0>, B0>, B0>, B1>, B0>, B1>, B0>, B1>, B1>, B0>, U596, 16, 33, 17, 4, 30 },
  ],
  Version27, UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, B1>, B1>, B1>, B0>, B0>, B1>, B0>, B0>, B1>, B0>, B0>, UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, B1>, B1>, B1>, B1>, B0>, B1>, B0>, B0>, B0>, B1>, B0>, UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, B1>, B1>, B1>, B1>, B0>, B1>, B0>, B0>, B0>, B0>, B1>, B1>, 27 => [
    { U153, U360, UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, B1>, B0>, B1>, B1>, B0>, B1>, B1>, B1>, B1>, B0>, B0>, 122, 8, 123, 4, 30 },
    { U74, U700, UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, B1>, B0>, B0>, B0>, B1>, B1>, B0>, B1>, B0>, B0>, B0>, 45, 22, 46, 3, 28 },
    { U54, U1020, U808, 23, 8, 24, 26, 30 },
    { U46, UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, B1>, B0>, B0>, B1>, B0>, B1>, B1>, B0>, B0>, B0>, B0>, U628, 15, 12, 16, 28, 30 },
  ],
  Version28, UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, B1>, B1>, B1>, B1>, B0>, B0>, B0>, B0>, B0>, B0>, B1>, UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, B1>, B0>, B0>, B0>, B0>, B0>, B1>, B0>, B0>, B0>, B0>, B1>, UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, B1>, B0>, B0>, B0>, B0>, B0>, B1>, B0>, B0>, B0>, B0>, B0>, B1>, 28 => [
    { U148, U390, UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, B1>, B0>, B1>, B1>, B1>, B1>, B1>, B1>, B0>, B1>, B1>, 117, 3, 118, 10, 30 },
    { U74, U728, UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, B1>, B0>, B0>, B1>, B0>, B1>, B0>, B1>, B0>, B0>, B1>, 45, 3, 46, 23, 28 },
    { U55, UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, B1>, B0>, B0>, B0>, B0>, B0>, B1>, B1>, B0>, B1>, B0>, U871, 24, 4, 25, 31, 30 },
    { U46, UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, B1>, B0>, B0>, B1>, B1>, B1>, B0>, B1>, B1>, B0>, B0>, U661, 15, 11, 16, 31, 30 },
  ],
  Version29, UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, B1>, B0>, B0>, B0>, B0>, B0>, B0>, B0>, B0>, B0>, B1>, B1>, UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, B1>, B0>, B0>, B0>, B1>, B0>, B1>, B0>, B0>, B1>, B0>, B0>, UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, B1>, B0>, B0>, B0>, B1>, B0>, B1>, B0>, B0>, B0>, B1>, B1>, B1>, 29 => [
    { U147, U420, UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, B1>, B1>, B0>, B0>, B1>, B0>, B1>, B1>, B1>, B1>, B1>, 116, 7, 117, 7, 30 },
    { U74, U784, UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, B1>, B0>, B0>, B1>, B1>, B1>, B1>, B0>, B0>, B1>, B1>, 45, 21, 46, 7, 28 },
    { U54, UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, B1>, B0>, B0>, B0>, B1>, B1>, B1>, B0>, B1>, B0>, B0>, U911, 23, 1, 24, 37, 30 },
    { U46, UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, B1>, B0>, B1>, B0>, B1>, B0>, B0>, B0>, B1>, B1>, B0>, U701, 15, 19, 16, 26, 30 },
  ],
  Version30, UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, B1>, B0>, B0>, B0>, B1>, B0>, B0>, B0>, B1>, B0>, B0>, B1>, UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, B1>, B0>, B0>, B1>, B0>, B0>, B1>, B0>, B1>, B0>, B1>, B1>, UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, B1>, B0>, B0>, B1>, B0>, B0>, B1>, B0>, B1>, B0>, B1>, B0>, B1>, 30 => [
    { U146, U450, UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, B1>, B1>, B0>, B1>, B1>, B0>, B0>, B0>, B1>, B1>, B1>, 115, 5, 116, 10, 30 },
    { U76, U812, UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, B1>, B0>, B1>, B0>, B1>, B0>, B1>, B1>, B1>, B0>, B1>, 47, 19, 48, 10, 28 },
    { U55, UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, B1>, B0>, B0>, B1>, B0>, B1>, B1>, B0>, B0>, B0>, B0>, U985, 24, 15, 25, 25, 30 },
    { U46, UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, B1>, B0>, B1>, B1>, B0>, B1>, B0>, B0>, B0>, B0>, B0>, U745, 15, 23, 16, 25, 30 },
  ],
  Version31, UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, B1>, B0>, B0>, B1>, B0>, B0>, B0>, B1>, B0>, B0>, B1>, B1>, UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, B1>, B0>, B0>, B1>, B1>, B0>, B1>, B1>, B0>, B1>, B1>, B0>, UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, B1>, B0>, B0>, B1>, B1>, B0>, B1>, B1>, B0>, B1>, B0>, B1>, B1>, 31 => [
    { U146, U480, UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, B1>, B1>, B1>, B0>, B0>, B1>, B1>, B0>, B0>, B1>, B1>, 115, 13, 116, 3, 30 },
    { U75, U868, UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, B1>, B0>, B1>, B1>, B0>, B1>, B0>, B1>, B1>, B1>, B1>, 46, 2, 47, 29, 28 },
    { U55, UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, B1>, B0>, B1>, B0>, B0>, B0>, B0>, B1>, B0>, B1>, B0>, UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, B1>, B0>, B0>, B0>, B0>, B0>, B0>, B1>, B0>, B0>, B1>, 24, 42, 25, 1, 30 },
    { U46, UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, B1>, B0>, B1>, B1>, B1>, B1>, B1>, B1>, B0>, B1>, B0>, U793, 15, 23, 16, 28, 30 },
  ],
  Version32, UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, B1>, B0>, B0>, B1>, B1>, B0>, B1>, B0>, B0>, B0>, B0>, B1>, UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, B1>, B0>, B1>, B0>, B0>, B1>, B0>, B0>, B0>, B1>, B0>, B1>, UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, B1>, B0>, B1>, B0>, B0>, B1>, B0>, B0>, B0>, B1>, B0>, B0>, B1>, 32 => [
    { U145, U510, UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, B1>, B1>, B1>, B1>, B0>, B1>, B0>, B0>, B0>, B1>, B1>, 115, 17, 0, 0, 30 },
    { U75, U924, UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, B1>, B1>, B0>, B0>, B0>, B0>, B0>, B0>, B1>, B0>, B1>, 46, 10, 47, 23, 28 },
    { U55, UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, B1>, B0>, B1>, B0>, B1>, B0>, B0>, B0>, B1>, B1>, B0>, UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, B1>, B0>, B0>, B0>, B1>, B0>, B1>, B1>, B0>, B1>, B1>, 24, 10, 25, 35, 30 },
    { U46, UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, B1>, B1>, B0>, B0>, B1>, B0>, B1>, B0>, B1>, B0>, B0>, U845, 15, 19, 16, 35, 30 },
  ],
  Version33, UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, B1>, B0>, B1>, B0>, B0>, B0>, B1>, B1>, B0>, B0>, B1>, B1>, UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, B1>, B0>, B1>, B0>, B1>, B1>, B0>, B1>, B1>, B0>, B0>, B0>, UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, B1>, B0>, B1>, B0>, B1>, B1>, B0>, B1>, B0>, B1>, B1>, B1>, B1>, 33 => [
    { U146, U540, UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, B1>, B0>, B0>, B0>, B0>, B0>, B0>, B1>, B0>, B1>, B1>, B1>, 115, 17, 116, 1, 30 },
    { U75, U980, UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, B1>, B1>, B0>, B0>, B1>, B0>, B1>, B1>, B1>, B1>, B1>, 46, 14, 47, 21, 28 },
    { U55, UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, B1>, B0>, B1>, B1>, B0>, B1>, B0>, B0>, B0>, B0>, B0>, UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, B1>, B0>, B0>, B1>, B0>, B0>, B1>, B0>, B0>, B1>, B1>, 24, 29, 25, 19, 30 },
    { U46, UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, B1>, B1>, B0>, B1>, B0>, B1>, B0>, B1>, B1>, B1>, B0>, U901, 15, 11, 16, 46, 30 },
  ],
  Version34, UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, B1>, B0>, B1>, B0>, B1>, B1>, B0>, B0>, B1>, B0>, B0>, B1>, UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, B1>, B0>, B1>, B1>, B0>, B1>, B1>, B0>, B1>, B1>, B1>, B1>, UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, B1>, B0>, B1>, B1>, B0>, B1>, B1>, B0>, B1>, B1>, B1>, B0>, B1>, 34 => [
    { U146, U570, UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, B1>, B0>, B0>, B0>, B1>, B0>, B0>, B0>, B1>, B1>, B1>, B1>, 115, 13, 116, 6, 30 },
    { U75, UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, B1>, B0>, B0>, B0>, B0>, B0>, B0>, B1>, B1>, B0>, B0>, UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, B1>, B1>, B0>, B1>, B0>, B1>, B1>, B1>, B1>, B0>, B1>, 46, 14, 47, 23, 28 },
    { U55, UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, B1>, B0>, B1>, B1>, B1>, B1>, B1>, B1>, B0>, B1>, B0>, UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, B1>, B0>, B0>, B1>, B1>, B0>, B0>, B1>, B1>, B1>, B1>, 24, 44, 25, 7, 30 },
    { U47, UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, B1>, B1>, B1>, B0>, B0>, B0>, B0>, B1>, B0>, B0>, B0>, U961, 16, 59, 17, 1, 30 },
  ],
  Version35, UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, B1>, B0>, B1>, B1>, B0>, B0>, B1>, B1>, B1>, B1>, B0>, B0>, UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, B1>, B1>, B0>, B0>, B0>, B0>, B0>, B0>, B1>, B0>, B1>, B0>, UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, B1>, B1>, B0>, B0>, B0>, B0>, B0>, B0>, B1>, B0>, B0>, B1>, B1>, 35 => [
    { U152, U570, UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, B1>, B0>, B0>, B1>, B0>, B0>, B0>, B0>, B0>, B0>, B1>, B0>, 121, 12, 122, 7, 30 },
    { U76, UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, B1>, B0>, B0>, B0>, B0>, B1>, B0>, B1>, B0>, B0>, B0>, UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, B1>, B1>, B1>, B0>, B0>, B0>, B1>, B0>, B1>, B0>, B0>, 47, 12, 48, 26, 28 },
    { U55, UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, B1>, B1>, B0>, B0>, B0>, B1>, B1>, B0>, B1>, B1>, B0>, UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, B1>, B0>, B1>, B0>, B0>, B0>, B0>, B0>, B1>, B1>, B0>, 24, 39, 25, 14, 30 },
    { U46, UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, B1>, B1>, B1>, B0>, B1>, B1>, B0>, B0>, B0>, B1>, B0>, U986, 15, 22, 16, 41, 30 },
  ],
  Version36, UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, B1>, B0>, B1>, B1>, B1>, B1>, B0>, B1>, B1>, B0>, B1>, B0>, UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, B1>, B1>, B0>, B0>, B1>, B0>, B1>, B0>, B1>, B0>, B0>, B1>, UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, B1>, B1>, B0>, B0>, B1>, B0>, B1>, B0>, B1>, B0>, B0>, B0>, B1>, 36 => [
    { U152, U600, UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, B1>, B0>, B0>, B1>, B1>, B0>, B0>, B0>, B0>, B0>, B1>, B0>, 121, 6, 122, 14, 30 },
    { U76, UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, B1>, B0>, B0>, B0>, B1>, B1>, B0>, B0>, B0>, B0>, B0>, UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, B1>, B1>, B1>, B0>, B1>, B1>, B1>, B1>, B0>, B1>, B0>, 47, 6, 48, 34, 28 },
    { U55, UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, B1>, B1>, B0>, B1>, B0>, B0>, B1>, B0>, B0>, B0>, B0>, UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, B1>, B0>, B1>, B0>, B1>, B0>, B0>, B1>, B0>, B1>, B0>, 24, 46, 25, 10, 30 },
    { U46, UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, B1>, B1>, B1>, B1>, B0>, B1>, B1>, B1>, B1>, B0>, B0>, UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, B1>, B0>, B0>, B0>, B0>, B0>, B1>, B1>, B1>, B1>, B0>, 15, 2, 16, 64, 30 },
  ],
  Version37, UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, B1>, B1>, B0>, B0>, B0>, B1>, B1>, B1>, B1>, B1>, B0>, B0>, UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, B1>, B1>, B0>, B1>, B0>, B1>, B0>, B0>, B1>, B1>, B0>, B0>, UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, B1>, B1>, B0>, B1>, B0>, B1>, B0>, B0>, B1>, B0>, B1>, B1>, B1>, 37 => [
    { U153, U630, UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, B1>, B0>, B1>, B0>, B0>, B0>, B0>, B0>, B0>, B1>, B1>, B0>, 122, 17, 123, 4, 30 },
    { U75, UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, B1>, B0>, B0>, B1>, B0>, B1>, B1>, B0>, B1>, B0>, B0>, UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, B1>, B1>, B1>, B1>, B1>, B0>, B0>, B1>, B0>, B0>, B0>, 46, 29, 47, 14, 28 },
    { U55, UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, B1>, B1>, B0>, B1>, B1>, B1>, B0>, B1>, B0>, B1>, B0>, UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, B1>, B0>, B1>, B1>, B0>, B0>, B1>, B0>, B0>, B1>, B0>, 24, 49, 25, 10, 30 },
    { U46, UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, B1>, B0>, B0>, B0>, B0>, B0>, B1>, B1>, B0>, B1>, B0>, B0>, UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, B1>, B0>, B0>, B0>, B1>, B0>, B0>, B1>, B0>, B0>, B0>, 15, 24, 16, 46, 30 },
  ],
  Version38, UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, B1>, B1>, B0>, B1>, B0>, B0>, B1>, B0>, B0>, B0>, B1>, B0>, UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, B1>, B1>, B0>, B1>, B1>, B1>, B1>, B1>, B0>, B0>, B1>, B1>, UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, B1>, B1>, B0>, B1>, B1>, B1>, B1>, B1>, B0>, B0>, B1>, B0>, B1>, 38 => [
    { U153, U660, UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, B1>, B0>, B1>, B0>, B1>, B0>, B0>, B0>, B1>, B1>, B1>, B0>, 122, 4, 123, 18, 30 },
    { U75, UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, B1>, B0>, B0>, B1>, B1>, B1>, B0>, B1>, B1>, B0>, B0>, UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, B1>, B0>, B0>, B0>, B0>, B0>, B1>, B1>, B0>, B1>, B1>, B0>, 46, 13, 47, 32, 28 },
    { U55, UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, B1>, B1>, B1>, B0>, B1>, B0>, B0>, B0>, B1>, B0>, B0>, UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, B1>, B0>, B1>, B1>, B1>, B0>, B1>, B1>, B1>, B1>, B0>, 24, 48, 25, 14, 30 },
    { U46, UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, B1>, B0>, B0>, B0>, B1>, B0>, B1>, B0>, B1>, B1>, B0>, B0>, UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, B1>, B0>, B0>, B0>, B1>, B1>, B1>, B0>, B1>, B1>, B0>, 15, 42, 16, 32, 30 },
  ],
  Version39, UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, B1>, B1>, B0>, B1>, B1>, B1>, B0>, B0>, B1>, B1>, B0>, B0>, UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, B1>, B1>, B1>, B0>, B1>, B0>, B0>, B1>, B1>, B1>, B1>, B0>, UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, B1>, B1>, B1>, B0>, B1>, B0>, B0>, B1>, B1>, B1>, B0>, B1>, B1>, 39 => [
    { U148, U720, UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, B1>, B0>, B1>, B0>, B1>, B1>, B1>, B1>, B1>, B1>, B0>, B0>, 117, 20, 118, 4, 30 },
    { U76, UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, B1>, B0>, B1>, B0>, B0>, B1>, B0>, B0>, B1>, B0>, B0>, UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, B1>, B0>, B0>, B0>, B1>, B0>, B1>, B0>, B1>, B0>, B0>, B0>, 47, 40, 48, 7, 28 },
    { U55, UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, B1>, B1>, B1>, B1>, B0>, B0>, B1>, B1>, B1>, B1>, B0>, UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, B1>, B1>, B0>, B0>, B0>, B1>, B0>, B1>, B1>, B1>, B0>, 24, 43, 25, 22, 30 },
    { U46, UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, B1>, B0>, B0>, B1>, B0>, B0>, B0>, B0>, B0>, B1>, B1>, B0>, UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, B1>, B0>, B0>, B1>, B1>, B0>, B0>, B0>, B1>, B1>, B0>, 15, 10, 16, 67, 30 },
  ],
  Version40, UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, B1>, B1>, B1>, B0>, B0>, B1>, B1>, B1>, B1>, B0>, B1>, B0>, UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, B1>, B1>, B1>, B1>, B0>, B1>, B0>, B0>, B1>, B1>, B0>, B1>, UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, B1>, B1>, B1>, B1>, B0>, B1>, B0>, B0>, B1>, B1>, B0>, B0>, B1>, 40 => [
    { U149, U750, UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, B1>, B0>, B1>, B1>, B1>, B0>, B0>, B0>, B1>, B1>, B0>, B0>, 118, 19, 119, 6, 30 },
    { U76, UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, B1>, B0>, B1>, B0>, B1>, B0>, B1>, B1>, B1>, B0>, B0>, UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, B1>, B0>, B0>, B1>, B0>, B0>, B0>, B1>, B1>, B1>, B1>, B0>, 47, 18, 48, 31, 28 },
    { U55, UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, B1>, B1>, B1>, B1>, B1>, B1>, B1>, B1>, B0>, B0>, B0>, UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, B1>, B1>, B0>, B1>, B0>, B0>, B0>, B0>, B0>, B1>, B0>, 24, 34, 25, 34, 30 },
    { U46, UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, B1>, B0>, B0>, B1>, B0>, B1>, B1>, B1>, B1>, B1>, B1>, B0>, UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UInt<UTerm, B1>, B0>, B0>, B1>, B1>, B1>, B1>, B1>, B1>, B0>, B0>, 15, 20, 16, 61, 30 },
  ]
}
