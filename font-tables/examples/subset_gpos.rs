use std::collections::BTreeSet;

use font_tables::{
    compile::{FontBuilder, ToOwnedTable},
    subset::{Input, Subset},
    tables::{self, TableProvider},
    FontRef,
};

fn main() {
    let args = match flags::Args::from_env() {
        Ok(args) => args,
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(1);
        }
    };

    let gids = populate_gids(&args.gids);
    let input = Input::from_gids(gids);
    let plan = input.make_plan();

    let bytes = std::fs::read(&args.path).expect("no font file found");
    let font = FontRef::new(&bytes).expect("error reading font bytes");
    let gpos = font.gpos().expect("no gpos table found");
    let mut gpos = gpos.to_owned_table().expect("couldn't own gpos");
    gpos.subset(&plan).expect("subsetting failed");
    let bytes = font_tables::compile::dump_table(&gpos);

    let mut builder = FontBuilder::default();
    builder.add_table(tables::gpos::TAG, bytes);
    let bytes = builder.build();
    std::fs::write(&args.out, &bytes).unwrap();
}

fn populate_gids(gid_str: &str) -> BTreeSet<u16> {
    let mut result = BTreeSet::new();
    for gid in gid_str.split(',') {
        if let Some((start, end)) = gid.split_once('-') {
            let start: u16 = start.parse().unwrap();
            let end: u16 = end.parse().unwrap();
            assert!(start <= end, "invalid gid range {gid}");
            result.extend(start..=end);
        } else {
            result.insert(gid.parse().unwrap());
        }
    }
    result
}

mod flags {
    use std::path::PathBuf;

    xflags::xflags! {
        /// Generate font table representations
        cmd args
            required path: PathBuf
            {
                required -o, --out out: PathBuf
                required --gids gids: String
            }

    }
}
