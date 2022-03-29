/// An array with members of different sizes
pub struct VarArray<B, T> {
    #[allow(dead_code)]
    bytes: B,
    _t: std::marker::PhantomData<T>,
}

//impl<'a, T> VarArray<'a, T> {}

impl<B: zerocopy::ByteSlice, T: super::VarSized<B>> VarArray<B, T> {
    pub fn new(bytes: B) -> Self {
        Self {
            bytes,
            _t: std::marker::PhantomData,
        }
    }

    pub fn get(&self, _idx: usize) -> Option<T> {
        //let mut offset = 0;

        unimplemented!("intending to delete this shortly")
        //for _ in 0..idx {
        //let nxt = self.bytes.get(offset..).and_then(T::read)?;
        //offset += nxt.len();
        //}
        //self.bytes.get(offset..).and_then(T::read)
    }

    //pub fn iter(&self) -> impl Iterator<Item = T> {
    //todo!("intending to delete this shortly")
    //let mut offset = 0;
    //let bytes = self.bytes;
    //std::iter::from_fn(move || {
    ////let blob = blob.get(offset..blob.len())?;
    //let nxt = bytes.get(offset..).and_then(T::read)?;
    //offset += nxt.len();
    //Some(nxt)
    //})
    //}
}
