//! CBOR decoder
use crate::{DagCbor, Error, Result};
use byteorder::{BigEndian, ByteOrder};
use core::convert::TryFrom;
use libipld_core::cid::Cid;
use libipld_core::codec::Decode;
use libipld_core::ipld::Ipld;
use std::collections::BTreeMap;
use std::io::Read;

pub fn read_u8<R: Read>(r: &mut R) -> Result<u8> {
    let mut buf = [0; 1];
    r.read_exact(&mut buf)?;
    Ok(buf[0])
}

pub fn read_u16<R: Read>(r: &mut R) -> Result<u16> {
    let mut buf = [0; 2];
    r.read_exact(&mut buf)?;
    Ok(BigEndian::read_u16(&buf))
}

pub fn read_u32<R: Read>(r: &mut R) -> Result<u32> {
    let mut buf = [0; 4];
    r.read_exact(&mut buf)?;
    Ok(BigEndian::read_u32(&buf))
}

pub fn read_u64<R: Read>(r: &mut R) -> Result<u64> {
    let mut buf = [0; 8];
    r.read_exact(&mut buf)?;
    Ok(BigEndian::read_u64(&buf))
}

pub fn read_f32<R: Read>(r: &mut R) -> Result<f32> {
    let mut buf = [0; 4];
    r.read_exact(&mut buf)?;
    Ok(BigEndian::read_f32(&buf))
}

pub fn read_f64<R: Read>(r: &mut R) -> Result<f64> {
    let mut buf = [0; 8];
    r.read_exact(&mut buf)?;
    Ok(BigEndian::read_f64(&buf))
}

pub fn read_bytes<R: Read>(r: &mut R, len: usize) -> Result<Vec<u8>> {
    let mut buf = vec![0; len];
    r.read_exact(&mut buf)?;
    Ok(buf)
}

pub fn read_str<R: Read>(r: &mut R, len: usize) -> Result<String> {
    let bytes = read_bytes(r, len)?;
    let string = std::str::from_utf8(&bytes)?;
    Ok(string.to_string())
}

pub fn read_key<R: Read>(r: &mut R, key: &str) -> Result<()> {
    let key_bytes = key.as_bytes();
    let bytes = read_bytes(r, key.len() + 1)?;
    if key_bytes == &bytes[1..] {
        Ok(())
    } else {
        Err(Error::UnexpectedKey)
    }
}

pub fn read<R: Read, T: TryReadCbor>(r: &mut R) -> Result<T> {
    let major = crate::decode::read_u8(r)?;
    if let Some(res) = T::try_read_cbor(r, major)? {
        Ok(res)
    } else {
        Err(Error::UnexpectedCode)
    }
}

pub fn read_list<R: Read, T: TryReadCbor>(r: &mut R, len: usize) -> Result<Vec<T>> {
    let mut list: Vec<T> = Vec::with_capacity(len);
    for _ in 0..len {
        list.push(read(r)?);
    }
    Ok(list)
}

pub fn read_map<R: Read, T: TryReadCbor>(r: &mut R, len: usize) -> Result<BTreeMap<String, T>> {
    let mut map: BTreeMap<String, T> = BTreeMap::new();
    for _ in 0..len {
        let key = read(r)?;
        let value = read(r)?;
        map.insert(key, value);
    }
    Ok(map)
}

pub fn read_link<R: Read>(r: &mut R) -> Result<Cid> {
    let tag = read_u8(r)?;
    if tag != 42 {
        return Err(Error::UnknownTag);
    }
    let ty = read_u8(r)?;
    if ty != 0x58 {
        return Err(Error::UnknownTag);
    }
    let len = read_u8(r)?;
    if len == 0 {
        return Err(Error::LengthOutOfRange);
    }
    let bytes = read_bytes(r, len as usize)?;
    if bytes[0] != 0 {
        return Err(Error::InvalidCidPrefix(bytes[0]));
    }

    // skip the first byte per
    // https://github.com/ipld/specs/blob/master/block-layer/codecs/dag-cbor.md#links
    Ok(Cid::try_from(&bytes[1..])?)
}

pub trait TryReadCbor: Sized {
    fn try_read_cbor<R: Read>(r: &mut R, major: u8) -> Result<Option<Self>>;
}

