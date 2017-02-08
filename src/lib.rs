/// ! # cid
/// !
/// ! Implementation of [cid](https://github.com/ipld/cid) in Rust.

extern crate multihash;
extern crate multibase;
extern crate varmint;

mod to_cid;
mod error;
mod codec;
mod version;

pub use to_cid::ToCid;
pub use version::Version;
pub use codec::Codec;
pub use error::{Error, Result};

use varmint::{WriteVarInt, ReadVarInt};
use std::fmt;

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
    pub mh_type: multihash::Hash,
    pub mh_len: usize,
}

impl Cid {
    /// Create a new CID.
    pub fn new(codec: Codec, version: Version, hash: &[u8]) -> Cid {
        Cid {
            version: version,
            codec: codec,
            hash: hash.into(),
        }
    }

    /// Create a new CID from raw data (binary or multibase encoded string)
    pub fn from<T: ToCid>(data: T) -> Result<Cid> {
        data.to_cid()
    }

    /// Create a new CID from a prefix and some data.
    pub fn new_from_prefix(prefix: &Prefix, data: &[u8]) -> Cid {
        let mut hash = multihash::encode(prefix.mh_type.to_owned(), data).unwrap();
        hash.truncate(prefix.mh_len + 2);
        Cid {
            version: prefix.version,
            codec: prefix.codec.to_owned(),
            hash: hash,
        }
    }

    fn to_string_v0(&self) -> Result<String> {
        use multibase::{encode, Base};

        let mut string = encode(Base::Base58btc, self.hash.as_slice())?;

        // Drop the first character as v0 does not know
        // about multibase
        string.remove(0);

        Ok(string)
    }

    fn to_string_v1(&self) -> Result<String> {
        use multibase::{encode, Base};

        encode(Base::Base58btc, self.to_bytes().as_slice()).map_err(Into::into)
    }

    pub fn to_string(&self) -> Result<String> {
        match self.version {
            Version::V0 => self.to_string_v0(),
            Version::V1 => self.to_string_v1(),
        }
    }

    fn to_bytes_v0(&self) -> Vec<u8> {
        self.hash.clone()
    }

    fn to_bytes_v1(&self) -> Vec<u8> {
        let mut res = Vec::with_capacity(16);
        res.write_u64_varint(self.version.into()).unwrap();
        res.write_u64_varint(self.codec.into()).unwrap();
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
        let mh = multihash::decode(self.hash.as_slice()).unwrap();

        Prefix {
            version: self.version,
            codec: self.codec.to_owned(),
            mh_type: mh.alg,
            mh_len: mh.digest.len(),
        }
    }
}

impl fmt::Display for Cid {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", Cid::to_string(self)?)
    }
}

impl Prefix {
    pub fn new_from_bytes(data: &[u8]) -> Result<Prefix> {
        let mut data = data;

        let raw_version = data.read_u64_varint()?;
        let raw_codec = data.read_u64_varint()?;
        let raw_mh_type = data.read_u64_varint()?;

        let version = Version::from(raw_version)?;
        let codec = Codec::from(raw_codec)?;

        let mh_type = multihash::Hash::from_code(raw_mh_type as u8)?;

        let mh_len = data.read_u64_varint()?;

        Ok(Prefix {
            version: version,
            codec: codec,
            mh_type: mh_type,
            mh_len: mh_len as usize,
        })
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        let mut res = Vec::with_capacity(4);

        // io can't fail on Vec
        res.write_u64_varint(self.version.into()).unwrap();
        res.write_u64_varint(self.codec.into()).unwrap();
        res.write_u64_varint(self.mh_type.code() as u64).unwrap();
        res.write_u64_varint(self.mh_len as u64).unwrap();

        res
    }
}
