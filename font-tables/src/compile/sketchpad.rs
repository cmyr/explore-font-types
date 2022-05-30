use std::rc::Rc;

use font_types::{BigEndian, Offset16, Scalar};

struct OwnedTable<T: TableInfo> {
    shape: T::Info,
    data: Rc<[u8]>,
}

impl<T: TableInfo> OwnedTable<T> {
    pub fn table_ref(&self) -> TableRef<T> {
        TableRef {
            shape: self.shape,
            data: self.data.as_ref(),
        }
    }

    fn is_unique(&self) -> bool {
        Rc::weak_count(&self.data) == 0 && Rc::strong_count(&self.data) == 1
    }

    pub fn as_mut(&mut self) -> TableMut<T> {
        if !self.is_unique() {
            self.data = Rc::from(&*self.data);
        }
        TableMut {
            shape: self.shape,
            data: Rc::get_mut(&mut self.data).unwrap(),
        }
    }
}

trait TableInfo {
    type Info: Copy;
    fn from_bytes(bytes: &[u8]) -> Option<Self::Info>;
}

struct TableRef<'a, T: TableInfo> {
    shape: T::Info,
    data: &'a [u8],
}

/// does this make sense??
struct TableMut<'a, T: TableInfo> {
    shape: T::Info,
    data: &'a mut [u8],
}

impl<T: TableInfo> TableRef<'_, T> {
    fn to_owned(&self) -> OwnedTable<T> {
        OwnedTable {
            data: self.data.into(),
            shape: self.shape,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct Gdef;

#[derive(Debug, Clone, Copy)]
struct GdefInfo;

impl<'a> TableRef<'a, Gdef> {
    fn new(data: &'a [u8]) -> Option<Self> {
        // verify that we have enough data
        Gdef::from_bytes(data).map(|shape| TableRef { shape, data })
    }
}
impl TableInfo for Gdef {
    type Info = GdefInfo;
    fn from_bytes(_bytes: &[u8]) -> Option<Self::Info> {
        Some(GdefInfo)
    }
}

pub struct Cursor<'a>(&'a [u8]);
impl<'a> Cursor<'a> {
    fn read_at<T: Scalar>(&self, offset: usize) -> Option<T> {
        None
    }
}

struct AttachList;

struct AttachListInfo {
    //attach_point_offsets_end: usize,
}

impl AttachListInfo {
    const fn coverage_offset(&self) -> usize {
        0
    }

    const fn glyph_count_offset(&self) -> usize {
        self.coverage_offset() + std::mem::size_of::<BigEndian<Offset16>>()
    }

    fn attach_point_offsets_offset(&self) -> usize {
        self.glyph_count_offset() + std::mem::size_of::<BigEndian<u16>>()
    }
}

impl TableInfo for AttachList {
    type Info = GdefInfo;
    fn from_bytes(bytes: &[u8]) -> Option<Self::Info> {
        let mut cursor = Cursor(bytes);
        let mut shape = AttachListInfo::default();
        let count = cursor.read_at::<u16>(shape.glyph_count_offset())?;
        let len = (count as usize) * std::mem::size_of::<BigEndian<Offset16>>();
    }
}

struct AttachPoint;
struct AttachPointInfo;

// kind of unrelated:
mod take2 {
    use font_types::FontRead;

    use crate::tables::gdef::AttachList;

    trait ObjectMarker<'a> {
        type Marked: FontRead<'a>;
    }

    struct AttachListMarker;

    impl<'a> ObjectMarker<'a> for AttachListMarker {
        type Marked = AttachList<'a>;
    }

    struct Offset16<T> {
        offset: u16,
        typ: std::marker::PhantomData<T>,
    }

    impl<'a, T: ObjectMarker<'a>> Offset16<T> {
        fn resolve(&self, from_data: &'a [u8]) -> Option<T::Marked> {
            from_data
                .get(self.offset as usize..)
                .and_then(T::Marked::read)
        }
    }
}