macro_rules! impl_decode {
    ($ty:ident) => {
        impl Decode<DagCbor> for $ty {
            fn decode<R: Read>(r: &mut R) -> Result<Self> {
                read(r)
            }
        }
    };
    ($ty:ident<T>) => {
        impl<T: TryReadCbor> Decode<DagCbor> for $ty<T> {
            fn decode<R: Read>(r: &mut R) -> Result<Self> {
                read(r)
            }
        }
    };
    ($ty:ident<$param:ident, T>) => {
        impl<T: TryReadCbor> Decode<DagCbor> for $ty<$param, T> {
            fn decode<R: Read>(r: &mut R) -> Result<Self> {
                read(r)
            }
        }
    };
    ($ty:ident<[u8]>) => {
        impl Decode<DagCbor> for $ty<[u8]> {
            fn decode<R: Read>(r: &mut R) -> Result<Self> {
                read(r)
            }
        }
    };
}

impl TryReadCbor for bool {
    fn try_read_cbor<R: Read>(_: &mut R, major: u8) -> Result<Option<Self>> {
        match major {
            0xf4 => Ok(Some(false)),
            0xf5 => Ok(Some(true)),
            _ => Ok(None),
        }
    }
}
impl_decode!(bool);

impl TryReadCbor for u8 {
    fn try_read_cbor<R: Read>(r: &mut R, major: u8) -> Result<Option<Self>> {
        match major {
            0x00..=0x17 => Ok(Some(major)),
            0x18 => Ok(Some(read_u8(r)?)),
            _ => Ok(None),
        }
    }
}
impl_decode!(u8);

impl TryReadCbor for u16 {
    fn try_read_cbor<R: Read>(r: &mut R, major: u8) -> Result<Option<Self>> {
        match major {
            0x00..=0x17 => Ok(Some(major as u16)),
            0x18 => Ok(Some(read_u8(r)? as u16)),
            0x19 => Ok(Some(read_u16(r)?)),
            _ => Ok(None),
        }
    }
}
impl_decode!(u16);

impl TryReadCbor for u32 {
    fn try_read_cbor<R: Read>(r: &mut R, major: u8) -> Result<Option<Self>> {
        match major {
            0x00..=0x17 => Ok(Some(major as u32)),
            0x18 => Ok(Some(read_u8(r)? as u32)),
            0x19 => Ok(Some(read_u16(r)? as u32)),
            0x1a => Ok(Some(read_u32(r)?)),
            _ => Ok(None),
        }
    }
}
impl_decode!(u32);

impl TryReadCbor for u64 {
    fn try_read_cbor<R: Read>(r: &mut R, major: u8) -> Result<Option<Self>> {
        match major {
            0x00..=0x17 => Ok(Some(major as u64)),
            0x18 => Ok(Some(read_u8(r)? as u64)),
            0x19 => Ok(Some(read_u16(r)? as u64)),
            0x1a => Ok(Some(read_u32(r)? as u64)),
            0x1b => Ok(Some(read_u64(r)?)),
            _ => Ok(None),
        }
    }
}
impl_decode!(u64);

impl TryReadCbor for i8 {
    fn try_read_cbor<R: Read>(r: &mut R, major: u8) -> Result<Option<Self>> {
        match major {
            0x20..=0x37 => Ok(Some(-1 - (major - 0x20) as i8)),
            0x38 => Ok(Some(-1 - read_u8(r)? as i8)),
            _ => Ok(None),
        }
    }
}
impl_decode!(i8);

impl TryReadCbor for i16 {
    fn try_read_cbor<R: Read>(r: &mut R, major: u8) -> Result<Option<Self>> {
        match major {
            0x20..=0x37 => Ok(Some(-1 - (major - 0x20) as i16)),
            0x38 => Ok(Some(-1 - read_u8(r)? as i16)),
            0x39 => Ok(Some(-1 - read_u16(r)? as i16)),
            _ => Ok(None),
        }
    }
}
impl_decode!(i16);

impl TryReadCbor for i32 {
    fn try_read_cbor<R: Read>(r: &mut R, major: u8) -> Result<Option<Self>> {
        match major {
            0x20..=0x37 => Ok(Some(-1 - (major - 0x20) as i32)),
            0x38 => Ok(Some(-1 - read_u8(r)? as i32)),
            0x39 => Ok(Some(-1 - read_u16(r)? as i32)),
            0x3a => Ok(Some(-1 - read_u32(r)? as i32)),
            _ => Ok(None),
        }
    }
}
impl_decode!(i32);

impl TryReadCbor for i64 {
    fn try_read_cbor<R: Read>(r: &mut R, major: u8) -> Result<Option<Self>> {
        match major {
            0x20..=0x37 => Ok(Some(-1 - (major - 0x20) as i64)),
            0x38 => Ok(Some(-1 - read_u8(r)? as i64)),
            0x39 => Ok(Some(-1 - read_u16(r)? as i64)),
            0x3a => Ok(Some(-1 - read_u32(r)? as i64)),
            0x3b => Ok(Some(-1 - read_u64(r)? as i64)),
            _ => Ok(None),
        }
    }
}
impl_decode!(i64);

