//! Inspect a font, printing information about tables

use std::collections::{BTreeMap, HashMap};

use font_tables::{
    layout::{ClassDef, FeatureList, LangSys, Lookup, LookupList, Script, ScriptList},
    tables::{
        self,
        cmap::Cmap4,
        gpos::{GposSubtable, PairPos, SinglePos, ValueRecord},
        TableProvider,
    },
    FontRef,
};
use font_types::{BigEndian, Offset, OffsetHost};

fn main() {
    let path = std::env::args().nth(1).expect("missing path argument");
    let bytes = std::fs::read(path).unwrap();
    let font = FontRef::new(&bytes).unwrap();
    print_font_info(&font);
}

fn print_font_info(font: &FontRef) {
    let num_tables = font.table_directory.num_tables();
    println!("loaded {} tables", num_tables);
    for record in font.table_directory.table_records() {
        println!(
            "table {} at {:?} (len {})",
            record.tag.get(),
            record.offset.get(),
            record.len.get()
        );
    }

    let head = font.head().expect("missing head");
    print_head_info(&head);

    if let Some(name) = font.name() {
        print_name_info(&name);
    }

    if let Some(post) = font.post() {
        print_post_info(&post);
    }

    if let Some(hhea) = font.hhea() {
        print_hhea_info(&hhea);
    }
    if let Some(maxp) = font.maxp() {
        print_maxp_info(&maxp);
        let long_loca = head.index_to_loc_format() == 1;
        if let Some(loca) = font.loca(maxp.num_glyphs(), long_loca) {
            let glyf = font.glyf().expect("missing glyf table");
            let mut simple_glyphs = 0;
            let mut composite_glyphs = 0;
            let mut total_points = 0;
            let mut x_min = 0;
            let mut y_min = 0;
            let mut x_max = 0;
            let mut y_max = 0;

            println!("\nglyf/loca:");
            for (i, offset) in loca
                .iter()
                .filter(|off| off.non_null().is_some())
                .enumerate()
            {
                match glyf.resolve_glyph(offset) {
                    Some(glyph) => {
                        x_min = x_min.min(glyph.x_min());
                        y_min = y_min.min(glyph.y_min());
                        x_max = x_max.max(glyph.x_max());
                        y_max = y_max.max(glyph.y_max());
                        if let tables::glyf::Glyph::Simple(glyph) = glyph {
                            simple_glyphs += 1;
                            total_points += glyph.iter_points().count();
                        } else {
                            composite_glyphs += 1;
                        }
                    }
                    None => {
                        eprintln!("  unable to load glyph {} at {:?}", i, offset);
                    }
                }
            }

            println!("  simple glyphs: {}", simple_glyphs);
            println!("  composite glyphs: {}", composite_glyphs);
            println!("  total points: {}", total_points);

            println!("  x_min: {}", x_min);
            println!("  y_min: {}", y_min);
            println!("  x_max: {}", x_max);
            println!("  y_max: {}", y_max);
        }
    }
    if let Some(cmap) = font.cmap() {
        print_cmap_info(&cmap);
    }

    if let Some(stat) = font.stat() {
        print_stat_info(&stat);
    }
    if let Some(gdef) = font.gdef() {
        print_gdef_info(&gdef);
    }

    if let Some(gpos) = font.gpos() {
        print_gpos_info(&gpos, font.cmap().as_ref());
    } else {
        println!("GPOS: None");
    }
}

fn print_head_info(head: &tables::head::Head) {
    println!(
        "\nhead version {}.{}",
        head.major_version, head.minor_version
    );
    println!("  revision {}", head.font_revision);
    println!("  upm {}", head.units_per_em);
    println!("  x/y min: {}, {}", head.x_min, head.y_min);
    println!("  x/y max: {}, {}", head.x_max, head.y_max);
}

fn print_hhea_info(hhea: &tables::hhea::Hhea) {
    println!(
        "\nhhea version {}.{}",
        hhea.major_version(),
        hhea.minor_version()
    );
    println!("  ascender {}", hhea.ascender());
    println!("  descender {}", hhea.descender());
    println!("  line gap {}", hhea.line_gap());
    println!("  max advance {}", hhea.advance_width_max());
    println!("  min left sidebearing {}", hhea.min_left_side_bearing());
    println!("  min right sidebearing {}", hhea.min_right_side_bearing());
}

