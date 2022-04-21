use font_types::{
    BigEndian, DynSizedArray, FontRead, MajorMinor, Offset16, Offset32, OffsetHost, Tag,
};

use self::value_record::{ValueFormat, ValueRecord};

mod value_record;

/// 'GPOS'
pub const TAG: Tag = Tag::new(b"GPOS");

const VERSION_1_0: MajorMinor = MajorMinor::new(1, 0);
const VERSION_1_1: MajorMinor = MajorMinor::new(1, 1);

font_types::tables! {
    /// [GPOS Version 1.0](https://docs.microsoft.com/en-us/typography/opentype/spec/gpos#gpos-header)
    #[offset_host]
    Gpos1_0<'a> {
        /// Major version of the GPOS table, = 1
        major_version: BigEndian<u16>,
        /// Minor version of the GPOS table, = 0
        minor_version: BigEndian<u16>,
        /// Offset to ScriptList table, from beginning of GPOS table
        script_list_offset: BigEndian<Offset16>,
        /// Offset to FeatureList table, from beginning of GPOS table
        feature_list_offset: BigEndian<Offset16>,
        /// Offset to LookupList table, from beginning of GPOS table
        lookup_list_offset: BigEndian<Offset16>,
    }

    /// [GPOS Version 1.1](https://docs.microsoft.com/en-us/typography/opentype/spec/gpos#gpos-header)
    #[offset_host]
    Gpos1_1<'a> {
        /// Major version of the GPOS table, = 1
        major_version: BigEndian<u16>,
        /// Minor version of the GPOS table, = 1
        minor_version: BigEndian<u16>,
        /// Offset to ScriptList table, from beginning of GPOS table
        script_list_offset: BigEndian<Offset16>,
        /// Offset to FeatureList table, from beginning of GPOS table
        feature_list_offset: BigEndian<Offset16>,
        /// Offset to LookupList table, from beginning of GPOS table
        lookup_list_offset: BigEndian<Offset16>,
        /// Offset to FeatureVariations table, from beginning of GPOS table
        /// (may be NULL)
        feature_variations_offset: BigEndian<Offset32>,
    }

    #[format(MajorMinor)]
    #[generate_getters]
    enum Gpos<'a> {
        #[version(VERSION_1_0)]
        Version1_0(Gpos1_0<'a>),
        #[version(VERSION_1_1)]
        Version1_1(Gpos1_1<'a>),
    }
}

