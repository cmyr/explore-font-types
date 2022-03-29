//! The [glyf (Glyph Data)](https://docs.microsoft.com/en-us/typography/opentype/spec/glyf) table

use font_types::{BigEndian, FontRead, Offset32, OffsetHost2, Tag};
use zerocopy::ByteSlice;

/// 'glyf'
pub const TAG: Tag = Tag::new(b"glyf");

font_types::tables! {
    /// The [glyf (Glyph Data)](https://docs.microsoft.com/en-us/typography/opentype/spec/glyf) table
    #[offset_host]
    Glyf<'a> {}

    /// The [Glyph Header](https://docs.microsoft.com/en-us/typography/opentype/spec/glyf#glyph-headers)
    GlyphHeader {
        /// If the number of contours is greater than or equal to zero,
        /// this is a simple glyph. If negative, this is a composite glyph
        /// — the value -1 should be used for composite glyphs.
        number_of_contours: BigEndian<i16>,
        /// Minimum x for coordinate data.
        x_min: BigEndian<i16>,
        /// Minimum y for coordinate data.
        y_min: BigEndian<i16>,
        /// Maximum x for coordinate data.
        x_max: BigEndian<i16>,
        /// Maximum y for coordinate data.
        y_max: BigEndian<i16>,
    }


    /// The [Glyph Header](https://docs.microsoft.com/en-us/typography/opentype/spec/glyf#glyph-headers)
    SimpleGlyph<'a> {
        header: GlyphHeader,
        #[count_with(get_n_contours, header)]
        end_pts_of_contours: [BigEndian<u16>],
        /// Total number of bytes for instructions. If instructionLength is
        /// zero, no instructions are present for this glyph, and this
        /// field is followed directly by the flags field.
        instruction_length: BigEndian<u16>,
        /// Array of instruction byte code for the glyph.
        #[count(instruction_length)]
        instructions: [BigEndian<u8>],
        #[count_all]
        //#[hidden]
        /// the raw data for flags & x/y coordinates
        glyph_data: [u8],

        ///// Array of flag elements. See below for details regarding the
        ///// number of flag array elements.
        //#[count(variable)]
        //flags: [BigEndian<SimpleGlyphFlags>],
        ///// Contour point x-coordinates. See below for details regarding
        ///// the number of coordinate array elements. Coordinate for the
        ///// first point is relative to (0,0); others are relative to
        ///// previous point.
        //#[count(variable)]
        //x_coordinates: [uint8 or int16],
        ///// Contour point y-coordinates. See below for details regarding
        ///// the number of coordinate array elements. Coordinate for the
        ///// first point is relative to (0,0); others are relative to
        ///// previous point.
        //#[count(variable)]
        //y_coordinates: [uint8 or int16],
    }

    /// Flags used in [SimpleGlyph]
