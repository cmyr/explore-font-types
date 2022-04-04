font_types::tables! {
    /// See [ValueRecord]
    #[flags(u16)]
    ValueFormat {
        /// Includes horizontal adjustment for placement
        X_PLACEMENT = 0x0001,
        /// Includes vertical adjustment for placement
        Y_PLACEMENT = 0x0002,
        /// Includes horizontal adjustment for advance
        X_ADVANCE = 0x0004,
        /// Includes vertical adjustment for advance
        Y_ADVANCE = 0x0008,
        /// Includes Device table (non-variable font) / VariationIndex
        /// table (variable font) for horizontal placement
        X_PLACEMENT_DEVICE = 0x0010,
        /// Includes Device table (non-variable font) / VariationIndex
        /// table (variable font) for vertical placement
        Y_PLACEMENT_DEVICE = 0x0020,
        /// Includes Device table (non-variable font) / VariationIndex
        /// table (variable font) for horizontal advance
        X_ADVANCE_DEVICE = 0x0040,
        /// Includes Device table (non-variable font) / VariationIndex
        /// table (variable font) for vertical advance
        Y_ADVANCE_DEVICE = 0x0080,
    }
}

impl ValueFormat {
    /// Return the number of bytes required to store a [`ValueRecord`] in this format.
    #[inline]
    pub fn record_byte_len(self) -> usize {
        self.bits().count_ones() as usize * 2
    }
}

pub struct ValueRecord<'a> {
    //data: LayoutVerified<&'a [u8], [BigEndian<i16>]>,
    data: &'a [u8],
    format: ValueFormat,
}

//pub struct RawValueRecord<'a> {
//data: LayoutVerified<&'a [u8], [BigEndian<i16>]>,
//format: ValueFormat,
//}

//macro_rules! get_valuerecord_field {
//($mask:expr, $format:expr, $data:expr) => {
//const MASK: u16 = $mask - 1;
//if $format.contains($mask) {
//let offset = ($format & MASK).count_ones();
//$data.get(offset..offset + 2).and_then(Uint16::read_from)
//} else {
//None
//}
//};
//}

impl<'a> ValueRecord<'a> {
    pub fn new(bytes: &'a [u8], format: ValueFormat) -> Option<Self> {
        todo!()
        //let count = format.bits().count_ones() as usize;
        //let (data, bytes) = LayoutVerified::new_slice_unaligned_from_prefix(bytes, count)?;
        //Some(ValueRecord {
        //data, format
        //})
        //if data.len() == format.bits().count_ones() as usize * std::mem::size_of::<i16>() {
        //Some(Self { data, format })
        //}
    }

    //pub fn x_placement(&self) -> Option<i16> {
    //self.format
    //.contains(ValueFormat::X_PLACEMENT)
    //.then(|| Uint16::read_from_prefix(self.data))
    //}

    //pub fn y_placement(&self) -> Option<i16> {
    //get_valuerecord_field!(ValueFormat::Y_PLACEMENT, self.format, self.data);
    //}

    //pub fn x_advance(&self) -> Option<i16> {
    //get_valuerecord_field!(ValueFormat::X_ADVANCE, self.format, self.data);
    //}

    //pub fn y_advance(&self) -> Option<i16> {
    //get_valuerecord_field!(ValueFormat::Y_ADVANCE, self.format, self.data);
    //}

    //pub fn y_pla_device_offset(&self) -> Option<i16> {
    //get_valuerecord_field!(ValueFormat::Y_PLACEMENT_DEVICE, self.format, self.data);
    //}

    //pub fn x_pla_device_offset(&self) -> Option<i16> {
    //get_valuerecord_field!(ValueFormat::X_PLACEMENT_DEVICE, self.format, self.data);
    //}

    //pub fn y_adv_device_offset(&self) -> Option<i16> {
    //get_valuerecord_field!(ValueFormat::Y_ADVANCE_DEVICE, self.format, self.data);
    //}

    //pub fn x_adv_device_offset(&self) -> Option<i16> {
    //get_valuerecord_field!(ValueFormat::X_ADVANCE_DEVICE, self.format, self.data);
    //}
}