pub enum GposSubtable<'a> {
    Single(SinglePos<'a>),
    Pair,
    Cursive(CursivePosFormat1<'a>),
    MarkToMark(MarkMarkPosFormat1<'a>),
    MarkToLig(MarkLigPosFormat1<'a>),
    MarkToBase(MarkBasePosFormat1<'a>),
    Contextual,
    ChainContextual,
    Extension,
}

impl<'a> GposSubtable<'a> {
    pub fn resolve(bytes: &'a [u8], type_: u16) -> Option<Self> {
        match type_ {
            1 => SinglePos::read(bytes).map(Self::Single),
            2 => Some(Self::Pair),
            3 => CursivePosFormat1::read(bytes).map(Self::Cursive),
            4 => MarkMarkPosFormat1::read(bytes).map(Self::MarkToMark),
            5 => MarkLigPosFormat1::read(bytes).map(Self::MarkToLig),
            6 => MarkBasePosFormat1::read(bytes).map(Self::MarkToBase),
            7 => Some(Self::Contextual),
            8 => Some(Self::ChainContextual),
            9 => Some(Self::Extension),
            _ => None,
        }
    }
}

font_types::tables! {

    ///// [Value Record](https://docs.microsoft.com/en-us/typography/opentype/spec/gpos#value-record)
    //ValueRecord {
        ///// Horizontal adjustment for placement, in design units.
        //x_placement: BigEndian<i16>,
        ///// Vertical adjustment for placement, in design units.
        //y_placement: BigEndian<i16>,
        ///// Horizontal adjustment for advance, in design units — only
        ///// used for horizontal layout.
        //x_advance: BigEndian<i16>,
        ///// Vertical adjustment for advance, in design units — only used
        ///// for vertical layout.
        //y_advance: BigEndian<i16>,
        ///// Offset to Device table (non-variable font) / VariationIndex
        ///// table (variable font) for horizontal placement, from beginning
        ///// of the immediate parent table (SinglePos or PairPosFormat2
        ///// lookup subtable, PairSet table within a PairPosFormat1 lookup
        ///// subtable) — may be NULL.
        //x_pla_device_offset: BigEndian<Offset16>,
        ///// Offset to Device table (non-variable font) / VariationIndex
        ///// table (variable font) for vertical placement, from beginning of
        ///// the immediate parent table (SinglePos or PairPosFormat2 lookup
        ///// subtable, PairSet table within a PairPosFormat1 lookup
        ///// subtable) — may be NULL.
        //y_pla_device_offset: BigEndian<Offset16>,
        ///// Offset to Device table (non-variable font) / VariationIndex
        ///// table (variable font) for horizontal advance, from beginning of
        ///// the immediate parent table (SinglePos or PairPosFormat2 lookup
        ///// subtable, PairSet table within a PairPosFormat1 lookup
        ///// subtable) — may be NULL.
        //x_adv_device_offset: BigEndian<Offset16>,
        ///// Offset to Device table (non-variable font) / VariationIndex
        ///// table (variable font) for vertical advance, from beginning of
        ///// the immediate parent table (SinglePos or PairPosFormat2 lookup
        ///// subtable, PairSet table within a PairPosFormat1 lookup
        ///// subtable) — may be NULL.
        //y_adv_device_offset: BigEndian<Offset16>,
    //}

    #[format(u16)]
    enum AnchorTable<'a> {
        #[version(1)]
        Format1(AnchorFormat1),
        #[version(2)]
        Format2(AnchorFormat2),
        //Format2(SinglePosFormat2<'a>),
        #[version(3)]
        Format3(AnchorFormat3<'a>),
    }

    /// [Anchor Table Format 1](https://docs.microsoft.com/en-us/typography/opentype/spec/gpos#anchor-table-format-1-design-units): Design Units
    AnchorFormat1 {
        /// Format identifier, = 1
        anchor_format: BigEndian<u16>,
        /// Horizontal value, in design units
        x_coordinate: BigEndian<i16>,
        /// Vertical value, in design units
        y_coordinate: BigEndian<i16>,
    }

    /// [Anchor Table Format 2](https://docs.microsoft.com/en-us/typography/opentype/spec/gpos#anchor-table-format-2-design-units-plus-contour-point): Design Units Plus Contour Point
    AnchorFormat2 {
        /// Format identifier, = 2
        anchor_format: BigEndian<u16>,
        /// Horizontal value, in design units
        x_coordinate: BigEndian<i16>,
        /// Vertical value, in design units
        y_coordinate: BigEndian<i16>,
        /// Index to glyph contour point
        anchor_point: BigEndian<u16>,
    }

    /// [Anchor Table Format 3]()https://docs.microsoft.com/en-us/typography/opentype/spec/gpos#anchor-table-format-3-design-units-plus-device-or-variationindex-tables: Design Units Plus Device or VariationIndex Tables
    #[offset_host]
    AnchorFormat3<'a> {
        /// Format identifier, = 3
        anchor_format: BigEndian<u16>,
        /// Horizontal value, in design units
        x_coordinate: BigEndian<i16>,
        /// Vertical value, in design units
        y_coordinate: BigEndian<i16>,
        /// Offset to Device table (non-variable font) / VariationIndex
        /// table (variable font) for X coordinate, from beginning of
        /// Anchor table (may be NULL)
        x_device_offset: BigEndian<Offset16>,
        /// Offset to Device table (non-variable font) / VariationIndex
        /// table (variable font) for Y coordinate, from beginning of
        /// Anchor table (may be NULL)
        y_device_offset: BigEndian<Offset16>,
    }

    /// [Mark Array Table](https://docs.microsoft.com/en-us/typography/opentype/spec/gpos#mark-array-table)
    MarkArray<'a> {
        /// Number of MarkRecords
        mark_count: BigEndian<u16>,
        /// Array of MarkRecords, ordered by corresponding glyphs in the
        /// associated mark Coverage table.
        #[count(mark_count)]
        mark_records: [MarkRecord],
    }

    /// Part of [MarkArray]
    MarkRecord {
        /// Class defined for the associated mark.
        mark_class: BigEndian<u16>,
        /// Offset to Anchor table, from beginning of MarkArray table.
        mark_anchor_offset: BigEndian<Offset16>,
    }
}

font_types::tables! {

    /// [Lookup Type 1](https://docs.microsoft.com/en-us/typography/opentype/spec/gpos#lookup-type-1-single-adjustment-positioning-subtable): Single Adjustment Positioning Subtable
    #[format(u16)]
    enum SinglePos<'a> {
        #[version(1)]
        Format1(SinglePosFormat1<'a>),
        #[version(2)]
        Format2(SinglePosFormat2<'a>),
    }

    /// [Single Adjustment Positioning Format 1](https://docs.microsoft.com/en-us/typography/opentype/spec/gpos#single-adjustment-positioning-format-1-single-positioning-value): Single Positioning Value
    #[offset_host]
    SinglePosFormat1<'a> {
        /// Format identifier: format = 1
        pos_format: BigEndian<u16>,
        /// Offset to Coverage table, from beginning of SinglePos subtable.
        coverage_offset: BigEndian<Offset16>,
        /// Defines the types of data in the ValueRecord.
        value_format: BigEndian<ValueFormat>,
        /// Defines positioning value(s) — applied to all glyphs in the
        /// Coverage table.
        #[read_with(value_format)]
        value_record: ValueRecord,
    }

    /// [Single Adjustment Positioning Format 2](https://docs.microsoft.com/en-us/typography/opentype/spec/gpos#single-adjustment-positioning-format-2-array-of-positioning-values): Array of Positioning Values
    #[offset_host]
    SinglePosFormat2<'a> {
        /// Format identifier: format = 2
        pos_format: BigEndian<u16>,
        /// Offset to Coverage table, from beginning of SinglePos subtable.
        coverage_offset: BigEndian<Offset16>,
        /// Defines the types of data in the ValueRecords.
        value_format: BigEndian<ValueFormat>,
        /// Number of ValueRecords — must equal glyphCount in the
        /// Coverage table.
        value_count: BigEndian<u16>,
        /// Array of ValueRecords — positioning values applied to glyphs.
        #[count_with(value_record_array_len, value_format, value_count)]
        #[read_with(value_format)]
        value_records: DynSizedArray<'a, ValueFormat, ValueRecord>,
    }
}

fn value_record_array_len(format: ValueFormat, count: u16) -> usize {
    count as usize * value_record_len(format)
}
fn value_record_len(format: ValueFormat) -> usize {
    format.bits().count_ones() as usize * std::mem::size_of::<u16>()
}

fn pair_value_record_len(count: u16, format1: ValueFormat, format2: ValueFormat) -> usize {
    std::mem::size_of::<u16>()
        + format1.record_byte_len()
        + format2.record_byte_len() * count as usize
}

font_types::tables! {

    ///// [Lookup Type 2](https://docs.microsoft.com/en-us/typography/opentype/spec/gpos#lookup-type-2-pair-adjustment-positioning-subtable): Pair Adjustment Positioning Subtable
    //PairPos {
        ///// //TODO
        //thing: fake,
    //}

    /// [Pair Adjustment Positioning Format 1](https://docs.microsoft.com/en-us/typography/opentype/spec/gpos#pair-adjustment-positioning-format-1-adjustments-for-glyph-pairs): Adjustments for Glyph Pairs
    #[offset_host]
    PairPosFormat1<'a> {
        /// Format identifier: format = 1
        pos_format: BigEndian<u16>,
        /// Offset to Coverage table, from beginning of PairPos subtable.
        coverage_offset: BigEndian<Offset16>,
        /// Defines the types of data in valueRecord1 — for the first
        /// glyph in the pair (may be zero).
        value_format1: BigEndian<ValueFormat>,
        /// Defines the types of data in valueRecord2 — for the second
        /// glyph in the pair (may be zero).
        value_format2: BigEndian<ValueFormat>,
        /// Number of PairSet tables
        pair_set_count: BigEndian<u16>,
        /// Array of offsets to PairSet tables. Offsets are from beginning
        /// of PairPos subtable, ordered by Coverage Index.
        #[count(pair_set_count)]
        pair_set_offsets: [BigEndian<Offset16>],
    }

    /// Part of [PairPosFormat1]
    #[read_args(value_format1 = "ValueFormat", value_format2 = "ValueFormat")]
    PairSet<'a> {
        /// Number of PairValueRecords
        pair_value_count: BigEndian<u16>,
        /// Array of PairValueRecords, ordered by glyph ID of the second
        /// glyph.
        #[count_with(pair_value_record_len, pair_value_count, value_format1, value_format2)]
        #[read_with(value_format1, value_format2)]
        pair_value_records: DynSizedArray<'a, (ValueFormat, ValueFormat), PairValueRecord<'a>>,
    }

    /// Part of [PairSet]
    #[read_args(value_format1 = "ValueFormat", value_format2 = "ValueFormat")]
    PairValueRecord<'a> {
        /// Glyph ID of second glyph in the pair (first glyph is listed in
        /// the Coverage table).
        second_glyph: BigEndian<u16>,
        /// Positioning data for the first glyph in the pair.
        #[read_with(value_format1)]
        value_record1: ValueRecord,
        /// Positioning data for the second glyph in the pair.
        #[read_with(value_format2)]
        value_record2: ValueRecord,
    }

    ///// [Pair Adjustment Positioning Format 2](https://docs.microsoft.com/en-us/typography/opentype/spec/gpos#pair-adjustment-positioning-format-2-class-pair-adjustment): Class Pair Adjustment
    //PairPosFormat2<'a> {
        ///// Format identifier: format = 2
        //pos_format: BigEndian<u16>,
        ///// Offset to Coverage table, from beginning of PairPos subtable.
        //coverage_offset: BigEndian<Offset16>,
        ///// ValueRecord definition — for the first glyph of the pair (may
        ///// be zero).
        //value_format1: BigEndian<u16>,
        ///// ValueRecord definition — for the second glyph of the pair
        ///// (may be zero).
        //value_format2: BigEndian<u16>,
        ///// Offset to ClassDef table, from beginning of PairPos subtable
        ///// — for the first glyph of the pair.
        //class_def1_offset: BigEndian<Offset16>,
        ///// Offset to ClassDef table, from beginning of PairPos subtable
        ///// — for the second glyph of the pair.
        //class_def2_offset: BigEndian<Offset16>,
        ///// Number of classes in classDef1 table — includes Class 0.
        //class1_count: BigEndian<u16>,
        ///// Number of classes in classDef2 table — includes Class 0.
        //class2_count: BigEndian<u16>,
        ///// Array of Class1 records, ordered by classes in classDef1.
        //#[count(class1_count)]
        ////#[count(0)]
        //class1_records: [Class1Record],
    //}

    ///// Part of [PairPosFormat2]
    //Class1Record<'a> {
        ///// Array of Class2 records, ordered by classes in classDef2.
        ////#[count(class2_count)]
        //#[count_all]
        //class2_records: [Class2Record],
    //}

    ///// Part of [PairPosFormat2]
    //Class2Record {
        ///// Positioning for first glyph — empty if valueFormat1 = 0.
        //value_record1: ValueRecord,
        ///// Positioning for second glyph — empty if valueFormat2 = 0.
        //value_record2: ValueRecord,
    //}
}

