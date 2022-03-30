/// A trait for font scalars.
///
/// This is an internal trait for encoding and decoding big-endian bytes.
///
/// You do not need to implement this trait directly; it is an implemention
/// detail of the [`BigEndian`] wrapper.
pub trait Scalar {
    /// The raw byte representation of this type.
    type Raw: Copy + zerocopy::Unaligned + zerocopy::FromBytes + zerocopy::AsBytes;

    /// The size of the raw type. Essentially an alias for `std::mem::size_of`.
    //TODO: remove this probably
    const SIZE: usize = std::mem::size_of::<Self::Raw>();

    /// Create an instance of this type from raw big-endian bytes
    fn from_raw(raw: Self::Raw) -> Self;
    /// Encode this type as raw big-endian bytes
    fn to_raw(self) -> Self::Raw;
}

/// A wrapper around raw big-endian bytes for some type.
#[derive(Clone, Copy, zerocopy::Unaligned, zerocopy::FromBytes)]
#[repr(transparent)]
pub struct BigEndian<T: Scalar>(T::Raw);

/// SAFETEY:
///
/// this trait requires that structs contain no padding.
///
/// 1. BigEndian is repr(transparent) so will always have same layout as inner type
/// 2. the inner type is always a fixed-size array, with no padding
/// 3. so this type can never have padding
/// 4. if my logic is wrong, it is because point 2 makes an invalid assumption :)))
unsafe impl<T: Scalar> zerocopy::AsBytes for BigEndian<T> {
    fn only_derive_is_allowed_to_implement_this_trait()
    where
        Self: Sized,
    {
    }
}

impl<T: Scalar> BigEndian<T> {
    /// Read a copy of this type from raw bytes.
    pub fn get(self) -> T {
        T::from_raw(self.0)
    }

    /// Set the value, overwriting the bytes.
    pub fn set(&mut self, value: T) {
        self.0 = value.to_raw();
    }
}

/// An internal macro for implementing the `RawType` trait.
#[macro_export]
macro_rules! newtype_scalar {
    ($name:ident, $raw:ty) => {
        impl crate::raw::Scalar for $name {
            type Raw = $raw;
            fn to_raw(self) -> $raw {
                self.0.to_raw()
            }

            fn from_raw(raw: $raw) -> Self {
                Self(crate::raw::Scalar::from_raw(raw))
            }
        }
    };
}

macro_rules! int_scalar {
    ($ident:ty) => {
        impl crate::raw::Scalar for $ident {
            type Raw = $ident;
            fn to_raw(self) -> $ident {
                self
            }
            fn from_raw(raw: $ident) -> $ident {
                raw
            }
        }
    };
    ($ty:ty, $raw:ty) => {
        impl crate::raw::Scalar for $ty {
            type Raw = $raw;
            fn to_raw(self) -> $raw {
                self.to_be_bytes()
            }

            fn from_raw(raw: $raw) -> $ty {
                Self::from_be_bytes(raw)
            }
        }
    };
}

int_scalar!(u8, [u8; 1]);
int_scalar!(i8, [u8; 1]);
int_scalar!(u16, [u8; 2]);
int_scalar!(i16, [u8; 2]);
int_scalar!(u32, [u8; 4]);
int_scalar!(i32, [u8; 4]);
int_scalar!(i64, [u8; 8]);
int_scalar!(crate::Uint24, [u8; 3]);

impl<T: Scalar> From<T> for BigEndian<T> {
    fn from(val: T) -> Self {
        BigEndian(val.to_raw())
    }
}

impl<T: std::fmt::Debug + Scalar + Copy> std::fmt::Debug for BigEndian<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.get().fmt(f)
    }
}

impl<T: std::fmt::Display + Scalar + Copy> std::fmt::Display for BigEndian<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.get().fmt(f)
    }
}
