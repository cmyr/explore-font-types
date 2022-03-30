//! Offsets to tables

use std::ops::RangeBounds;

use crate::Uint24;

/// A trait for the different offset representations.
pub trait Offset {
    /// Returns this offsize as a `usize`, or `None` if it is `0`.
    fn non_null(self) -> Option<usize>;
}

pub struct OffsetData<B> {
    data: B,
    /// The start offset of the data relative to the start of the containing table.
    ///
    /// Offsets are often calculated from the start of a table, but table fields are
    /// not included in the data here; this tracks the length of those fields,
    /// so that we can determine the correct offset positions.
    start_offset: usize,
}

impl<B: zerocopy::ByteSlice> OffsetData<B> {
    pub fn new(data: B, start_offset: usize) -> Self {
        Self { data, start_offset }
    }

    /// Return the bytes for a given offset
    pub fn bytes_at_offset(&self, offset: impl Offset) -> Option<&[u8]> {
        offset
            .non_null()
            //FIXME: this could underflow, which is useful in debugging but needs more thought
            .and_then(|off| self.data.get(off - self.start_offset..))
        //.unwrap_or_default()
    }

    #[inline]
    pub fn get(&self, index: impl RangeBounds<usize>) -> Option<&[u8]> {
        let index = resolve_range(index, self.start_offset, self.data.len());
        self.data.get(index)
    }
}

impl<B: zerocopy::ByteSliceMut> OffsetData<B> {
    /// Return the mutable bytes for a given offset
    pub fn bytes_at_offset_mut(&mut self, offset: impl Offset) -> Option<&mut [u8]> {
        offset
            .non_null()
            .and_then(|off| self.data.get_mut(off - self.start_offset..))
        //.unwrap_or_default()
    }

    #[inline]
    pub fn get_mut(&mut self, index: impl RangeBounds<usize>) -> Option<&mut [u8]> {
        let index = resolve_range(index, self.start_offset, self.data.len());
        self.data.get_mut(index)
    }
}

pub trait OffsetHost2<'a, B: zerocopy::ByteSlice + 'a> {
    fn data(&self) -> &OffsetData<B>;

    fn resolve_offset<T: crate::FontRead<&'a [u8]> + 'a>(
        &'a self,
        offset: impl Offset,
    ) -> Option<T> {
        self.data()
            .bytes_at_offset(offset)
            .and_then(crate::FontRead::read)
    }
}

pub trait OffsetHost2Mut<'a, B: zerocopy::ByteSliceMut + 'a> {
    fn data_mut(&mut self) -> &mut OffsetData<B>;

    fn resolve_offset<T: crate::FontRead<&'a mut [u8]> + 'a>(
        &'a mut self,
        offset: impl Offset,
    ) -> Option<T> {
        self.data_mut()
            .bytes_at_offset_mut(offset)
            .and_then(crate::FontRead::read)
    }
}

/// A type that contains data referenced by offsets.
pub trait OffsetHost<'a, B: zerocopy::ByteSlice + 'a> {
    /// Return a slice of bytes from which offsets may be resolved.
    ///
    /// This should be relative to the start of the host.
    fn bytes(&self) -> &B;

    fn bytes_mut(&mut self) -> &mut B;

    /// Return the bytes for a given offset
    fn bytes_at_offset(&'a self, offset: impl Offset) -> &'a [u8] {
        offset
            .non_null()
            .and_then(|off| self.bytes().get(off..))
            .unwrap_or_default()
    }

    fn resolve_offset<T: crate::FontRead<&'a [u8]>>(&'a self, offset: impl Offset) -> Option<T> {
        crate::FontRead::read(self.bytes_at_offset(offset))
    }
}

pub trait OffsetHostMut<'a, B: zerocopy::ByteSliceMut + 'a>: OffsetHost<'a, B> {
    fn bytes_at_offset_mut(&'a mut self, offset: impl Offset) -> &'a mut [u8] {
        offset
            .non_null()
            .and_then(|off| self.bytes_mut().get_mut(off..))
            .unwrap_or_default()
    }

    fn resolve_offset_mut<T: crate::FontRead<&'a mut [u8]>>(
        &'a mut self,
        offset: impl Offset,
    ) -> Option<T> {
        crate::FontRead::read(self.bytes_at_offset_mut(offset))
    }
}

macro_rules! impl_offset {
    ($name:ident, $bits:literal, $rawty:ty) => {
        #[doc = concat!("A", stringify!($bits), "-bit offset to a table.")]
        ///
        /// Specific offset fields may or may not permit NULL values; however we
        /// assume that errors are possible, and expect the caller to handle
        /// the `None` case.
        #[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
        pub struct $name($rawty);

        impl $name {
            /// Create a new offset.
            pub fn new(raw: $rawty) -> Self {
                Self(raw)
            }
        }

        impl crate::raw::Scalar for $name {
            type Raw = <$rawty as crate::raw::Scalar>::Raw;
            fn from_raw(raw: Self::Raw) -> Self {
                let raw = <$rawty>::from_raw(raw);
                $name::new(raw)
            }

            fn to_raw(self) -> Self::Raw {
                self.0.to_raw()
            }
        }

        impl Offset for $name {
            fn non_null(self) -> Option<usize> {
                let raw: u32 = self.0.into();
                if raw == 0 {
                    None
                } else {
                    Some(raw as usize)
                }
            }
        }
    };
}

impl_offset!(Offset16, 16, u16);
impl_offset!(Offset24, 24, Uint24);
impl_offset!(Offset32, 32, u32);

#[inline]
fn resolve_range(
    index: impl RangeBounds<usize>,
    start: usize,
    len: usize,
) -> std::ops::Range<usize> {
    let ix_start = match index.start_bound() {
        std::ops::Bound::Unbounded => 0,
        std::ops::Bound::Included(i) => i.saturating_sub(start),
        _ => unreachable!(),
    };

    let ix_end = match index.end_bound() {
        std::ops::Bound::Unbounded => len.max(ix_start),
        std::ops::Bound::Excluded(i) => i.saturating_sub(start),
        std::ops::Bound::Included(i) => i.saturating_sub(start) + 1,
    };
    ix_start..ix_end
}