font_types::tables! {

    ///// [Lookup Type 3](https://docs.microsoft.com/en-us/typography/opentype/spec/gpos#lookup-type-3-cursive-attachment-positioning-subtable): Cursive Attachment Positioning Subtable
    //CursivePos {
        ///// //TODO
        //thing: fake,
    //}

    /// [Cursive Attachment Positioning Format 1](https://docs.microsoft.com/en-us/typography/opentype/spec/gpos#cursive-attachment-positioning-format1-cursive-attachment): Cursvie attachment
    #[offset_host]
    CursivePosFormat1<'a> {
        /// Format identifier: format = 1
        pos_format: BigEndian<u16>,
        /// Offset to Coverage table, from beginning of CursivePos subtable.
        coverage_offset: BigEndian<Offset16>,
        /// Number of EntryExit records
        entry_exit_count: BigEndian<u16>,
        /// Array of EntryExit records, in Coverage index order.
        #[count(entry_exit_count)]
        entry_exit_record: [EntryExitRecord],
    }

    /// Part of [CursivePosFormat1]
    EntryExitRecord {
        /// Offset to entryAnchor table, from beginning of CursivePos
        /// subtable (may be NULL).
        entry_anchor_offset: BigEndian<Offset16>,
        /// Offset to exitAnchor table, from beginning of CursivePos
        /// subtable (may be NULL).
        exit_anchor_offset: BigEndian<Offset16>,
    }
}

