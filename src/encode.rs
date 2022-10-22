//! CBOR encoder.
use crate::{error::NumberOutOfRange, RawCborCodec};
use byteorder::{BigEndian, ByteOrder};
use libipld_core::{cid::Cid, codec::Encode, error::Result, ipld::Ipld};
use std::{collections::BTreeMap, io::Write, ops::Deref, sync::Arc};

/// Writes a null byte to a cbor encoded byte stream.
pub fn write_null<W: Write>(w: &mut W) -> Result<()> {
    w.write_all(&[0xf6])?;
    Ok(())
}

/// Writes a u8 to a cbor encoded byte stream.
pub fn write_u8<W: Write>(w: &mut W, major: u8, value: u8) -> Result<()> {
    if value <= 0x17 {
        let buf = [major << 5 | value];
        w.write_all(&buf)?;
    } else {
        let buf = [major << 5 | 24, value];
        w.write_all(&buf)?;
    }
    Ok(())
}

/// Writes a u16 to a cbor encoded byte stream.
pub fn write_u16<W: Write>(w: &mut W, major: u8, value: u16) -> Result<()> {
    if value <= u16::from(u8::max_value()) {
        write_u8(w, major, value as u8)?;
    } else {
        let mut buf = [major << 5 | 25, 0, 0];
        BigEndian::write_u16(&mut buf[1..], value);
        w.write_all(&buf)?;
    }
    Ok(())
}

/// Writes a u32 to a cbor encoded byte stream.
pub fn write_u32<W: Write>(w: &mut W, major: u8, value: u32) -> Result<()> {
    if value <= u32::from(u16::max_value()) {
        write_u16(w, major, value as u16)?;
    } else {
        let mut buf = [major << 5 | 26, 0, 0, 0, 0];
        BigEndian::write_u32(&mut buf[1..], value);
        w.write_all(&buf)?;
    }
    Ok(())
}

/// Writes a u64 to a cbor encoded byte stream.
pub fn write_u64<W: Write>(w: &mut W, major: u8, value: u64) -> Result<()> {
    if value <= u64::from(u32::max_value()) {
        write_u32(w, major, value as u32)?;
    } else {
        let mut buf = [major << 5 | 27, 0, 0, 0, 0, 0, 0, 0, 0];
        BigEndian::write_u64(&mut buf[1..], value);
        w.write_all(&buf)?;
    }
    Ok(())
}

/// Writes a tag to a cbor encoded byte stream.
pub fn write_tag<W: Write>(w: &mut W, tag: u64) -> Result<()> {
    write_u64(w, 6, tag)
}

impl Encode<RawCborCodec> for bool {
    fn encode<W: Write>(&self, _: RawCborCodec, w: &mut W) -> Result<()> {
        let buf = if *self { [0xf5] } else { [0xf4] };
        w.write_all(&buf)?;
        Ok(())
    }
}

impl Encode<RawCborCodec> for u8 {
    fn encode<W: Write>(&self, _: RawCborCodec, w: &mut W) -> Result<()> {
        write_u8(w, 0, *self)
    }
}

impl Encode<RawCborCodec> for u16 {
    fn encode<W: Write>(&self, _: RawCborCodec, w: &mut W) -> Result<()> {
        write_u16(w, 0, *self)
    }
}

impl Encode<RawCborCodec> for u32 {
    fn encode<W: Write>(&self, _: RawCborCodec, w: &mut W) -> Result<()> {
        write_u32(w, 0, *self)
    }
}

impl Encode<RawCborCodec> for u64 {
    fn encode<W: Write>(&self, _: RawCborCodec, w: &mut W) -> Result<()> {
        write_u64(w, 0, *self)
    }
}

impl Encode<RawCborCodec> for i8 {
    fn encode<W: Write>(&self, _: RawCborCodec, w: &mut W) -> Result<()> {
        write_u8(w, 1, -(*self + 1) as u8)
    }
}