impl TryReadCbor for f32 {
    fn try_read_cbor<R: Read>(r: &mut R, major: u8) -> Result<Option<Self>> {
        match major {
            0xfa => Ok(Some(read_f32(r)?)),
            _ => Ok(None),
        }
    }
}
impl_decode!(f32);

impl TryReadCbor for f64 {
    fn try_read_cbor<R: Read>(r: &mut R, major: u8) -> Result<Option<Self>> {
        match major {
            0xfa => Ok(Some(read_f32(r)? as f64)),
            0xfb => Ok(Some(read_f64(r)?)),
            _ => Ok(None),
        }
    }
}
impl_decode!(f64);

impl TryReadCbor for String {
    fn try_read_cbor<R: Read>(r: &mut R, major: u8) -> Result<Option<Self>> {
        let len = match major {
            0x60..=0x77 => major as usize - 0x60,
            0x78 => read_u8(r)? as usize,
            0x79 => read_u16(r)? as usize,
            0x7a => read_u32(r)? as usize,
            0x7b => {
                let len = read_u64(r)?;
                if len > usize::max_value() as u64 {
                    return Err(Error::LengthOutOfRange);
                }
                len as usize
            }
            _ => return Ok(None),
        };
        Ok(Some(read_str(r, len)?))
    }
}
impl_decode!(String);

impl TryReadCbor for Cid {
    fn try_read_cbor<R: Read>(r: &mut R, major: u8) -> Result<Option<Self>> {
        match major {
            0xd8 => Ok(Some(read_link(r)?)),
            _ => Ok(None),
        }
    }
}
impl_decode!(Cid);

impl TryReadCbor for Box<[u8]> {
    fn try_read_cbor<R: Read>(r: &mut R, major: u8) -> Result<Option<Self>> {
        let len = match major {
            0x40..=0x57 => major as usize - 0x40,
            0x58 => read_u8(r)? as usize,
            0x59 => read_u16(r)? as usize,
            0x5a => read_u32(r)? as usize,
            0x5b => {
                let len = read_u64(r)?;
                if len > usize::max_value() as u64 {
                    return Err(Error::LengthOutOfRange);
                }
                len as usize
            }
            _ => return Ok(None),
        };
        Ok(Some(read_bytes(r, len)?.into_boxed_slice()))
    }
}
impl_decode!(Box<[u8]>);

impl<T: TryReadCbor> TryReadCbor for Option<T> {
    fn try_read_cbor<R: Read>(r: &mut R, major: u8) -> Result<Option<Self>> {
        match major {
            0xf6 => Ok(Some(None)),
            0xf7 => Ok(Some(None)),
            _ => {
                if let Some(res) = T::try_read_cbor(r, major)? {
                    Ok(Some(Some(res)))
                } else {
                    Ok(None)
                }
            }
        }
    }
}
impl_decode!(Option<T>);

impl<T: TryReadCbor> TryReadCbor for Vec<T> {
    fn try_read_cbor<R: Read>(r: &mut R, major: u8) -> Result<Option<Self>> {
        let len = match major {
            0x80..=0x97 => major as usize - 0x80,
            0x98 => read_u8(r)? as usize,
            0x99 => read_u16(r)? as usize,
            0x9a => read_u32(r)? as usize,
            0x9b => {
                let len = read_u64(r)?;
                if len > usize::max_value() as u64 {
                    return Err(Error::LengthOutOfRange);
                }
                len as usize
            }
            _ => return Ok(None),
        };
        Ok(Some(read_list(r, len)?))
    }
}
impl_decode!(Vec<T>);

impl<T: TryReadCbor> TryReadCbor for BTreeMap<String, T> {
    fn try_read_cbor<R: Read>(r: &mut R, major: u8) -> Result<Option<Self>> {
        let len = match major {
            0xa0..=0xb7 => major as usize - 0xa0,
            0xb8 => read_u8(r)? as usize,
            0xb9 => read_u16(r)? as usize,
            0xba => read_u32(r)? as usize,
            0xbb => {
                let len = read_u64(r)?;
                if len > usize::max_value() as u64 {
                    return Err(Error::LengthOutOfRange);
                }
                len as usize
            }
            _ => return Ok(None),
        };
        Ok(Some(read_map(r, len)?))
    }
}
impl_decode!(BTreeMap<String, T>);

