//! CBOR codec.
#![deny(missing_docs)]
#![deny(warnings)]

use core::convert::TryFrom;
use libipld_core::codec::{Codec, Decode, Encode};
pub use libipld_core::error::{Result, UnsupportedCodec};

pub mod decode;
pub mod encode;
pub mod error;

/// Raw CBOR codec.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct RawCborCodec;

impl Codec for RawCborCodec {}

impl From<RawCborCodec> for u64 {
    fn from(_: RawCborCodec) -> Self {
        0x51
    }
}

impl TryFrom<u64> for RawCborCodec {
    type Error = UnsupportedCodec;

    fn try_from(_: u64) -> core::result::Result<Self, Self::Error> {
        Ok(Self)
    }
}

/// Marker trait for types supporting the `CborCodec`.
pub trait Cbor42: Encode<RawCborCodec> + Decode<RawCborCodec> {}

impl<T: Encode<RawCborCodec> + Decode<RawCborCodec>> Cbor42 for T {}

#[cfg(test)]
mod tests {
    use super::*;
    use libipld_core::cid::Cid;
    use libipld_core::ipld::Ipld;
    use libipld_macro::ipld;
    use multihash::{Code, MultihashDigest};
    use std::collections::HashSet;

    #[test]
    fn test_encode_decode_cbor() {
        let cid = Cid::new_v1(0, Code::Blake3_256.digest(&b"cid"[..]));
        let ipld = ipld!({
          "number": 1,
          "list": [true, null, false],
          "bytes": vec![0, 1, 2, 3],
          "map": { "float": 0.0, "string": "hello" },
          "link": cid,
        });
        let bytes = RawCborCodec.encode(&ipld).unwrap();
        let ipld2 = RawCborCodec.decode(&bytes).unwrap();
        assert_eq!(ipld, ipld2);
    }

    #[test]
    fn test_references() {
        let cid = Cid::new_v1(0, Code::Blake3_256.digest(&b"0"[..]));
        let ipld = ipld!({
            "list": [true, cid],
        });
        let bytes = RawCborCodec.encode(&ipld).unwrap();
        let mut set = HashSet::new();
        RawCborCodec.references::<Ipld, _>(&bytes, &mut set).unwrap();
        assert!(set.contains(&cid));
    }
}
