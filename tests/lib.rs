extern crate cid;
extern crate try_from;
extern crate multihash;

use cid::{Cid, Codec, Error, Prefix};
use try_from::TryFrom;

#[test]
fn basic_marshalling() {
    let h = multihash::encode(multihash::Hash::SHA2256, b"beep boop").unwrap();

    let cid = Cid::new(Codec::DagProtobuf, 1, h.to_vec());

    let data = cid.as_bytes();
    let out = Cid::try_from(data).unwrap();

    assert_eq!(cid, out);

    let s = cid.to_string();
    let out2 = Cid::try_from(&s[..]).unwrap();

    assert_eq!(cid, out2);
}

#[test]
fn empty_string() {
    assert_eq!(Cid::try_from(""), Err(Error::InputTooShort));
}

#[test]
fn v0_handling() {
    let old = "QmdfTbBqBPQ7VNxZEYEj14VmRuZBkqFbiwReogJgS1zR1n";
    let cid = Cid::try_from(old).unwrap();

    assert_eq!(cid.version, 0);
    assert_eq!(cid.to_string(), old);
}

#[test]
fn v0_error() {
    let bad = "QmdfTbBqBPQ7VNxZEYEj14VmRuZBkqFbiwReogJgS1zIII";
    assert_eq!(Cid::try_from(bad), Err(Error::ParsingError));
}

#[test]
fn prefix_roundtrip() {
    let data = b"awesome test content";
    let h = multihash::encode(multihash::Hash::SHA2256, data).unwrap();

    let cid = Cid::new(Codec::DagProtobuf, 1, h.to_vec());
    let prefix = cid.prefix();

    let cid2 = Cid::new_from_prefix(&prefix, data);

    assert_eq!(cid, cid2);

    let prefix_bytes = prefix.as_bytes();
    let prefix2 = Prefix::new_from_bytes(&prefix_bytes).unwrap();

    assert_eq!(prefix, prefix2);
}

#[test]
fn try_from() {
    let the_hash = "QmdfTbBqBPQ7VNxZEYEj14VmRuZBkqFbiwReogJgS1zR1n";

    let cases = vec![
        format!("/ipfs/{:}", &the_hash),
        format!("https://ipfs.io/ipfs/{:}", &the_hash),
        format!("http://localhost:8080/ipfs/{:}", &the_hash),
    ];

    for case in cases {
        let cid = Cid::try_from(case).unwrap();
        assert_eq!(cid.version, 0);
        assert_eq!(cid.to_string(), the_hash);
    }
}
