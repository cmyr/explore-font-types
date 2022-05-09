//! compiling font tables

use std::collections::HashMap;

use font_types::{
    serialize::{Serialize2, Serializer2},
    FontRead, FontWrite, Offset, OffsetHost, OffsetLen, Uint24,
};

mod cmap;
mod gdef;
mod graph;
mod sketchpad;

use graph::{ObjectId, ObjectStore};

use self::graph::Graph;

#[cfg(test)]
mod hex_diff;

pub trait Table {
    /// Write our data and information about offsets into this [TableWriter].
    fn describe(&self, writer: &mut TableWriter);
}

//pub(crate) trait RawTable {
//fn describe(&self, writer: &mut RawTableWriter);
//}

#[derive(Debug)]
pub struct TableWriter {
    /// Finished tables, associated with an ObjectId; duplicate tables share an id.
    tables: ObjectStore,
    /// Tables currently being written.
    ///
    /// Tables are processed as they are encountered (as subtables)
    stack: Vec<TableData>,
}

//pub(crate) struct RawTableWriter<'a> {
//inner: &'a mut TableWriter,
//offset_bytes: &'a [u8],
//}

#[derive(Debug, Clone, Copy)]
pub struct OffsetMarker<T> {
    object: ObjectId,
    phantom: std::marker::PhantomData<T>,
}

impl<'a, T: Offset> OffsetMarker<T> {
    pub(crate) fn new(object: ObjectId) -> Self {
        OffsetMarker {
            object,
            phantom: std::marker::PhantomData,
        }
    }
}

pub struct OffsetMarker2<'a, T> {
    object: &'a dyn Table,
    phantom: std::marker::PhantomData<T>,
}

pub fn dump_table<T: Table>(table: &T) -> Vec<u8> {
    let mut writer = TableWriter::default();
    table.describe(&mut writer);
    let (root, graph) = writer.finish();
    dump_impl(root, graph)
}

fn dump_impl(root: ObjectId, graph: Graph) -> Vec<u8> {
    let sorted = graph.kahn_sort(root);

    let mut offsets = HashMap::new();
    let mut out = Vec::new();
    let mut off = 0;

    // first pass: write out bytes, record positions of offsets
    for id in &sorted {
        let node = graph.get_node(*id).unwrap();
        offsets.insert(*id, off);
        off += node.bytes.len() as u32;
        out.extend_from_slice(&node.bytes);
    }

    // second pass: write offsets
    let mut off = 0;
    for id in &sorted {
        let node = graph.get_node(*id).unwrap();
        for offset in &node.offsets {
            let abs_off = *offsets.get(&offset.object).unwrap();
            let rel_off = abs_off - off as u32;
            let buffer_pos = off + offset.pos as usize;
            let write_over = out.get_mut(buffer_pos..).unwrap();
            write_offset(write_over, offset.len, rel_off).unwrap();
        }
        off += node.bytes.len();
    }
    out
}

//TODO: some kind of error if an offset is OOB?
fn write_offset(at: &mut [u8], len: OffsetLen, resolved: u32) -> Result<(), ()> {
    let at = &mut at[..len as u8 as usize];
    match len {
        OffsetLen::Offset16 => at.copy_from_slice(
            u16::try_from(resolved)
                .map_err(|_| ())?
                .to_be_bytes()
                .as_slice(),
        ),
        OffsetLen::Offset24 => at.copy_from_slice(
            Uint24::checked_new(resolved)
                .ok_or(())?
                .to_be_bytes()
                .as_slice(),
        ),
        OffsetLen::Offset32 => at.copy_from_slice(resolved.to_be_bytes().as_slice()),
    }
    Ok(())
}

impl TableWriter {
    fn add_table(&mut self, table: &dyn Table) -> ObjectId {
        self.stack.push(TableData::default());
        table.describe(self);
        self.tables.add(self.stack.pop().unwrap())
    }

    fn add_table_raw<'a>(&mut self, table: &impl Serialize2) -> ObjectId {
        self.stack.push(TableData::default());
        table.serialize(self, &[]);
        self.tables.add(self.stack.pop().unwrap())
    }

    /// Finish this table, returning the root Id and the object graph.
    fn finish(mut self) -> (ObjectId, Graph) {
        // we start with one table which is only removed now
        let id = self.tables.add(self.stack.pop().unwrap());
        let graph = self.tables.into_graph();
        (id, graph)
    }

    fn dump(self) -> Vec<u8> {
        let (root, graph) = self.finish();
        dump_impl(root, graph)
    }

    pub fn write(&mut self, item: impl FontWrite) {
        let buf = self.stack.last_mut().unwrap();
        item.write(&mut buf.bytes)
    }

    pub fn write_offset0<T: Offset>(&mut self, obj: &dyn Table) {
        let obj_id = self.add_table(obj);
        let data = self.stack.last_mut().unwrap();
        data.add_offset::<T>(obj_id);
    }

    pub fn write_offset_marker<T: Offset>(&mut self, marker: OffsetMarker<T>) {
        self.stack
            .last_mut()
            .unwrap()
            .add_offset::<T>(marker.object);
    }
}

impl Serializer2 for TableWriter {
    fn write_be_bytes(&mut self, bytes: &[u8]) {
        self.write(bytes)
    }

    //fn resolve_offset<T: FontSerialize<'se>, O: Offset>(&self, offset: O) {
    //panic!("TableWriter struct cannot resolve raw offsets")
    //}

    fn write_offset<O, T>(&mut self, obj: &T)
    where
        O: Offset,
        T: Serialize2,
    {
        let obj_id = self.add_table_raw(obj);
        self.stack.last_mut().unwrap().add_offset::<O>(obj_id);
    }
}