#[flags(u8)]
    SimpleGlyphFlags {
        /// Bit 0: If set, the point is on the curve; otherwise, it is off
        /// the curve.
        ON_CURVE_POINT = 0x01,
        /// Bit 1: If set, the corresponding x-coordinate is 1 byte long,
        /// and the sign is determined by the
        /// X_IS_SAME_OR_POSITIVE_X_SHORT_VECTOR flag. If not set, its
        /// interpretation depends on the
        /// X_IS_SAME_OR_POSITIVE_X_SHORT_VECTOR flag: If that other flag
        /// is set, the x-coordinate is the same as the previous
        /// x-coordinate, and no element is added to the xCoordinates
        /// array. If both flags are not set, the corresponding element in
        /// the xCoordinates array is two bytes and interpreted as a signed
        /// integer. See the description of the
        /// X_IS_SAME_OR_POSITIVE_X_SHORT_VECTOR flag for additional
        /// information.
        X_SHORT_VECTOR = 0x02,
        /// Bit 2: If set, the corresponding y-coordinate is 1 byte long,
        /// and the sign is determined by the
        /// Y_IS_SAME_OR_POSITIVE_Y_SHORT_VECTOR flag. If not set, its
        /// interpretation depends on the
        /// Y_IS_SAME_OR_POSITIVE_Y_SHORT_VECTOR flag: If that other flag
        /// is set, the y-coordinate is the same as the previous
        /// y-coordinate, and no element is added to the yCoordinates
        /// array. If both flags are not set, the corresponding element in
        /// the yCoordinates array is two bytes and interpreted as a signed
        /// integer. See the description of the
        /// Y_IS_SAME_OR_POSITIVE_Y_SHORT_VECTOR flag for additional
        /// information.
        Y_SHORT_VECTOR = 0x04,
        /// Bit 3: If set, the next byte (read as unsigned) specifies the
        /// number of additional times this flag byte is to be repeated in
        /// the logical flags array — that is, the number of additional
        /// logical flag entries inserted after this entry. (In the
        /// expanded logical array, this bit is ignored.) In this way, the
        /// number of flags listed can be smaller than the number of points
        /// in the glyph description.
        REPEAT_FLAG = 0x08,
        /// Bit 4: This flag has two meanings, depending on how the
        /// X_SHORT_VECTOR flag is set. If X_SHORT_VECTOR is set, this bit
        /// describes the sign of the value, with 1 equalling positive and
        /// 0 negative. If X_SHORT_VECTOR is not set and this bit is set,
        /// then the current x-coordinate is the same as the previous
        /// x-coordinate. If X_SHORT_VECTOR is not set and this bit is also
        /// not set, the current x-coordinate is a signed 16-bit delta
        /// vector.
        X_IS_SAME_OR_POSITIVE_X_SHORT_VECTOR = 0x10,
        /// Bit 5: This flag has two meanings, depending on how the
        /// Y_SHORT_VECTOR flag is set. If Y_SHORT_VECTOR is set, this bit
        /// describes the sign of the value, with 1 equalling positive and
        /// 0 negative. If Y_SHORT_VECTOR is not set and this bit is set,
        /// then the current y-coordinate is the same as the previous
        /// y-coordinate. If Y_SHORT_VECTOR is not set and this bit is also
        /// not set, the current y-coordinate is a signed 16-bit delta
        /// vector.
        Y_IS_SAME_OR_POSITIVE_Y_SHORT_VECTOR = 0x20,
        /// Bit 6: If set, contours in the glyph description may overlap.
        /// Use of this flag is not required in OpenType — that is, it is
        /// valid to have contours overlap without having this flag set. It
        /// may affect behaviors in some platforms, however. (See the
        /// discussion of “Overlapping contours” in Apple’s
        /// specification for details regarding behavior in Apple
        /// platforms.) When used, it must be set on the first flag byte
        /// for the glyph. See additional details below.
        OVERLAP_SIMPLE = 0x40,

        ///// Bit 7 is reserved: set to zero.
        //Reserved = 0x80,
    }

    /// [CompositeGlyph](https://docs.microsoft.com/en-us/typography/opentype/spec/glyf#glyph-headers)
    CompositeGlyph<'a> {
        header: GlyphHeader,
        /// component flag
        flags: BigEndian<CompositeGlyphFlags>,
        /// glyph index of component
        glyph_index: BigEndian<u16>,
        #[count_all]
        offset_data: [u8],

        ///// x-offset for component or point number; type depends on bits 0
        ///// and 1 in component flags
        //argument1: uint8, int8, uint16 or int16,
        ///// y-offset for component or point number; type depends on bits 0
        ///// and 1 in component flags
        //argument2: uint8, int8, uint16 or int16,
    }

    /// Flags used in [CompositeGlyph]
