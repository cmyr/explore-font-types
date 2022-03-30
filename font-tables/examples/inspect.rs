//! Inspect a font, printing information about tables

use font_tables::{
    tables::{self, TableProvider, TableProviderMut},
    FontRef,
};
use font_types::{BigEndian, Fixed, Offset, OffsetHost, OffsetHost2};
use zerocopy::ByteSlice;

fn main() {
    let path = std::env::args().nth(1).expect("missing path argument");
    let mut bytes = std::fs::read(path).unwrap();
    let mut font = FontRef::new(bytes.as_mut_slice()).unwrap();
    mutate(&mut font);
    print_font_info(&font);
}

fn mutate(font: &mut FontRef<&mut [u8]>) {
    let head = font.head_mut().expect("missing head");
    head.font_revision.set(Fixed::from_f64(420.69));
}

fn print_font_info<B: ByteSlice>(font: &FontRef<B>) {
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

fn print_name_info<B: ByteSlice>(name: &tables::name::Name<B>) {
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

fn print_post_info<B: ByteSlice>(post: &tables::post::Post<B>) {
    println!("\npost version {}", post.version());
    println!("  num glyphs: {}", post.num_names());
    println!("  italic angle {}", post.italic_angle());
    println!("  underline position {}", post.underline_position());
    println!("  underline thickness {}", post.underline_thickness());
    println!("  fixed pitch: {}", post.is_fixed_pitch() > 0);
}

fn print_stat_info<B: ByteSlice>(stat: &tables::stat::Stat<B>) {
    println!(
        "\nSTAT version {}.{}",
        stat.major_version(),
        stat.minor_version()
    );
    println!("  design axis count: {}", stat.design_axis_count());
    println!("  axis value count: {}", stat.axis_value_count());
}

fn print_cmap_info<B: ByteSlice>(cmap: &tables::cmap::Cmap<B>) {
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
