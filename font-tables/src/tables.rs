//! Font tables.

pub mod cmap;
pub mod glyf;
pub mod head;
pub mod hhea;
pub mod hmtx;
pub mod loca;
pub mod maxp;
pub mod name;
pub mod post;
pub mod stat;

use font_types::{FontRead, Tag};
use zerocopy::LayoutVerified;

/// An interface for accessing tables from a font (or font-like object)
pub trait TableProvider {
    fn data_for_tag(&self, tag: Tag) -> Option<&[u8]>;

    fn head(&self) -> Option<head::Head> {
        self.data_for_tag(Tag::new(b"head"))
            .and_then(head::Head::read)
    }

    fn name(&self) -> Option<name::Name<&[u8]>> {
        self.data_for_tag(name::TAG).and_then(name::Name::read)
    }

    fn hhea(&self) -> Option<hhea::Hhea> {
        self.data_for_tag(hhea::TAG).and_then(hhea::Hhea::read)
    }

    fn hmtx(&self) -> Option<hmtx::Hmtx<&[u8]>> {
        //FIXME: should we make the user pass these in?
        let num_glyphs = self.maxp().map(|maxp| maxp.num_glyphs())?;
        let number_of_h_metrics = self.hhea().map(|hhea| hhea.number_of_h_metrics())?;
        self.data_for_tag(hmtx::TAG).and_then(|data| {
            hmtx::Hmtx::read(data, num_glyphs as usize, number_of_h_metrics as usize)
        })
    }

    fn maxp(&self) -> Option<maxp::Maxp> {
        self.data_for_tag(maxp::TAG).and_then(maxp::Maxp::read)
    }

    fn post(&self) -> Option<post::Post<&[u8]>> {
        self.data_for_tag(post::TAG).and_then(post::Post::read)
    }

    fn stat(&self) -> Option<stat::Stat<&[u8]>> {
        self.data_for_tag(stat::TAG).and_then(stat::Stat::read)
    }

    fn loca(&self, num_glyphs: u16, is_long: bool) -> Option<loca::Loca> {
        let bytes = self.data_for_tag(loca::TAG)?;
        loca::Loca::read(bytes, num_glyphs, is_long)
    }

    fn glyf(&self) -> Option<glyf::Glyf<&[u8]>> {
        self.data_for_tag(glyf::TAG).and_then(glyf::Glyf::read)
    }

    fn cmap(&self) -> Option<cmap::Cmap<&[u8]>> {
        self.data_for_tag(Tag::new(b"cmap"))
            .and_then(cmap::Cmap::read)
    }
}

pub trait TableProviderMut {
    fn data_for_tag_mut(&mut self, tag: Tag) -> Option<&mut [u8]>;

    fn head(&self) -> Option<&mut head::Head> {
        let data = self.data_for_tag_mut(Tag::new(b"head"))?;
        let table = LayoutVerified::<_, head::Head>::new_unaligned(data)?;
        table.into_mut()
        //.and_then(head::Head::read)
    }

    fn name(&self) -> Option<name::Name<&[u8]>> {
        self.data_for_tag(name::TAG).and_then(name::Name::read)
    }

    fn hhea(&self) -> Option<hhea::Hhea> {
        self.data_for_tag(hhea::TAG).and_then(hhea::Hhea::read)
    }

    fn hmtx(&self) -> Option<hmtx::Hmtx<&[u8]>> {
        //FIXME: should we make the user pass these in?
        let num_glyphs = self.maxp().map(|maxp| maxp.num_glyphs())?;
        let number_of_h_metrics = self.hhea().map(|hhea| hhea.number_of_h_metrics())?;
        self.data_for_tag(hmtx::TAG).and_then(|data| {
            hmtx::Hmtx::read(data, num_glyphs as usize, number_of_h_metrics as usize)
        })
    }

    fn maxp(&self) -> Option<maxp::Maxp> {
        self.data_for_tag(maxp::TAG).and_then(maxp::Maxp::read)
    }

    fn post(&self) -> Option<post::Post<&[u8]>> {
        self.data_for_tag(post::TAG).and_then(post::Post::read)
    }

    fn stat(&self) -> Option<stat::Stat<&[u8]>> {
        self.data_for_tag(stat::TAG).and_then(stat::Stat::read)
    }

    fn loca(&self, num_glyphs: u16, is_long: bool) -> Option<loca::Loca> {
        let bytes = self.data_for_tag(loca::TAG)?;
        loca::Loca::read(bytes, num_glyphs, is_long)
    }

    fn glyf(&self) -> Option<glyf::Glyf<&[u8]>> {
        self.data_for_tag(glyf::TAG).and_then(glyf::Glyf::read)
    }

    fn cmap(&self) -> Option<cmap::Cmap<&[u8]>> {
        self.data_for_tag(Tag::new(b"cmap"))
            .and_then(cmap::Cmap::read)
    }
}