#[flags(u16)]
    CompositeGlyphFlags {
        /// Bit 0: If this is set, the arguments are 16-bit (uint16 or
        /// int16); otherwise, they are bytes (uint8 or int8).
        ARG_1_AND_2_ARE_WORDS = 0x0001,
        /// Bit 1: If this is set, the arguments are signed xy values;
        /// otherwise, they are unsigned point numbers.
        ARGS_ARE_XY_VALUES = 0x0002,
        /// Bit 2: If set and ARGS_ARE_XY_VALUES is also set, the xy values
        /// are rounded to the nearest grid line. Ignored if
        /// ARGS_ARE_XY_VALUES is not set.
        ROUND_XY_TO_GRID = 0x0004,
        /// Bit 3: This indicates that there is a simple scale for the
        /// component. Otherwise, scale = 1.0.
        WE_HAVE_A_SCALE = 0x0008,
        /// Bit 5: Indicates at least one more glyph after this one.
        MORE_COMPONENTS = 0x0020,
        /// Bit 6: The x direction will use a different scale from the y
        /// direction.
        WE_HAVE_AN_X_AND_Y_SCALE = 0x0040,
        /// Bit 7: There is a 2 by 2 transformation that will be used to
        /// scale the component.
        WE_HAVE_A_TWO_BY_TWO = 0x0080,
        /// Bit 8: Following the last component are instructions for the
        /// composite character.
        WE_HAVE_INSTRUCTIONS = 0x0100,
        /// Bit 9: If set, this forces the aw and lsb (and rsb) for the
        /// composite to be equal to those from this component glyph. This
        /// works for hinted and unhinted glyphs.
        USE_MY_METRICS = 0x0200,
        /// Bit 10: If set, the components of the compound glyph overlap.
        /// Use of this flag is not required in OpenType — that is, it is
        /// valid to have components overlap without having this flag set.
        /// It may affect behaviors in some platforms, however. (See
        /// Apple’s specification for details regarding behavior in Apple
        /// platforms.) When used, it must be set on the flag word for the
        /// first component. See additional remarks, above, for the similar
        /// OVERLAP_SIMPLE flag used in simple-glyph descriptions.
        OVERLAP_COMPOUND = 0x0400,
        /// Bit 11: The composite is designed to have the component offset
        /// scaled. Ignored if ARGS_ARE_XY_VALUES is not set.
        SCALED_COMPONENT_OFFSET = 0x0800,
        /// Bit 12: The composite is designed not to have the component
        /// offset scaled. Ignored if ARGS_ARE_XY_VALUES is not set.
        UNSCALED_COMPONENT_OFFSET = 0x1000,

        ///// Bits 4, 13, 14 and 15 are reserved: set to 0.
        //Reserved = 0xE010,
    }

    #[format(i16)]
    //#[generate_getters]
    enum Glyph<'a> {
        #[version_with(non_negative_i16)]
        Simple(SimpleGlyph<'a>),
        #[version_with(i16::is_negative)]
        Composite(CompositeGlyph<'a>),
    }
}

fn non_negative_i16(val: i16) -> bool {
    !val.is_negative()
}

fn get_n_contours(header: &GlyphHeader) -> usize {
    header.number_of_contours() as usize
}

impl<'a, B: ByteSlice> Glyf<B> {
    pub fn resolve_glyph(&'a self, offset: Offset32) -> Option<Glyph<&'a [u8]>> {
        self.resolve_offset(offset)
    }
}

impl<B: ByteSlice> Glyph<B> {
    fn header(&self) -> &GlyphHeader {
        match self {
            Self::Simple(table) => table.header(),
            Self::Composite(table) => table.header(),
        }
    }

    pub fn number_of_contours(&self) -> i16 {
        self.header().number_of_contours()
    }

    pub fn x_min(&self) -> i16 {
        self.header().x_min()
    }

    pub fn y_min(&self) -> i16 {
        self.header().y_min()
    }

    pub fn x_max(&self) -> i16 {
        self.header().x_max()
    }

    pub fn y_max(&self) -> i16 {
        self.header().y_max()
    }
}