impl TryReadCbor for Ipld {
    fn try_read_cbor<R: Read>(r: &mut R, major: u8) -> Result<Option<Self>> {
        let ipld = match major {
            // Major type 0: an unsigned integer
            0x00..=0x17 => Ipld::Integer(major as i128),
            0x18 => Ipld::Integer(read_u8(r)? as i128),
            0x19 => Ipld::Integer(read_u16(r)? as i128),
            0x1a => Ipld::Integer(read_u32(r)? as i128),
            0x1b => Ipld::Integer(read_u64(r)? as i128),

            // Major type 1: a negative integer
            0x20..=0x37 => Ipld::Integer(-1 - (major - 0x20) as i128),
            0x38 => Ipld::Integer(-1 - read_u8(r)? as i128),
            0x39 => Ipld::Integer(-1 - read_u16(r)? as i128),
            0x3a => Ipld::Integer(-1 - read_u32(r)? as i128),
            0x3b => Ipld::Integer(-1 - read_u64(r)? as i128),

            // Major type 2: a byte string
            0x40..=0x57 => {
                let len = major - 0x40;
                let bytes = read_bytes(r, len as usize)?;
                Ipld::Bytes(bytes)
            }
            0x58 => {
                let len = read_u8(r)?;
                let bytes = read_bytes(r, len as usize)?;
                Ipld::Bytes(bytes)
            }
            0x59 => {
                let len = read_u16(r)?;
                let bytes = read_bytes(r, len as usize)?;
                Ipld::Bytes(bytes)
            }
            0x5a => {
                let len = read_u32(r)?;
                let bytes = read_bytes(r, len as usize)?;
                Ipld::Bytes(bytes)
            }
            0x5b => {
                let len = read_u64(r)?;
                if len > usize::max_value() as u64 {
                    return Err(Error::LengthOutOfRange);
                }
                let bytes = read_bytes(r, len as usize)?;
                Ipld::Bytes(bytes)
            }

            // Major type 3: a text string
            0x60..=0x77 => {
                let len = major - 0x60;
                let string = read_str(r, len as usize)?;
                Ipld::String(string)
            }
            0x78 => {
                let len = read_u8(r)?;
                let string = read_str(r, len as usize)?;
                Ipld::String(string)
            }
            0x79 => {
                let len = read_u16(r)?;
                let string = read_str(r, len as usize)?;
                Ipld::String(string)
            }
            0x7a => {
                let len = read_u32(r)?;
                let string = read_str(r, len as usize)?;
                Ipld::String(string)
            }
            0x7b => {
                let len = read_u64(r)?;
                if len > usize::max_value() as u64 {
                    return Err(Error::LengthOutOfRange);
                }
                let string = read_str(r, len as usize)?;
                Ipld::String(string)
            }

            // Major type 4: an array of data items
            0x80..=0x97 => {
                let len = major - 0x80;
                let list = read_list(r, len as usize)?;
                Ipld::List(list)
            }
            0x98 => {
                let len = read_u8(r)?;
                let list = read_list(r, len as usize)?;
                Ipld::List(list)
            }
            0x99 => {
                let len = read_u16(r)?;
                let list = read_list(r, len as usize)?;
                Ipld::List(list)
            }
            0x9a => {
                let len = read_u32(r)?;
                let list = read_list(r, len as usize)?;
                Ipld::List(list)
            }
            0x9b => {
                let len = read_u64(r)?;
                if len > usize::max_value() as u64 {
                    return Err(Error::LengthOutOfRange);
                }
                let list = read_list(r, len as usize)?;
                Ipld::List(list)
            }

            // Major type 5: a map of pairs of data items
            0xa0..=0xb7 => {
                let len = major - 0xa0;
                let map = read_map(r, len as usize)?;
                Ipld::Map(map)
            }
            0xb8 => {
                let len = read_u8(r)?;
                let map = read_map(r, len as usize)?;
                Ipld::Map(map)
            }
            0xb9 => {
                let len = read_u16(r)?;
                let map = read_map(r, len as usize)?;
                Ipld::Map(map)
            }
            0xba => {
                let len = read_u32(r)?;
                let map = read_map(r, len as usize)?;
                Ipld::Map(map)
            }
            0xbb => {
                let len = read_u64(r)?;
                if len > usize::max_value() as u64 {
                    return Err(Error::LengthOutOfRange);
                }
                let map = read_map(r, len as usize)?;
                Ipld::Map(map)
            }

            // Major type 6: optional semantic tagging of other major types
            0xd8 => Ipld::Link(read_link(r)?),

            // Major type 7: floating-point numbers and other simple data types that need no content
            0xf4 => Ipld::Bool(false),
            0xf5 => Ipld::Bool(true),
            0xf6 => Ipld::Null,
            0xf7 => Ipld::Null,
            0xfa => Ipld::Float(read_f32(r)? as f64),
            0xfb => Ipld::Float(read_f64(r)?),
            _ => return Ok(None),
        };
        Ok(Some(ipld))
    }
}
impl_decode!(Ipld);