impl Encode<RawCborCodec> for i16 {
    fn encode<W: Write>(&self, _: RawCborCodec, w: &mut W) -> Result<()> {
        write_u16(w, 1, -(*self + 1) as u16)
    }
}

impl Encode<RawCborCodec> for i32 {
    fn encode<W: Write>(&self, _: RawCborCodec, w: &mut W) -> Result<()> {
        write_u32(w, 1, -(*self + 1) as u32)
    }
}

impl Encode<RawCborCodec> for i64 {
    fn encode<W: Write>(&self, _: RawCborCodec, w: &mut W) -> Result<()> {
        write_u64(w, 1, -(*self + 1) as u64)
    }
}

impl Encode<RawCborCodec> for f32 {
    #[allow(clippy::float_cmp)]
    fn encode<W: Write>(&self, _: RawCborCodec, w: &mut W) -> Result<()> {
        if self.is_infinite() {
            if self.is_sign_positive() {
                w.write_all(&[0xf9, 0x7c, 0x00])?;
            } else {
                w.write_all(&[0xf9, 0xfc, 0x00])?;
            }
        } else if self.is_nan() {
            w.write_all(&[0xf9, 0x7e, 0x00])?;
        } else {
            let mut buf = [0xfa, 0, 0, 0, 0];
            BigEndian::write_f32(&mut buf[1..], *self);
            w.write_all(&buf)?;
        }
        Ok(())
    }
}

impl Encode<RawCborCodec> for f64 {
    #[allow(clippy::float_cmp)]
    fn encode<W: Write>(&self, c: RawCborCodec, w: &mut W) -> Result<()> {
        if !self.is_finite() || f64::from(*self as f32) == *self {
            let value = *self as f32;
            value.encode(c, w)?;
        } else {
            let mut buf = [0xfb, 0, 0, 0, 0, 0, 0, 0, 0];
            BigEndian::write_f64(&mut buf[1..], *self);
            w.write_all(&buf)?;
        }
        Ok(())
    }
}

impl Encode<RawCborCodec> for [u8] {
    fn encode<W: Write>(&self, _: RawCborCodec, w: &mut W) -> Result<()> {
        write_u64(w, 2, self.len() as u64)?;
        w.write_all(self)?;
        Ok(())
    }
}

impl Encode<RawCborCodec> for Box<[u8]> {
    fn encode<W: Write>(&self, c: RawCborCodec, w: &mut W) -> Result<()> {
        self[..].encode(c, w)
    }
}

impl Encode<RawCborCodec> for str {
    fn encode<W: Write>(&self, _: RawCborCodec, w: &mut W) -> Result<()> {
        write_u64(w, 3, self.len() as u64)?;
        w.write_all(self.as_bytes())?;
        Ok(())
    }
}

impl Encode<RawCborCodec> for String {
    fn encode<W: Write>(&self, c: RawCborCodec, w: &mut W) -> Result<()> {
        self.as_str().encode(c, w)
    }
}

impl Encode<RawCborCodec> for i128 {
    fn encode<W: Write>(&self, _: RawCborCodec, w: &mut W) -> Result<()> {
        if *self < 0 {
            if -(*self + 1) > u64::max_value() as i128 {
                return Err(NumberOutOfRange::new::<i128>().into());
            }
            write_u64(w, 1, -(*self + 1) as u64)?;
        } else {
            if *self > u64::max_value() as i128 {
                return Err(NumberOutOfRange::new::<i128>().into());
            }
            write_u64(w, 0, *self as u64)?;
        }
        Ok(())
    }
}

impl Encode<RawCborCodec> for Cid {
    fn encode<W: Write>(&self, _: RawCborCodec, w: &mut W) -> Result<()> {
        write_tag(w, 42)?;
        // insert zero byte per https://github.com/ipld/specs/blob/master/block-layer/codecs/dag-cbor.md#links
        // TODO: don't allocate
        let buf = self.to_bytes();
        let len = buf.len();
        write_u64(w, 2, len as u64 + 1)?;
        w.write_all(&[0])?;
        w.write_all(&buf[..len])?;
        Ok(())
    }
}