impl<B: ByteSlice> SimpleGlyph<B> {
    pub fn iter_points(&self) -> PointIter<'_> {
        self.iter_points_impl()
            .unwrap_or_else(|| PointIter::new(&[], &[], &[], &[]))
    }

    fn iter_points_impl(&self) -> Option<PointIter<'_>> {
        let end_points = self.end_pts_of_contours();
        let n_points = end_points.last()?.get().checked_add(1)?;
        let data = self.glyph_data();
        let lens = resolve_coords_len(data, n_points)?;
        let total_len = lens.flags + lens.x_coords + lens.y_coords;
        if data.len() < total_len as usize {
            return None;
        }

        let (flags, data) = data.split_at(lens.flags as usize);
        let (x_coords, y_coords) = data.split_at(lens.x_coords as usize);

        Some(PointIter::new(end_points, flags, x_coords, y_coords))
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Point {
    pub x: i16,
    pub y: i16,
}

#[derive(Clone, Copy, Debug)]
pub enum GlyphPoint {
    OffCurve(Point),
    OnCurve(Point),
    End(Point),
}

pub struct PointIter<'a> {
    end_points: &'a [BigEndian<u16>],
    cur_point: u16,
    flags: Cursor<'a>,
    x_coords: Cursor<'a>,
    y_coords: Cursor<'a>,
    flag_repeats: u8,
    cur_flags: SimpleGlyphFlags,
    cur_x: i16,
    cur_y: i16,
}

impl<'a> Iterator for PointIter<'a> {
    type Item = GlyphPoint;
    fn next(&mut self) -> Option<GlyphPoint> {
        let next_end = self.end_points.first()?.get();
        let is_end = next_end <= self.cur_point; // LE because points could be out of order?
        if is_end {
            self.end_points = &self.end_points[1..];
        }
        self.advance_flags();
        self.advance_points();
        self.cur_point = self.cur_point.saturating_add(1);

        let point = Point {
            x: self.cur_x,
            y: self.cur_y,
        };

        if is_end {
            Some(GlyphPoint::End(point))
        } else if self.cur_flags.contains(SimpleGlyphFlags::ON_CURVE_POINT) {
            Some(GlyphPoint::OnCurve(point))
        } else {
            Some(GlyphPoint::OffCurve(point))
        }
    }
}

impl<'a> PointIter<'a> {
    fn new(
        end_points: &'a [BigEndian<u16>],
        flags: &'a [u8],
        x_coords: &'a [u8],
        y_coords: &'a [u8],
    ) -> Self {
        Self {
            end_points,
            flags: Cursor::new(flags),
            x_coords: Cursor::new(x_coords),
            y_coords: Cursor::new(y_coords),
            cur_point: 0,
            flag_repeats: 0,
            cur_flags: SimpleGlyphFlags::empty(),
            cur_x: 0,
            cur_y: 0,
        }
    }

    fn advance_flags(&mut self) {
        if self.flag_repeats == 0 {
            self.cur_flags =
                SimpleGlyphFlags::from_bits_truncate(self.flags.bump().unwrap_or_default());
            self.flag_repeats = self
                .cur_flags
                .contains(SimpleGlyphFlags::REPEAT_FLAG)
                .then(|| self.flags.bump())
                .flatten()
                .unwrap_or(1);
        }
        self.flag_repeats -= 1;
    }

    fn advance_points(&mut self) {
        let x_short = self.cur_flags.contains(SimpleGlyphFlags::X_SHORT_VECTOR);
        let x_same_or_pos = self
            .cur_flags
            .contains(SimpleGlyphFlags::X_IS_SAME_OR_POSITIVE_X_SHORT_VECTOR);
        let y_short = self.cur_flags.contains(SimpleGlyphFlags::Y_SHORT_VECTOR);
        let y_same_or_pos = self
            .cur_flags
            .contains(SimpleGlyphFlags::Y_IS_SAME_OR_POSITIVE_Y_SHORT_VECTOR);

        let delta_x = match (x_short, x_same_or_pos) {
            (true, false) => -(self.x_coords.bump::<u8>().unwrap_or(0) as i16),
            (true, true) => self.x_coords.bump::<u8>().unwrap_or(0) as i16,
            (false, false) => self.x_coords.bump::<i16>().unwrap_or(0),
            _ => 0,
        };

        let delta_y = match (y_short, y_same_or_pos) {
            (true, false) => -(self.y_coords.bump::<u8>().unwrap_or(0) as i16),
            (true, true) => self.y_coords.bump::<u8>().unwrap_or(0) as i16,
            (false, false) => self.y_coords.bump::<i16>().unwrap_or(0),
            _ => 0,
        };

        self.cur_x = self.cur_x.wrapping_add(delta_x);
        self.cur_y = self.cur_y.wrapping_add(delta_y);
    }
}