fn print_maxp_info(maxp: &tables::maxp::Maxp) {
    println!("\nmaxp version {}", maxp.version());
    println!("  num_glyphs: {}", maxp.num_glyphs());
}

fn print_name_info(name: &tables::name::Name) {
    println!("\nname version {}", name.version());
    println!("  records: {}", name.count());

    let mut plat_id = 101;
    let mut enc_id = u16::MAX;
    for record in name.name_record() {
        if record.platform_id() != plat_id || record.encoding_id() != enc_id {
            plat_id = record.platform_id();
            enc_id = record.encoding_id();
            println!("  platform {} encoding {}:", plat_id, enc_id);
        }
        if let Some(entry) = name.resolve(record) {
            println!("    {}: '{}'", record.name_id(), entry);
        } else {
            println!("    {} (unknown encoding)", record.name_id());
        }
    }
}

fn print_post_info(post: &tables::post::Post) {
    println!("\npost version {}", post.version());
    println!("  num glyphs: {}", post.num_names());
    println!("  italic angle {}", post.italic_angle());
    println!("  underline position {}", post.underline_position());
    println!("  underline thickness {}", post.underline_thickness());
    println!("  fixed pitch: {}", post.is_fixed_pitch() > 0);
}

fn print_stat_info(stat: &tables::stat::Stat) {
    println!(
        "\nSTAT version {}.{}",
        stat.major_version(),
        stat.minor_version()
    );
    println!("  design axis count: {}", stat.design_axis_count());
    println!("  axis value count: {}", stat.axis_value_count());
}

fn print_cmap_info(cmap: &tables::cmap::Cmap) {
    println!(
        "\ncmap version {}, {} tables",
        cmap.version(),
        cmap.num_tables()
    );

    for record in cmap.encoding_records() {
        let platform_id = tables::cmap::PlatformId::new(record.platform_id());
        let encoding_id = record.encoding_id();
        let format: BigEndian<u16> = cmap
            .resolve_offset(record.subtable_offset())
            .expect("failed to resolve subtable");
        println!("  ({:?}, {}) format {}", platform_id, encoding_id, format);
        if format.get() == 4 {
            let subtable: Cmap4 = cmap.resolve_offset(record.subtable_offset()).unwrap();
            let reverse = subtable.reverse();
            print_char_ranges(reverse.values().copied());
        }
    }
}

enum CharIterState {
    None, // we haven't started
    Single(char),
    Contiguous { idx: usize, last: char },
    Discontiguous { idx: usize, last: char },
}

/// I thought this would be useful, but glyph ids are rarely in contiguous
/// unicode ranges :shrug:
fn print_char_ranges(chars: impl Iterator<Item = char>) {
    let chrs = chars.collect::<Vec<_>>();
    //chrs.sort();
    let mut state = CharIterState::None;
    for (i, chr) in chrs.iter().copied().enumerate() {
        state = match state {
            CharIterState::None => CharIterState::Single(chr),
            CharIterState::Single(prev) if (chr as u32).saturating_sub(prev as u32) == 1 => {
                CharIterState::Contiguous {
                    idx: i - 1,
                    last: chr,
                }
            }
            CharIterState::Single(_) => CharIterState::Discontiguous {
                idx: i - 1,
                last: chr,
            },
            CharIterState::Discontiguous { idx, last }
                if (chr as u32).saturating_sub(last as u32) == 1 =>
            {
                print_singles(&chrs[idx..i], false);
                CharIterState::Single(chr)
            }
            CharIterState::Discontiguous { idx, .. } => {
                CharIterState::Discontiguous { idx, last: chr }
            }
            CharIterState::Contiguous { idx, last }
                if (chr as u32).saturating_sub(last as u32) > 1 =>
            {
                print_range(&chrs[idx..i], false);
                CharIterState::Single(chr)
            }
            CharIterState::Contiguous { idx, .. } => CharIterState::Contiguous { idx, last: chr },
        };
    }
    match state {
        CharIterState::Contiguous { idx, .. } => print_range(&chrs[idx..], true),
        CharIterState::Discontiguous { idx, .. } => print_singles(&chrs[idx..], true),
        CharIterState::Single(c) => println!("{c}"),
        _ => (),
    }

    fn print_range(chars: &[char], newline: bool) {
        if chars.len() < 4 {
            return print_singles(chars, newline);
        }
        print!(
            "({:?}..{:?}) ",
            chars.first().unwrap(),
            chars.last().unwrap()
        );
        if newline {
            println!()
        }
    }
    fn print_singles(chars: &[char], newline: bool) {
        for c in chars {
            print!("{c} ");
        }
        if newline {
            println!()
        }
    }
}

