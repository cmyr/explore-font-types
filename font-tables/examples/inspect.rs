//! Inspect a font, printing information about tables

use font_tables::{
    layout::{ClassDef, FeatureList, LangSys, LookupList, Script, ScriptList},
    tables::{self, TableProvider},
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
        print_gpos_info(&gpos);
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

fn print_gpos_info(gpos: &tables::gpos::Gpos) {
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

    let lookup_list: LookupList = gpos
        .resolve_offset(gpos.lookup_list_offset())
        .expect("failed to resolve lookuplist");
    println!("{} lookups:", lookup_list.lookup_count());
    for lookup in lookup_list.iter_lookups() {
        println!(
            "  type {}, {} subtables",
            lookup.lookup_type(),
            lookup.sub_table_count()
        );
    }
}

//fn print_gpos_sub_info(table: &tables::gpos::GposSubtable) {
//match table {
//tables::gpos::GposSubtable::Single(table) => match table {
//tables::gpos::SinglePos::Format1(table) => println!(
//"  {}: SinglePosFormat1, value_format {:b}",
//i,
//table.value_format()
//),
//tables::gpos::SinglePos::Format2(table) => println!(
//"  {}: SinglePosFormat2, count {} value_format {:b}",
//i,
//table.value_count(),
//table.value_format()
//),
//},
//tables::gpos::GposSubtable::Pair => println!("  {}: Pair", i),
//tables::gpos::GposSubtable::Cursive(_) => println!("  {}: Cursive", i),
//tables::gpos::GposSubtable::MarkToBase(_) => println!("  {}: MarkBase", i),
//tables::gpos::GposSubtable::MarkToLig(_) => println!("  {}: MarkToLig", i),
//tables::gpos::GposSubtable::MarkToMark(_) => println!("  {}: MarkToMark", i),
//tables::gpos::GposSubtable::Contextual => println!("  {}: Contextual", i),
//tables::gpos::GposSubtable::ChainContextual => println!("  {}: ChainContextual", i),
//tables::gpos::GposSubtable::Extension => println!("  {}: Extension", i),
//}
//}