//taken from ttf_parser https://docs.rs/ttf-parser/latest/src/ttf_parser/tables/glyf.rs.html#1-677
/// Resolves coordinate arrays length.
///
/// The length depends on *Simple Glyph Flags*, so we have to process them all to find it.
fn resolve_coords_len(data: &[u8], points_total: u16) -> Option<FieldLengths> {
    let mut cursor = Cursor::new(data);

    let mut flags_left = u32::from(points_total);
    //let mut repeats;
    let mut x_coords_len = 0;
    let mut y_coords_len = 0;
    //let mut flags_seen = 0;
    while flags_left > 0 {
        let flags: SimpleGlyphFlags = cursor.bump()?;

        // The number of times a glyph point repeats.
        let repeats = if flags.contains(SimpleGlyphFlags::REPEAT_FLAG) {
            let repeats: u8 = cursor.bump()?;
            u32::from(repeats) + 1
        } else {
            1
        };

        if repeats > flags_left {
            return None;
        }

        // Non-obfuscated code below.
        // Branchless version is surprisingly faster.
        //
        // if flags.x_short() {
        //     // Coordinate is 1 byte long.
        //     x_coords_len += repeats;
        // } else if !flags.x_is_same_or_positive_short() {
        //     // Coordinate is 2 bytes long.
        //     x_coords_len += repeats * 2;
        // }
        // if flags.y_short() {
        //     // Coordinate is 1 byte long.
        //     y_coords_len += repeats;
        // } else if !flags.y_is_same_or_positive_short() {
        //     // Coordinate is 2 bytes long.
        //     y_coords_len += repeats * 2;
        // }
        let x_short = SimpleGlyphFlags::X_SHORT_VECTOR;
        let x_long = SimpleGlyphFlags::X_SHORT_VECTOR
            | SimpleGlyphFlags::X_IS_SAME_OR_POSITIVE_X_SHORT_VECTOR;
        let y_short = SimpleGlyphFlags::Y_SHORT_VECTOR;
        let y_long = SimpleGlyphFlags::Y_SHORT_VECTOR
            | SimpleGlyphFlags::Y_IS_SAME_OR_POSITIVE_Y_SHORT_VECTOR;
        x_coords_len += ((flags & x_short).bits() != 0) as u32 * repeats;
        x_coords_len += ((flags & x_long).bits() == 0) as u32 * repeats;

        y_coords_len += ((flags & y_short).bits() != 0) as u32 * repeats;
        y_coords_len += ((flags & y_long).bits() == 0) as u32 * repeats;

        flags_left -= repeats;
    }

    Some(FieldLengths {
        flags: cursor.pos as u32,
        x_coords: x_coords_len,
        y_coords: y_coords_len,
    })
    //Some((flags_len, x_coords_len, y_coords_len))
}

struct FieldLengths {
    flags: u32,
    x_coords: u32,
    y_coords: u32,
}

/// A slice of bytes and an index into them.
struct Cursor<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> Cursor<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self { data, pos: 0 }
    }

    /// Attempt to read `T` at the current location, advancing if successful.
    fn bump<T: font_types::Scalar>(&mut self) -> Option<T> {
        let r = BigEndian::<T>::read(self.data)?;
        self.pos += std::mem::size_of::<T::Raw>();
        Some(r.get())
    }
}