font_types::tables! {

    ///// [Lookup Type 4](https://docs.microsoft.com/en-us/typography/opentype/spec/gpos#lookup-type-4-mark-to-base-attachment-positioning-subtable): Mark-to-Base Attachment Positioning Subtable
    //MarkBasePos {
        ///// //TODO
        //thing: fake,
    //}

    /// [Mark-to-Base Attachment Positioning Format 1](https://docs.microsoft.com/en-us/typography/opentype/spec/gpos#mark-to-base-attachment-positioning-format-1-mark-to-base-attachment-point): Mark-to-base Attachment Point
    #[offset_host]
    MarkBasePosFormat1<'a> {
        /// Format identifier: format = 1
        pos_format: BigEndian<u16>,
        /// Offset to markCoverage table, from beginning of MarkBasePos
        /// subtable.
        mark_coverage_offset: BigEndian<Offset16>,
        /// Offset to baseCoverage table, from beginning of MarkBasePos
        /// subtable.
        base_coverage_offset: BigEndian<Offset16>,
        /// Number of classes defined for marks
        mark_class_count: BigEndian<u16>,
        /// Offset to MarkArray table, from beginning of MarkBasePos
        /// subtable.
        mark_array_offset: BigEndian<Offset16>,
        /// Offset to BaseArray table, from beginning of MarkBasePos
        /// subtable.
        base_array_offset: BigEndian<Offset16>,
    }

    ///// Part of [MarkBasePosFormat1]
    //BaseArray<'a> {
        ///// Number of BaseRecords
        //base_count: BigEndian<u16>,
        ///// Array of BaseRecords, in order of baseCoverage Index.
        //#[count(base_count)]
        //base_records: [BaseRecord<'a>],
    //}

    ///// Part of [BaseArray]
    //BaseRecord<'a> {
        ///// Array of offsets (one per mark class) to Anchor tables. Offsets
        ///// are from beginning of BaseArray table, ordered by class
        ///// (offsets may be NULL).
        ////#[count(mark_class_count)]
        //#[count(1)]
        //base_anchor_offsets: [BigEndian<Offset16>],
    //}
}

