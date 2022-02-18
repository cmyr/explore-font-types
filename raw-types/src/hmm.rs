use zerocopy::{ByteSlice, LayoutVerified};

use crate::Uint16;


//struct ChainedSequenceRule {
    ///// Number of glyphs in the backtrack sequence
    //backtrack_glyph_count: Uint16,
    ///// Array of backtrack glyph IDs
    //#[count(backtrack_glyph_count)]
    //backtrack_sequence: [Uint16],
    ///// Number of glyphs in the input sequence
    //input_glyph_count: Uint16,
    ///// - 1]    Array of input glyph IDs—start with second glyph
    //#[count(input_glyph_count)]
    //input_sequence: [Uint16],
//}

//fn init(bytes: &[u8]) -> Option<()> {
    //let backtrack_glyph_count = bytes.get(0..2).map(Uint16::from_bytes)?;
    //let backtrack_sequence_len = backtrack_glyph_count * std::mem::size_of::<Uint16>();
    //let input_glyph_count = bytes
        //.get(2 + backtrack_sequence_len..2 + backtrack_sequence_len + 2)
        //.map(Uint16::from_bytes)?;
//}

//impl ChainedSequenceRule {
    //fn backtrack_sequence(&self) -> &[Uint16] {
        //let pos = std::mem::size_of::<Uint16>();
        //let len = self.backtrack_glyph_count() * std::mem::size_of::<Uint16>();
        //self.make_slice(pos..pos + len)
    //}

    //fn input_glyph_count(&self) -> Uint16 {
        //let pos = std::mem::size_of::<Uint16>()
            //+ self.backtrack_glyph_count() * std::mem::size_of::<Uint16>();
        //self.0.read(pos)
    //}

    //fn input_sequence(&self) -> &[Uint16] {
        //let pos = std::mem::size_of::<Uint16>()
            //+ self.backtrack_glyph_count() * std::mem::size_of::<Uint16>();
    //}
//}

struct ZcChainedSequence<B> {
    backtrack_sequence: LayoutVerified<B, [Uint16]>,
    input_sequence: LayoutVerified<B, [Uint16]>,
}

impl<B: ByteSlice> ZcChainedSequence<B> {
    fn new(bytes: B) -> Option<Self> {
        let (backtrack_count, bytes) = LayoutVerified::<_, Uint16>::new_unaligned_from_prefix(bytes)?;
        let (backtrack_sequence, bytes) = LayoutVerified::<_, [Uint16]>::new_slice_unaligned_from_prefix(bytes, backtrack_count.get() as usize)?;
        let (input_count, bytes) = LayoutVerified::<_, Uint16>::new_unaligned_from_prefix(bytes)?;
        let (input_sequence, bytes) = LayoutVerified::<_, [Uint16]>::new_slice_unaligned_from_prefix(bytes, backtrack_count.get() as usize)?;
        //let (input_count, bytes) = LayoutVerified::<_, Uint16>::new_unaligned_from_prefix(bytes)?;


    }
}