fn print_gdef_info(gdef: &tables::gdef::Gdef) {
    println!(
        "\nGDEF version {}.{}",
        gdef.major_version(),
        gdef.minor_version()
    );
    if let Some(class_def) = gdef.glyph_class_def() {
        let format = match class_def {
            ClassDef::Format1(_) => 1,
            ClassDef::Format2(_) => 2,
        };
        println!("   ClassDef format {}", format);
    }

    if let Some(attach_list) = gdef.attach_list() {
        println!("  AttachList ({} glyphs)", attach_list.glyph_count());
    }

    if let Some(lig_caret_list) = gdef.lig_caret_list() {
        println!(
            "  LigCaretList ({} glyphs)",
            lig_caret_list.lig_glyph_count()
        );
    }

    if let Some(class_def) = gdef.mark_attach_class_def() {
        let format = match class_def {
            ClassDef::Format1(_) => 1,
            ClassDef::Format2(_) => 2,
        };
        println!("   MarkAttach ClassDef format {}", format);
    }

    if let Some(glyph_sets) = gdef.mark_glyph_sets_def() {
        println!(
            "  MarkGlyphSets ({} glyphs)",
            glyph_sets.mark_glyph_set_count()
        );
    }
}

fn print_gpos_info(gpos: &tables::gpos::Gpos, cmap: Option<&tables::cmap::Cmap>) {
    println!(
        "\nGPOS version {}.{}",
        gpos.major_version(),
        gpos.minor_version()
    );
    let script_list: ScriptList = gpos
        .resolve_offset(gpos.script_list_offset())
        .expect("failed to get script list");
    let feature_list: FeatureList = gpos
        .resolve_offset(gpos.feature_list_offset())
        .expect("failed to resolve feature list");
    println!("{} scripts:", script_list.script_count());
    for record in script_list.script_records() {
        let script: Script = script_list
            .resolve_offset(record.script_offset())
            .expect("failed to get script");
        println!("  {}: {:?}", record.script_tag(), record.script_offset());
        for lang_sys in script.lang_sys_records() {
            let record: LangSys = script
                .resolve_offset(lang_sys.lang_sys_offset())
                .expect("couldn't resolve lang_sys");
            print!("    {} (", lang_sys.lang_sys_tag(),);
            let mut first = true;
            for idx in record.feature_indices() {
                if let Some(feat) = feature_list.feature_records().get(idx.get() as usize) {
                    if !first {
                        print!(" ");
                    }
                    print!(
                        "{}({})",
                        feat.feature_tag(),
                        feat.feature_offset().non_null().unwrap_or_default()
                    );
                    first = false;
                }
            }
            println!(")");
        }
    }

    let reverse_cmap = cmap
        .and_then(|table| {
            table.encoding_records().iter().find_map(|record| {
                table
                    .resolve_offset::<Cmap4, _>(record.subtable_offset())
                    .map(|cmap4| cmap4.reverse())
            })
        })
        .unwrap_or_default();
    let lookup_list: LookupList = gpos
        .resolve_offset(gpos.lookup_list_offset())
        .expect("failed to resolve lookuplist");
    println!("{} lookups:", lookup_list.lookup_count());
    for lookup in lookup_list.iter_lookups() {
        print_gpos_lookup_info(&lookup, &reverse_cmap);
    }
}