font_types::tables! {
    /// [Mark-to-Ligature Positioning Format 1](https://docs.microsoft.com/en-us/typography/opentype/spec/gpos#mark-to-ligature-attachment-positioning-format-1-mark-to-ligature-attachment): Mark-to-Ligature Attachment
    #[offset_host]
    MarkLigPosFormat1<'a> {
        /// Format identifier: format = 1
        pos_format: BigEndian<u16>,
        /// Offset to markCoverage table, from beginning of MarkLigPos
        /// subtable.
        mark_coverage_offset: BigEndian<Offset16>,
        /// Offset to ligatureCoverage table, from beginning of MarkLigPos
        /// subtable.
        ligature_coverage_offset: BigEndian<Offset16>,
        /// Number of defined mark classes
        mark_class_count: BigEndian<u16>,
        /// Offset to MarkArray table, from beginning of MarkLigPos
        /// subtable.
        mark_array_offset: BigEndian<Offset16>,
        /// Offset to LigatureArray table, from beginning of MarkLigPos
        /// subtable.
        ligature_array_offset: BigEndian<Offset16>,
    }

    /// Part of [MarkLigPosFormat1]
    LigatureArray<'a> {
        /// Number of LigatureAttach table offsets
        ligature_count: BigEndian<u16>,
        /// Array of offsets to LigatureAttach tables. Offsets are from
        /// beginning of LigatureArray table, ordered by ligatureCoverage
        /// index.
        #[count(ligature_count)]
        ligature_attach_offsets: [BigEndian<Offset16>],
    }

    ///// Part of [MarkLigPosFormat1]
    //LigatureAttach<'a> {
        ///// Number of ComponentRecords in this ligature
        //component_count: BigEndian<u16>,
        ///// Array of Component records, ordered in writing direction.
        //#[count(component_count)]
        //component_records: [ComponentRecord],
    //}

    ///// Part of [MarkLigPosFormat1]
    //ComponentRecord<'a> {
        ///// Array of offsets (one per class) to Anchor tables. Offsets are
        ///// from beginning of LigatureAttach table, ordered by class
        ///// (offsets may be NULL).
        //#[count(mark_class_count)]
        //ligature_anchor_offsets: [BigEndian<Offset16>],
    //}
}

