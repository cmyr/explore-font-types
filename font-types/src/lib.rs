//! Common [scalar data types][data types] used in font files
//!
//! [data types]: https://docs.microsoft.com/en-us/typography/opentype/spec/otff#data-types

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(any(feature = "std", test))]
#[macro_use]
extern crate std;

#[cfg(all(not(feature = "std"), not(test)))]
#[macro_use]
extern crate core as std;

mod fixed;
mod fword;
mod longdatetime;
mod offset;
mod raw;
mod tag;
mod uint24;
mod var_array;
mod version;

#[doc(hidden)]
pub mod test_helpers;

pub use font_types_macro::tables;

pub use fixed::{F2Dot14, Fixed};
pub use fword::{FWord, UfWord};
pub use longdatetime::LongDateTime;
pub use offset::{Offset, Offset16, Offset24, Offset32, OffsetHost};
pub use offset::{OffsetData, OffsetHost2, OffsetHost2Mut};
pub use raw::{BigEndian, Scalar};
pub use tag::Tag;
pub use uint24::Uint24;
pub use var_array::VarArray;
pub use version::{MajorMinor, Version16Dot16};

/// A type that can be read from some chunk of bytes.
pub trait FontRead<B: zerocopy::ByteSlice>: Sized {
    /// attempt to read self from raw bytes.
    ///
    /// `bytes` may contain 'extra' bytes; the implemention should ignore them.
    fn read(bytes: B) -> Option<Self>;
}

//HACK: I'm not sure how this should work
/// A trait for types with variable length.
///
/// Currently we implement this by hand where necessary; it is only necessary
/// if these types occur in an array?
#[allow(clippy::len_without_is_empty)]
pub trait VarSized<B: zerocopy::ByteSlice>: FontRead<B> {
    fn len(&self) -> usize;
}

impl<B: zerocopy::ByteSlice, T: zerocopy::FromBytes + zerocopy::Unaligned> FontRead<B> for T {
    fn read(bytes: B) -> Option<Self> {
        T::read_from_prefix(bytes)
    }
    //fn read(bytes: &'a [u8]) -> Option<Self> {
    //T::read_from_prefix(bytes)
    //}
}
