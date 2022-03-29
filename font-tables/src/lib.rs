//! font tables, records, etc.

#![cfg_attr(not(feature = "std"), no_std)]
// we autogenerate len methods in some places
#![allow(clippy::len_without_is_empty)]

#[cfg(any(feature = "std", test))]
#[allow(unused_imports)]
#[macro_use]
extern crate std;

#[cfg(all(not(feature = "std"), not(test)))]
#[macro_use]
extern crate core as std;

use font_types::{BigEndian, FontRead, Offset, Offset32, Tag};
use zerocopy::ByteSlice;

//pub mod layout;
pub mod tables;

/// A temporary type for accessing tables
pub struct FontRef<B> {
    pub table_directory: TableDirectory<B>,
    data: font_types::OffsetData<B>,
}

const TT_MAGIC: u32 = 0x00010000;
const OT_MAGIC: u32 = 0x4F54544F;

impl<B: zerocopy::ByteSlice> FontRef<B> {
    pub fn new(data: B) -> Result<Self, u32> {
        let num_tables = data
            .get(4..)
            .and_then(BigEndian::<u16>::read)
            .unwrap_or_else(|| 0.into());
        let record_len = num_tables.get() as usize * std::mem::size_of::<TableRecord>();
        let directory_len = (12 + record_len).min(data.len());
        let (head, tail) = data.split_at(directory_len);
        let table_directory = TableDirectory::read(head).ok_or(0x_dead_beef_u32)?;

        if [TT_MAGIC, OT_MAGIC].contains(&table_directory.sfnt_version()) {
            let data = font_types::OffsetData::new(tail, directory_len);
            Ok(FontRef {
                data,
                table_directory,
            })
        } else {
            Err(table_directory.sfnt_version())
        }
    }

    pub fn table_data(&self, tag: Tag) -> Option<&[u8]> {
        self.table_directory
            .table_records()
            .binary_search_by(|rec| rec.tag.get().cmp(&tag))
            .ok()
            .and_then(|idx| self.table_directory.table_records().get(idx))
            .and_then(|record| self.data.bytes_at_offset(record.offset.get()))
    }
}

impl<B: zerocopy::ByteSliceMut> FontRef<B> {
    pub fn table_data_mut(&mut self, tag: Tag) -> Option<&mut [u8]> {
        self.table_directory
            .table_records()
            .binary_search_by(|rec| rec.tag.get().cmp(&tag))
            .ok()
            .and_then(|idx| self.table_directory.table_records().get(idx))
            .and_then(|record| self.data.bytes_at_offset_mut(record.offset.get()))
    }
}

impl<B: ByteSlice> tables::TableProvider for FontRef<B> {
    fn data_for_tag(&self, tag: Tag) -> Option<&[u8]> {
        self.table_data(tag)
    }
}

font_types::tables! {
    TableDirectory<'a> {
        sfnt_version: BigEndian<u32>,
        num_tables: BigEndian<u16>,
        search_range: BigEndian<u16>,
        entry_selector: BigEndian<u16>,
        range_shift: BigEndian<u16>,
        #[count(num_tables)]
        table_records: [ TableRecord ],
    }

    /// Record for a table in a font.
    TableRecord {
        /// Table identifier.
        tag: BigEndian<Tag>,
        /// Checksum for the table.
        checksum: BigEndian<u32>,
        /// Offset from the beginning of the font data.
        offset: BigEndian<Offset32>,
        /// Length of the table.
        len: BigEndian<u32>,
    }
}