impl<T: Encode<RawCborCodec>> Encode<RawCborCodec> for Option<T> {
    fn encode<W: Write>(&self, c: RawCborCodec, w: &mut W) -> Result<()> {
        if let Some(value) = self {
            value.encode(c, w)?;
        } else {
            write_null(w)?;
        }
        Ok(())
    }
}

impl<T: Encode<RawCborCodec>> Encode<RawCborCodec> for Vec<T> {
    fn encode<W: Write>(&self, c: RawCborCodec, w: &mut W) -> Result<()> {
        write_u64(w, 4, self.len() as u64)?;
        for value in self {
            value.encode(c, w)?;
        }
        Ok(())
    }
}

impl<K: Encode<RawCborCodec>, T: Encode<RawCborCodec> + 'static> Encode<RawCborCodec> for BTreeMap<K, T> {
    fn encode<W: Write>(&self, c: RawCborCodec, w: &mut W) -> Result<()> {
        write_u64(w, 5, self.len() as u64)?;
        for (k, v) in self {
            k.encode(c, w)?;
            v.encode(c, w)?;
        }
        Ok(())
    }
}

impl Encode<RawCborCodec> for Ipld {
    fn encode<W: Write>(&self, c: RawCborCodec, w: &mut W) -> Result<()> {
        match self {
            Self::Null => write_null(w),
            Self::Bool(b) => b.encode(c, w),
            Self::Integer(i) => i.encode(c, w),
            Self::Float(f) => f.encode(c, w),
            Self::Bytes(b) => b.as_slice().encode(c, w),
            Self::String(s) => s.encode(c, w),
            Self::List(l) => l.encode(c, w),
            Self::Map(m) => m.encode(c, w),
            Self::Link(cid) => cid.encode(c, w),
        }
    }
}

impl<T: Encode<RawCborCodec>> Encode<RawCborCodec> for Arc<T> {
    fn encode<W: Write>(&self, c: RawCborCodec, w: &mut W) -> Result<()> {
        self.deref().encode(c, w)
    }
}

impl Encode<RawCborCodec> for () {
    fn encode<W: Write>(&self, _c: RawCborCodec, w: &mut W) -> Result<()> {
        write_u8(w, 4, 0)?;
        Ok(())
    }
}

impl<A: Encode<RawCborCodec>> Encode<RawCborCodec> for (A,) {
    fn encode<W: Write>(&self, c: RawCborCodec, w: &mut W) -> Result<()> {
        write_u8(w, 4, 1)?;
        self.0.encode(c, w)?;
        Ok(())
    }
}

impl<A: Encode<RawCborCodec>, B: Encode<RawCborCodec>> Encode<RawCborCodec> for (A, B) {
    fn encode<W: Write>(&self, c: RawCborCodec, w: &mut W) -> Result<()> {
        write_u8(w, 4, 2)?;
        self.0.encode(c, w)?;
        self.1.encode(c, w)?;
        Ok(())
    }
}

impl<A: Encode<RawCborCodec>, B: Encode<RawCborCodec>, C: Encode<RawCborCodec>> Encode<RawCborCodec>
    for (A, B, C)
{
    fn encode<W: Write>(&self, c: RawCborCodec, w: &mut W) -> Result<()> {
        write_u8(w, 4, 3)?;
        self.0.encode(c, w)?;
        self.1.encode(c, w)?;
        self.2.encode(c, w)?;
        Ok(())
    }
}

impl<A: Encode<RawCborCodec>, B: Encode<RawCborCodec>, C: Encode<RawCborCodec>, D: Encode<RawCborCodec>>
    Encode<RawCborCodec> for (A, B, C, D)
{
    fn encode<W: Write>(&self, c: RawCborCodec, w: &mut W) -> Result<()> {
        write_u8(w, 4, 4)?;
        self.0.encode(c, w)?;
        self.1.encode(c, w)?;
        self.2.encode(c, w)?;
        self.3.encode(c, w)?;
        Ok(())
    }
}
