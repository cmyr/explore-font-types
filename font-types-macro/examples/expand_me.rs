//! something to macro-expand when debugging

#![allow(dead_code)]

use font_types::{BigEndian, Fixed, Version16Dot16};

font_types::tables! {
    Post1_0 {
        /// 0x00010000 for version 1.0 0x00020000 for version 2.0
        version: BigEndian<Version16Dot16>,
        /// Italic angle in counter-clockwise degrees from the vertical.
        italic_angle: BigEndian<Fixed>,
    }

    /// [post (PostScript)](https://docs.microsoft.com/en-us/typography/opentype/spec/post#header) table
    Post2_0<'a> {
        /// 3.0
        version: BigEndian<Version16Dot16>,
        italic_angle: BigEndian<Fixed>,
        #[hidden]
        num_glyphs: BigEndian<u16>,
        /// Array of indices into the string data. See below for details.
        #[count(num_glyphs)]
        glyph_name_index: [BigEndian<u16>],
    }

    #[format(Version16Dot16)]
    #[generate_getters]
    enum Post<'a> {
        #[version(Version16Dot16::VERSION_1_0)]
        Post1_0(Post1_0),
        #[version(Version16Dot16::VERSION_2_0)]
        Post2_0(Post2_0<'a>),
    }
}

fn main() {}