//impl<'a> RawTableWriter<'a> {
//pub fn with_host(inner: &'a mut TableWriter, offset_bytes: &'a [u8]) -> Self {
//RawTableWriter {
//inner,
//offset_bytes,
//}
//}

//fn bytes_at_offset(&self, offset: impl Offset) -> &'a [u8] {
//offset
//.non_null()
//.and_then(|off| self.offset_bytes.get(off..))
//.unwrap_or_default()
//}
//}

//impl<'se> Serializer<'se> for RawTableWriter<'se> {
//fn write_be_bytes(&mut self, bytes: &[u8]) {
//self.inner.stack.last_mut().unwrap().write(bytes)
//}

//fn resolve_offset<T: FontSerialize<'se>, O: Offset>(&self, offest: O) {
//match T::read(self.bytes_at_offset(offest)) {
//Some(table) => self.write_offset::<O, _>(&table),
//None => self.write_be_bytes(O::SIZE.null_bytes()),
//}
//}

//fn write_offset<O, T>(&mut self, obj: &T) where O: Offset, T: FontSerialize<'se>  {
//self.inner.write_offset::<O, _>(obj)

//}
//}

impl Default for TableWriter {
    fn default() -> Self {
        TableWriter {
            tables: ObjectStore::default(),
            stack: vec![TableData::default()],
        }
    }
}

/// The encoded data for a given table, along with info on included offsets
#[derive(Debug, Default, Clone, Hash, PartialEq, Eq)]
pub(crate) struct TableData {
    bytes: Vec<u8>,
    offsets: Vec<OffsetRecord>,
}

/// The position and type of an offset, along with the id of the pointed-to entity
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
struct OffsetRecord {
    /// the position of the offset within the parent table
    pos: u32,
    /// the offset type (16/24/32 bit)
    len: OffsetLen,
    /// The object pointed to by the offset
    object: ObjectId,
}

impl TableData {
    fn add_offset<T: Offset>(&mut self, object: ObjectId) {
        self.offsets.push(OffsetRecord {
            pos: self.bytes.len() as u32,
            len: T::SIZE,
            object,
        });

        self.write(T::SIZE.null_bytes());
    }

    fn write(&mut self, bytes: &[u8]) {
        self.bytes.extend(bytes)
    }
}

#[cfg(test)]
#[rustfmt::skip::macros(assert_hex_eq)]
mod tests {
    use font_types::Offset16;

    use crate::assert_hex_eq;

    use super::*;

    struct Table1 {
        version: u16,
        records: Vec<SomeRecord>,
    }

    struct SomeRecord {
        value: u16,
        offset: Table2,
    }

    struct Table0 {
        version: u16,
        offsets: Vec<Table0a>,
    }

    struct Table0a {
        version: u16,
        offset: Table2,
    }

    struct Table2 {
        version: u16,
        bigness: u16,
    }

    impl Table for Table2 {
        fn describe(&self, writer: &mut TableWriter) {
            writer.write(self.version);
            writer.write(self.bigness);
        }
    }

    impl Table for Table1 {
        fn describe(&self, writer: &mut TableWriter) {
            writer.write(self.version);
            for record in &self.records {
                writer.write(record.value);
                writer.write_offset0::<Offset16>(&record.offset);
            }
        }
    }

    impl Table for Table0 {
        fn describe(&self, writer: &mut TableWriter) {
            writer.write(self.version);
            for offset in &self.offsets {
                writer.write_offset0::<Offset16>(offset);
            }
        }
    }

    impl Table for Table0a {
        fn describe(&self, writer: &mut TableWriter) {
            writer.write(self.version);
            writer.write_offset0::<Offset16>(&self.offset);
        }
    }

    #[test]
    fn simple_dedup() {
        let table = Table1 {
            version: 0xffff,
            records: vec![
                SomeRecord {
                    value: 0x1010,
                    offset: Table2 {
                        version: 0x2020,
                        bigness: 0x3030,
                    },
                },
                SomeRecord {
                    value: 0x4040,
                    offset: Table2 {
                        version: 0x5050,
                        bigness: 0x6060,
                    },
                },
                SomeRecord {
                    value: 0x6969,
                    offset: Table2 {
                        version: 0x2020,
                        bigness: 0x3030,
                    },
                },
            ],
        };

        let bytes = super::dump_table(&table);
        assert_hex_eq!(bytes.as_slice(), &[
            0xff, 0xff,

            0x10, 0x10,
            0x00, 0x12, //18

            0x40, 0x40,
            0x00, 0x0e, //14

            0x69, 0x69,
            0x00, 0x12, //18

            0x50, 0x50,
            0x60, 0x60,

            0x20, 0x20,
            0x30, 0x30,
        ]);
    }

    #[test]
    fn sibling_dedup() {
        let table = Table0 {
            version: 0xffff,
            offsets: vec![
                Table0a {
                    version: 0xa1a1,
                    offset: Table2 {
                        version: 0x2020,
                        bigness: 0x3030,
                    },
                },
                Table0a {
                    version: 0xa2a2,
                    offset: Table2 {
                        version: 0x2020,
                        bigness: 0x3030,
                    },
                },
            ],
        };

        let bytes = super::dump_table(&table);

        assert_hex_eq!(bytes.as_slice(), &[
            0xff, 0xff,

            0x00, 0x06, // offset1: 6
            0x00, 0x0a, // offset2: 10

            0xa1, 0xa1, // 0a #1
            0x00, 0x08, //8

            0xa2, 0xa2, // 0a #2
            0x00, 0x4, //4

            0x20, 0x20,
            0x30, 0x30,
        ]);
    }
}
