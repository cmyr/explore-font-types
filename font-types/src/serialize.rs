use crate::{BigEndian, FontRead, Offset, Scalar};

//pub trait FontSerialize<'se>: FontRead<'se> {
//fn serialize<S: Serializer<'se>>(&self, serializer: &mut S);
//}

//pub trait Serializer<'se>: Sized {
//fn write_be_bytes(&mut self, bytes: &[u8]);
//fn resolve_offset<T: FontSerialize<'se>, O: Offset>(&self, offset: O);
//fn write_offset<O: Offset, T: FontSerialize<'se>>(&mut self, obj: &T);

//fn write_record(&mut self, record: &impl FontSerialize<'se>) {
//record.serialize(self);
//}
//}

pub trait Serialize2 {
    fn serialize(&self, serializer: &mut impl Serializer2, offset_bytes: &[u8]);
}

//pub trait ObjectStore {
//fn add_object<T: Serialize2>(&mut self, obj: &T) -> ObjectId;
//}

pub trait Serializer2 {
    fn write_be_bytes(&mut self, bytes: &[u8]);
    fn write_offset<O: Offset, T: Serialize2>(&mut self, obj: &T);
    fn write_offset_maybe_null<O: Offset, T: Serialize2>(&mut self, obj: Option<&T>) {
        match obj {
            Some(obj) => self.write_offset::<O, _>(obj),
            None => self.write_be_bytes(O::SIZE.null_bytes()),
        }
    }
}

impl<T: Scalar> Serialize2 for BigEndian<T> {
    fn serialize(&self, serializer: &mut impl Serializer2, _offset_bytes: &[u8]) {
        serializer.write_be_bytes(self.0.as_ref())
    }
}

//NOTE: this would be a fun approach, but it's only going to work if offsets know their type.
//macro_rules! serialize_scalar {
//($ty:ty) => {
//impl crate::serialize::Serialize2 for BigEndian<$ty> {
//fn serialize(&self, serializer: &mut impl Serializer2, _: &[u8]) {
//serializer.write_be_bytes(self.0.as_ref())
//}
//}
//};
//}

//macro_rules! serialize_offset {
//($ty:ty) => {
//impl crate::serialize::Serialize2 for BigEndian<$ty> {
//fn serialize(&self, serializer: &mut impl Serializer2, offset_bytes: &[u8]) {
//match self.get().read(offset_bytes) {
//Some(table) => serializer.write_offset::<$ty, _>(&table),
//None => serializer.write_be_bytes(<$ty as Offset>::SIZE.null_bytes()),

//}
//}
//}
//};
//}

//serialize_offset!(crate::Offset16);
