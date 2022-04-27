use std::collections::BTreeMap;

use super::{graph::ObjectId, OffsetMarker, Table, TableWriter};
use font_types::Offset16;

type GlyphId = u16;

pub struct GdefBuilder {
    writer: TableWriter,
    glyph_class_def: Option<OffsetMarker<Offset16>>,
    attach_list: Option<OffsetMarker<Offset16>>,
    lig_caret_list: Option<OffsetMarker<Offset16>>,
    mark_attach_class_def: Option<OffsetMarker<Offset16>>,
    mark_glyph_sets_def: Option<OffsetMarker<Offset16>>,
}

impl GdefBuilder {
    fn set_glyph_class_def(&mut self, obj: &dyn Table) {
        let obj_id = self.writer.add_table(obj);
        self.glyph_class_def = Some(OffsetMarker::new(obj_id));
    }

    fn set_attach_list(&mut self, obj: &dyn Table) {
        let obj_id = self.writer.add_table(obj);
        self.glyph_class_def = Some(OffsetMarker::new(obj_id));
    }
}

// ... I mean.. this basically works?? is there an actual issue?
// or more interestingly: what do we need, in order for this to work?
//

//struct LigCaretList {
//coverage: CoverageTable,
//lig_glyphs: Vec<LigGlyph>,
//}

mod owned_style {
    use super::*;

    struct LigCaretList {
        glyphs: BTreeMap<GlyphId, LigGlyph>,
    }

    struct LigGlyph(Vec<CaretValue>);

    impl LigCaretList {
        pub fn add_lig_glyph(&mut self, glyph: GlyphId, carets: Vec<CaretValue>) {
            self.glyphs.insert(glyph, LigGlyph(carets));
        }
    }

    enum CaretValue {
        Format1 {
            coordinate: i16,
        },
        Format2 {
            caret_value_point_index: u16,
        },
        Format3 {
            coordinate: i16,
            device: Box<dyn Table>,
        },
        // or:
        //Format3a {
        //coordinate: i16,
        //device: Device,
        //},
        //Format3b {
        //coordinate: i16,
        //variations: VariaionTable,
        //}
    }

    impl Table for LigCaretList {
        fn describe(&self, writer: &mut TableWriter) {
            //let coverage = self.glyphs.keys().copied().collect::<CoverageTable>();
            //writer.write_offset0::<Offset16>(&coverage);
            writer.write(self.glyphs.len() as u16);
            for caret in self.glyphs.values() {
                writer.write_offset::<Offset16>(caret);
            }
        }
    }

    impl Table for LigGlyph {
        fn describe(&self, writer: &mut TableWriter) {
            writer.write(u16::try_from(self.0.len()).expect("how do i do errors"));
            for caret in &self.0 {
                writer.write_offset::<Offset16>(caret);
            }
        }
    }

    impl Table for CaretValue {
        fn describe(&self, writer: &mut TableWriter) {
            match self {
                CaretValue::Format1 { coordinate } => {
                    writer.write(1u16);
                    writer.write(*coordinate);
                }
                CaretValue::Format2 {
                    caret_value_point_index,
                } => {
                    writer.write(2u16);
                    writer.write(*caret_value_point_index);
                }
                CaretValue::Format3 { coordinate, device } => {
                    writer.write(3u16);
                    writer.write(*coordinate);
                    writer.write_offset::<Offset16>(device.as_ref());
                }
            }
        }
    }
}

mod builder_style {
    use super::*;

    impl GdefBuilder {
        fn add_lig_caret_list(&mut self, f: impl FnOnce(&mut LigCaretListBuilder)) {
            let mut builder = LigCaretListBuilder {
                writer: &mut self.writer,
                glyphs: Default::default(),
            };
            f(&mut builder);
        }
    }

    struct LigCaretListBuilder<'builder> {
        writer: &'builder mut TableWriter,
        glyphs: BTreeMap<GlyphId, Vec<CaretValue>>,
    }

    impl<'builder> LigCaretListBuilder<'builder> {
        fn add_lig_glyph(&mut self, id: GlyphId, f: impl FnOnce(&mut LigGlyphBuilder)) {
            let mut builder = LigGlyphBuilder {
                carets: Vec::default(),
                writer: self.writer,
            };
            f(&mut builder);
            self.glyphs.insert(id, builder.carets);
        }
    }

    struct LigGlyphBuilder<'builder> {
        carets: Vec<CaretValue>,
        writer: &'builder mut TableWriter,
    }

    impl<'builder> LigGlyphBuilder<'builder> {
        fn add_caret_format1(&mut self, coordinate: i16) {
            self.carets.push(CaretValue::Format1 { coordinate });
        }

        fn add_caret_format2(&mut self, caret_value_point_index: u16) {
            self.carets.push(CaretValue::Format2 {
                caret_value_point_index,
            });
        }

        fn add_caret_format3(&mut self, coordinate: i16, device: &dyn Table) {
            let device = self.writer.add_table(device);
            self.carets.push(CaretValue::Format3 { coordinate, device });
        }
    }

    enum CaretValue {
        Format1 { coordinate: i16 },
        Format2 { caret_value_point_index: u16 },
        Format3 { coordinate: i16, device: ObjectId },
    }
}

//struct LigGlyph
