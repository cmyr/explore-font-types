use std::collections::{BTreeSet, HashMap};

use font_types::GlyphId;

pub struct Input {
    glyph_ids: BTreeSet<GlyphId>,
}

impl Input {
    pub fn from_gids(mut glyph_ids: BTreeSet<GlyphId>) -> Self {
        glyph_ids.insert(0); // always include .notdef
        Input { glyph_ids }
    }

    pub fn make_plan(&self) -> Plan {
        let gid_map = self
            .glyph_ids
            .iter()
            .enumerate()
            .map(|(i, gid)| (*gid, u16::try_from(i).unwrap()))
            .collect();
        Plan { gid_map }
    }
}

pub struct Plan {
    gid_map: HashMap<GlyphId, GlyphId>,
}

impl Plan {
    pub fn remap_gid(&self, gid: GlyphId) -> Option<GlyphId> {
        self.gid_map.get(&gid).copied()
    }
}

#[derive(Debug, Clone)]
pub struct Error {
    msg: String,
}

impl Error {
    pub fn new(s: impl Into<String>) -> Self {
        Error { msg: s.into() }
    }
}

pub trait Subset {
    /// Subset this object. Returns `true` if the object should be retained.
    fn subset(&mut self, plan: &Plan) -> Result<bool, Error>;
}
