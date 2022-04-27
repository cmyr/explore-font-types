use std::rc::Rc;

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