fn print_gpos_lookup_info(lookup: &Lookup, cmap: &BTreeMap<u16, char>) {
    println!(
        "  type {}, {} subtables",
        lookup.lookup_type(),
        lookup.sub_table_count()
    );

    for (i, subtable) in lookup.iter_subtables_gpos().enumerate() {
        println!("  subtable {} format {}", i, subtable.format());
        let coverage_ids = match subtable.coverage().map(|x| x.glyph_ids()) {
            Some(x) => x,
            None => continue,
        };
        match subtable {
            GposSubtable::Single(SinglePos::Format1(table)) => {
                let record = table.value_record();
                println!("  record: {:?}", record);
            }
            GposSubtable::Single(SinglePos::Format2(table)) => {
                let records = table.value_records().iter().collect::<Vec<_>>();
                println!("  {:?}", records);
            }
            GposSubtable::Pair(PairPos::Format1(table)) => {
                let mut records = HashMap::new();
                for (idx, id) in coverage_ids.iter().enumerate() {
                    let offset = table.pair_set_offsets().get(idx).map(|x| x.get()).unwrap();
                    records
                        .entry(offset)
                        .or_insert_with(|| Vec::new())
                        .push(cmap.get(id).unwrap())
                }

                for (offset, glyphs) in &records {
                    let pair_set = table.get_pair_set(*offset).unwrap();
                    if glyphs.len() == 1 {
                        print!("    {} ", glyphs.first().unwrap());
                    } else {
                        print!("   ");
                        glyphs.iter().for_each(|g| print!(" {g}"));
                        print!("\n  ");
                        //print!("    {:?}\n      ", glyphs);
                    }

                    for record in pair_set.pair_value_records().iter() {
                        println!(
                            "+ {}: {}, {}",
                            cmap.get(&record.second_glyph()).unwrap(),
                            ValueRecordFmt(record.value_record1().clone()),
                            ValueRecordFmt(record.value_record2().clone())
                        );
                    }
                }
            }
            GposSubtable::Pair(PairPos::Format2(table)) => {
                let class1 = table
                    .resolve_offset::<ClassDef, _>(table.class_def1_offset())
                    .unwrap()
                    .to_class_list();
                let class2 = table
                    .resolve_offset::<ClassDef, _>(table.class_def2_offset())
                    .unwrap()
                    .to_class_list();

                for ((class1, glyphs1), c1_record) in
                    class1.iter().zip(table.class1_records().iter())
                {
                    let glyphs1 = glyphs1.iter().map(|gid| {
                        cmap.get(gid)
                            .copied()
                            .unwrap_or(char::REPLACEMENT_CHARACTER)
                    });
                    print!("    class {class1} glyphs:");
                    glyphs1.for_each(|g| print!(" {g}"));
                    println!();
                    for ((class2, glyphs2), c2_record) in
                        class2.iter().zip(c1_record.class2_records().iter())
                    {
                        let glyphs2 = glyphs2.iter().map(|gid| {
                            cmap.get(gid)
                                .copied()
                                .unwrap_or(char::REPLACEMENT_CHARACTER)
                        });
                        print!("  + class {class2}");
                        glyphs2.for_each(|g| print!(" {g}"));
                        println!(
                            "\n    {}, {}",
                            ValueRecordFmt(c2_record.value_record1().clone()),
                            ValueRecordFmt(c2_record.value_record2().clone())
                        );
                    }
                }
            }

            _ => (),
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct FmtField {
    name: &'static str,
    value: i16,
}

impl std::fmt::Display for FmtField {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}: {}", self.name, self.value)
    }
}

struct ValueRecordFmt(ValueRecord);

impl std::fmt::Display for ValueRecordFmt {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut fields = [None; 4];
        fields[0] = self.0.x_placement.map(|v| FmtField {
            name: "x_place",
            value: v.get(),
        });
        fields[1] = self.0.x_advance.map(|v| FmtField {
            name: "x_adv",
            value: v.get(),
        });
        fields[2] = self.0.y_placement.map(|v| FmtField {
            name: "y_place",
            value: v.get(),
        });
        fields[3] = self.0.y_advance.map(|v| FmtField {
            name: "y_adv",
            value: v.get(),
        });
        write!(f, "(")?;
        for (i, field) in fields.iter().flatten().enumerate() {
            if i > 0 {
                write!(f, " ")?;
            }
            write!(f, "{field}")?;
        }
        if fields.iter().all(Option::is_none) {
            write!(f, "empty")?;
        }
        write!(f, ")")
    }
}
