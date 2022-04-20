struct ScriptList {
    records: Vec<ScriptRecord>,
}

struct ScriptRecord {
    tag: Tag,
    script: Script,
}

struct Script {
    default_lang_sys: Option<LangSys>,
    lang_sys_records: Vec<LangSysRecord>,
}

struct LangSysRecord {
    tag: Tag,
    lang_sys: LangSys,
}

struct LangSys {
    required_feature_index: Option<u16>,
    feature_indices: Vec<u16>,
}

struct LayoutTable {
    script_list: ScriptList,
    feature_list: FeatureList,
    lookup_list: LookupList,
}

struct FeatureList {
    records: Vec<FeatureRecord>,
}

struct FeatureRecord {
    tag: Tag,
    offset: Feature,
}

struct Feature {
    lookup_list_indices: Vec<u16>,
}

struct LookupList {
    lookup_offsets: Vec<Lookup>,
}

struct Lookup {
    lookup_type: u16,
    lookup_flag: u16,
    subtables: Vec<Subtable>,
    mark_filtering_set: Option<u16>,
}

struct CoverageFormat1Owned {
    glyph_array: Vec<u16>,
}

struct CoverageFormat2Owned {
    range_records: Vec<RangeRecord>,
}

enum CoverageTableOwned {
    Format1(CoverageFormat1Owned),
    Format2(CoverageFormat2Owned),
}

// or just:
struct CoverageTableOwned2 {
    glyphs: BTreeSet<u16>,
}

struct ClassDefFormat1Owned {
    start_glyph_id: u16,
    class_value_array: Vec<u16>,
}

struct ClassDefFormat2Owned {
    class_range_records: Vec<ClassRangeRecord>,
}

enum ClassDefOwned {
    Format1(ClassDefFormat1Owned),
    Format2(ClassDefFormat2Owned),
}

struct SequenceLookupRecordOwned {
    sequence_index: u16,
    lookup_list_index: u16,
}

struct SequenceContextFormat1Owned {
    coverage: CoverageTableOwned,
    seq_rule_sets: Vec<SequenceRuleSetOwned>,
}

struct SequenceRuleSetOwned {
    seq_rules: Vec<SequenceRuleOwned>,
}

struct SequenceRuleOwned {
    input_sequence: Vec<GlyphId>,
    seq_lookup_records: Vec<SequenceLookupRecordOwned>,
}

struct SequenceContextFormat2Owned {
    coverage: CoverageTableOwned,
    class_def: ClassDefOwned,
    class_seq_rule_sets: Vec<ClassSequenceRuleSetOwned>,
}

struct ClassSequenceRuleSetOwned {
    class_seq_rules: Vec<ClassSequenceRuleOwned>,
}

struct ClassSequenceRuleOwned {
    input_sequence: Vec<u16>,
    seq_lookup_records: Vec<SequenceLookupRecordOwned>,
}

