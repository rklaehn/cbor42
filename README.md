# Raw CBOR codec for libipld

Unlike dag-cbor, which is a subset of valid CBOR, this crate is for any CBOR, additionally supporting [tag 42](https://www.iana.org/assignments/cbor-tags/cbor-tags.xhtml) for ipld links.

While dag-cbor is good if you want interoperability with dag-pb or dag-json, if you want just cbor with tags, this crate is for you.

## Non string keys

One thing dag-cbor does not allow but this crate allows is dictionaries with non-string keys, which can be very useful for data modeling. E.g. you have maps with integer keys or even structured keys that you want to encode without having to resort to encoding them as strings or using lists of 2-tuples.

This crate allows serializing any map
```rust
impl<K: Encode<CborCodec>, T: Encode<CborCodec> + 'static> Encode<CborCodec> for BTreeMap<K, T> {
```

IPLD dag-cbor allows serializing only maps with string keys
```rust
impl<T: Encode<DagCbor> + 'static> Encode<DagCbor> for BTreeMap<String, T> {
```

## Map key sort order

Another difference is that while the dag-cbor encoding spends some effort canonicalizing data (e.g. sorting by map keys), this
encoding does not. This can have some performance advantages.

In this crate, map data just gets written in whatever order it is in the map, without creating temporary objects:
```rust
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
```

In the IPLD dag-cbor crate, map data gets stored in a vec and then sorted by length
```rust
impl<T: Encode<DagCbor> + 'static> Encode<DagCbor> for BTreeMap<String, T> {
    fn encode<W: Write>(&self, c: DagCbor, w: &mut W) -> Result<()> {
        write_u64(w, MajorKind::Map, self.len() as u64)?;
        // CBOR RFC-7049 specifies a canonical sort order, where keys are sorted by length first.
        // This was later revised with RFC-8949, but we need to stick to the original order to stay
        // compatible with existing data.
        let mut cbor_order = Vec::from_iter(self);
        cbor_order.sort_unstable_by(|&(key_a, _), &(key_b, _)| {
            match key_a.len().cmp(&key_b.len()) {
                Ordering::Greater => Ordering::Greater,
                Ordering::Less => Ordering::Less,
                Ordering::Equal => key_a.cmp(key_b),
            }
        });
        for (k, v) in cbor_order {
            k.encode(c, w)?;
            v.encode(c, w)?;
        }
        Ok(())
    }
}
```