font_types::tables! {
    /// [Mark-to-Mark Attachment Positioning Format 1](https://docs.microsoft.com/en-us/typography/opentype/spec/gpos#mark-to-mark-attachment-positioning-format-1-mark-to-mark-attachment): Mark-to-Mark Attachment
    #[offset_host]
    MarkMarkPosFormat1<'a> {
        /// Format identifier: format = 1
        pos_format: BigEndian<u16>,
        /// Offset to Combining Mark Coverage table, from beginning of
        /// MarkMarkPos subtable.
        mark1_coverage_offset: BigEndian<Offset16>,
        /// Offset to Base Mark Coverage table, from beginning of
        /// MarkMarkPos subtable.
        mark2_coverage_offset: BigEndian<Offset16>,
        /// Number of Combining Mark classes defined
        mark_class_count: BigEndian<u16>,
        /// Offset to MarkArray table for mark1, from beginning of
        /// MarkMarkPos subtable.
        mark1_array_offset: BigEndian<Offset16>,
        /// Offset to Mark2Array table for mark2, from beginning of
        /// MarkMarkPos subtable.
        mark2_array_offset: BigEndian<Offset16>,
    }

    ///// Part of [MarkMarkPosFormat1]
    //Mark2Array<'a> {
        ///// Number of Mark2 records
        //mark2_count: BigEndian<u16>,
        ///// Array of Mark2Records, in Coverage order.
        //#[count(mark2_count)]
        //mark2_records: [Mark2Record],
    //}

    ///// Part of [MarkMarkPosFormat1]
    //Mark2Record<'a> {
        ///// Array of offsets (one per class) to Anchor tables. Offsets are
        ///// from beginning of Mark2Array table, in class order (offsets may
        ///// be NULL).
        //#[count(mark_class_count)]
        //mark2_anchor_offsets: [BigEndian<Offset16>],
    //}
}

font_types::tables! {
    /// [Extension Positioning Subtable Format 1](https://docs.microsoft.com/en-us/typography/opentype/spec/gpos#extension-positioning-subtable-format-1)
    ExtensionPosFormat1 {
        /// Format identifier: format = 1
        pos_format: BigEndian<u16>,
        /// Lookup type of subtable referenced by extensionOffset (i.e. the
        /// extension subtable).
        extension_lookup_type: BigEndian<u16>,
        /// Offset to the extension subtable, of lookup type
        /// extensionLookupType, relative to the start of the
        /// ExtensionPosFormat1 subtable.
        extension_offset: BigEndian<Offset32>,
    }
}
