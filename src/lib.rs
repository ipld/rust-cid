mod codec;
mod error;
/// ! # cid
/// !
/// ! Implementation of [cid](https://github.com/ipld/cid) in Rust.
mod to_cid;
mod version;

pub use codec::Codec;
pub use error::{Error, Result};
pub use to_cid::ToCid;
pub use version::Version;

use integer_encoding::{VarIntReader, VarIntWriter};
use multihash::Multihash;
use std::fmt;
use std::io::Cursor;

/// Representation of a CID.
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Cid {
    pub version: Version,
    pub codec: Codec,
    pub hash: Vec<u8>,
}

/// Prefix represents all metadata of a CID, without the actual content.
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Prefix {
    pub version: Version,
    pub codec: Codec,
    pub mh_type: multihash::Code,
    pub mh_len: usize,
}

impl Cid {
    /// Create a new CID.
    pub fn new(codec: Codec, version: Version, hash: &[u8]) -> Cid {
        Cid {
            version,
            codec,
            hash: hash.into(),
        }
    }

    /// Create a new CID from raw data (binary or multibase encoded string)
    pub fn from<T: ToCid>(data: T) -> Result<Cid> {
        data.to_cid()
    }

    /// Create a new CID from a prefix and some data.
    pub fn new_from_prefix(prefix: &Prefix, data: &[u8]) -> Cid {
        let mut hash = prefix.mh_type.hasher().unwrap().digest(data).into_bytes();
        hash.truncate(prefix.mh_len + 2);
        Cid {
            version: prefix.version,
            codec: prefix.codec.to_owned(),
            hash,
        }
    }

    fn to_string_v0(&self) -> String {
        use multibase::{encode, Base};

        let mut string = encode(Base::Base58Btc, self.hash.as_slice());

        // Drop the first character as v0 does not know
        // about multibase
        string.remove(0);

        string
    }

    fn to_string_v1(&self) -> String {
        use multibase::{encode, Base};

        encode(Base::Base58Btc, self.to_bytes().as_slice())
    }

    fn to_bytes_v0(&self) -> Vec<u8> {
        self.hash.clone()
    }

    fn to_bytes_v1(&self) -> Vec<u8> {
        let mut res = Vec::with_capacity(16);
        res.write_varint(u64::from(self.version)).unwrap();
        res.write_varint(u64::from(self.codec)).unwrap();
        res.extend_from_slice(&self.hash);

        res
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        match self.version {
            Version::V0 => self.to_bytes_v0(),
            Version::V1 => self.to_bytes_v1(),
        }
    }

    pub fn prefix(&self) -> Prefix {
        // Unwrap is safe, as this should have been validated on creation
        let mh = Multihash::from_bytes(Vec::from(self.hash.as_slice())).unwrap();

        Prefix {
            version: self.version,
            codec: self.codec.to_owned(),
            mh_type: mh.algorithm(),
            mh_len: mh.digest().len(),
        }
    }
}

#[allow(clippy::derive_hash_xor_eq)]
impl std::hash::Hash for Cid {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.to_bytes().hash(state);
    }
}

impl fmt::Display for Cid {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let output = match self.version {
            Version::V0 => self.to_string_v0(),
            Version::V1 => self.to_string_v1(),
        };
        write!(f, "{}", output)
    }
}

impl Prefix {
    pub fn new_from_bytes(data: &[u8]) -> Result<Prefix> {
        let mut cur = Cursor::new(data);

        let raw_version = cur.read_varint()?;
        let raw_codec = cur.read_varint()?;
        let raw_mh_type: u64 = cur.read_varint()?;

        let version = Version::from(raw_version)?;
        let codec = Codec::from(raw_codec)?;

        let mh_type = match multihash::Code::from_u64(raw_mh_type) {
            multihash::Code::Custom(_) => Err(Error::UnknownCodec),
            code => Ok(code),
        }?;

        let mh_len = cur.read_varint()?;

        Ok(Prefix {
            version,
            codec,
            mh_type,
            mh_len,
        })
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        let mut res = Vec::with_capacity(4);

        // io can't fail on Vec
        res.write_varint(u64::from(self.version)).unwrap();
        res.write_varint(u64::from(self.codec)).unwrap();
        res.write_varint(self.mh_type.to_u64()).unwrap();
        res.write_varint(self.mh_len as u64).unwrap();

        res
    }
}
