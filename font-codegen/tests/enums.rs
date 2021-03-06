use font_types::{Version16Dot16, BigEndian};

const VERSION_0_5: Version16Dot16 = Version16Dot16::new(0, 5);
const VERSION_1_0: Version16Dot16 = Version16Dot16::new(1, 0);

font_types_macro::tables! {
    Maxp05 {
         version: BigEndian<Version16Dot16>,
         num_glyphs: BigEndian<u16>,
    }

    Maxp10 {
         version: BigEndian<Version16Dot16>,
         num_glyphs: BigEndian<u16>,
         max_points: BigEndian<u16>,
         max_contours: BigEndian<u16>,
         max_composite_points: BigEndian<u16>,
    }

    #[format(Version16Dot16)]
    enum Maxp {
        #[version(VERSION_0_5)]
        Version0_5(Maxp05),
        #[version(VERSION_1_0)]
        Version1_0(Maxp10),
    }
}

font_types_macro::tables! {
    One {
         one: BigEndian<u16>,
    }

    #[format(u16)]
    enum OneOrTwo {
        #[version(1)]
        One(One),
    }

    #[repr(u16)]
    enum EncodingId {
        Up = 1,
        Down = 2,
        Big = 3,
    }
}

fn main() {
    let encoding = EncodingId::new(1);
    assert_eq!(encoding, EncodingId::Up);
}
