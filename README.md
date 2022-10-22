# Raw CBOR codec for libipld

Unlike dag-cbor, which is a subset of valid CBOR, this crate is for any CBOR, additionally supporting tag 42 for ipld links.

While dag-cbor is good if you want interoperability with dag-pb or dag-json and the wider IPLD ecosystem, if you want just cbor with tags, this crate is for you.

One thing dag-cbor does not allow but this crate allows is dictionaries with non-string keys, which can be very useful for data
modeling.

This crate:
```rust
impl<K: Encode<CborCodec>, T: Encode<CborCodec> + 'static> Encode<CborCodec> for BTreeMap<K, T> {
```

IPLD dag-cbor:
```rust
impl<T: Encode<DagCbor> + 'static> Encode<DagCbor> for BTreeMap<String, T> {
```