# Raw CBOR codec for libipld

Unlike dag-cbor, which is a subset of valid CBOR, this crate is for any CBOR, additionally supporting [tag 42](https://www.iana.org/assignments/cbor-tags/cbor-tags.xhtml) for ipld links.

While dag-cbor is good if you want interoperability with dag-pb or dag-json, if you want just cbor with tags, this crate is for you.

One thing dag-cbor does not allow but this crate allows is dictionaries with non-string keys, which can be very useful for data modeling. E.g. you have maps with integer keys or even structured keys that you want to encode without having to resort to encoding them as strings or using lists of 2-tuples.

TLDR:

This crate:
```rust
impl<K: Encode<CborCodec>, T: Encode<CborCodec> + 'static> Encode<CborCodec> for BTreeMap<K, T> {
```

IPLD dag-cbor:
```rust
impl<T: Encode<DagCbor> + 'static> Encode<DagCbor> for BTreeMap<String, T> {
```
