//! compile-time representations of offsets

use font_types::Offset;

use crate::subset::Subset;

use super::FontWrite;

/// An offset subtable.
pub struct OffsetMarker<W, T> {
    width: std::marker::PhantomData<W>,
    obj: Option<T>,
}

/// An offset subtable which may be null.
pub struct NullableOffsetMarker<W, T> {
    width: std::marker::PhantomData<W>,
    obj: Option<T>,
}

impl<W, T> OffsetMarker<W, T> {
    //TODO: how do we handle malformed inputs? do we error earlier than this?
    /// Get the object. Fonts in the wild may be malformed, so this still returns
    /// an option?
    pub fn get(&self) -> Option<&T> {
        self.obj.as_ref()
    }

    pub fn get_mut(&mut self) -> Option<&mut T> {
        self.obj.as_mut()
    }

    pub fn set(&mut self, obj: T) {
        self.obj = Some(obj);
    }

    pub fn clear(&mut self) {
        self.obj = None;
    }
}

impl<W, T: Subset> Subset for OffsetMarker<W, T> {
    fn subset(&mut self, plan: &crate::subset::Plan) -> Result<bool, crate::subset::Error> {
        let retain = self
            .get_mut()
            .map(|t| t.subset(plan))
            .transpose()?
            .unwrap_or(false);
        if !retain {
            self.clear();
        }
        Ok(retain)
    }
}

impl<W: Offset, T> OffsetMarker<W, T> {
    /// Create a new marker.
    pub fn new(obj: T) -> Self {
        OffsetMarker {
            width: std::marker::PhantomData,
            obj: Some(obj),
        }
    }

    /// Creates a new marker with an object that may be null.
    //TODO: figure out how we're actually handling null offsets. Some offsets
    //are allowed to be null, but even offsets that aren't *may* be null,
    //and we should handle this.
    pub fn new_maybe_null(obj: Option<T>) -> Self {
        OffsetMarker {
            width: std::marker::PhantomData,
            obj,
        }
    }
}

impl<W, T> NullableOffsetMarker<W, T> {
    //TODO: how do we handle malformed inputs? do we error earlier than this?
    /// Get the object, if it exists.
    pub fn get(&self) -> Option<&T> {
        self.obj.as_ref()
    }
}

impl<W: Offset, T> NullableOffsetMarker<W, T> {
    pub fn new(obj: Option<T>) -> Self {
        NullableOffsetMarker {
            width: std::marker::PhantomData,
            obj,
        }
    }
}

impl<W: Offset, T: FontWrite> FontWrite for OffsetMarker<W, T> {
    fn write_into(&self, writer: &mut super::TableWriter) {
        match self.obj.as_ref() {
            Some(obj) => writer.write_offset::<W>(obj),
            None => {
                eprintln!("warning: unexpected null OffsetMarker");
                writer.write_slice(W::SIZE.null_bytes());
            }
        }
    }
}

impl<W: Offset, T: FontWrite> FontWrite for NullableOffsetMarker<W, T> {
    fn write_into(&self, writer: &mut super::TableWriter) {
        match self.obj.as_ref() {
            Some(obj) => writer.write_offset::<W>(obj),
            None => writer.write_slice(W::SIZE.null_bytes()),
        }
    }
}

impl<W, T> Default for OffsetMarker<W, T> {
    fn default() -> Self {
        OffsetMarker {
            width: std::marker::PhantomData,
            obj: None,
        }
    }
}

impl<W, T> Default for NullableOffsetMarker<W, T> {
    fn default() -> Self {
        NullableOffsetMarker {
            width: std::marker::PhantomData,
            obj: None,
        }
    }
}

impl<W, T: PartialEq> PartialEq for OffsetMarker<W, T> {
    fn eq(&self, other: &Self) -> bool {
        self.obj == other.obj
    }
}

impl<W, T: PartialEq> PartialEq for NullableOffsetMarker<W, T> {
    fn eq(&self, other: &Self) -> bool {
        self.obj == other.obj
    }
}

impl<W: Offset, T: std::fmt::Debug> std::fmt::Debug for OffsetMarker<W, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "OffsetMarker({}, {:?})", W::SIZE, self.obj.as_ref(),)
    }
}

impl<W: Offset, T: std::fmt::Debug> std::fmt::Debug for NullableOffsetMarker<W, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "NullableOffsetMarker({}, {:?})",
            W::SIZE,
            self.obj.as_ref(),
        )
    }
}
