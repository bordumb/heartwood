use std::{fmt, str::FromStr};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::crypto;
use crate::crypto::PublicKey;

#[derive(Error, Debug)]
pub enum DidError {
    #[error("invalid did: {0}")]
    Did(String),
    #[error("invalid keri prefix: {0}")]
    Keri(String),
    #[error("invalid public key: {0}")]
    PublicKey(#[from] crypto::PublicKeyError),
}

#[derive(Serialize, Deserialize, Eq, PartialOrd, Ord, Hash, Clone)]
#[serde(into = "String", try_from = "String")]
pub enum Did {
    Key(PublicKey),
    Keri(String),
}

impl Did {
    /// We use the format specified by the DID `key` method, which is described as:
    ///
    /// `did:key:MULTIBASE(base58-btc, MULTICODEC(public-key-type, raw-public-key-bytes))`
    ///
    pub fn encode(&self) -> String {
        match self {
            Self::Key(key) => format!("did:key:{}", key.to_human()),
            Self::Keri(prefix) => format!("did:keri:{}", prefix),
        }
    }

    pub fn decode(input: &str) -> Result<Self, DidError> {
        if let Some(key) = input.strip_prefix("did:key:") {
            PublicKey::from_str(key).map(Self::Key).map_err(DidError::from)
        } else if let Some(prefix) = input.strip_prefix("did:keri:") {
            if prefix.is_empty() {
                return Err(DidError::Keri("prefix cannot be empty".into()));
            }
            // TODO: Full validation of KERI prefix
            if prefix
                .chars()
                .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
            {
                Ok(Self::Keri(prefix.to_owned()))
            } else {
                Err(DidError::Keri(format!("invalid characters in prefix",)))
            }
        } else {
            Err(DidError::Did(input.to_owned()))
        }
    }

    pub fn as_key(&self) -> Option<&PublicKey> {
        match self {
            Self::Key(key) => Some(key),
            _ => None,
        }
    }

    pub fn as_keri_prefix(&self) -> Option<&str> {
        match self {
            Self::Keri(prefix) => Some(prefix),
            _ => None,
        }
    }

    pub fn is_rotatable(&self) -> bool {
        matches!(self, Self::Keri(_))
    }

    pub fn to_ref_component(&self) -> String {
        match self {
            Self::Key(key) => key.to_human(),
            Self::Keri(prefix) => format!("did-keri-{}", prefix),
        }
    }
}

impl PartialEq for Did {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Key(l), Self::Key(r)) => l == r,
            (Self::Keri(l), Self::Keri(r)) => l == r,
            _ => false,
        }
    }
}

impl From<PublicKey> for Did {
    fn from(key: PublicKey) -> Self {
        Self::Key(key)
    }
}

impl From<&PublicKey> for Did {
    fn from(key: &PublicKey) -> Self {
        Self::Key(*key)
    }
}

impl From<Did> for String {
    fn from(other: Did) -> Self {
        other.encode()
    }
}

impl FromStr for Did {
    type Err = DidError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::decode(s)
    }
}

impl TryFrom<String> for Did {
    type Error = DidError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::decode(&value)
    }
}

impl fmt::Display for Did {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.encode())
    }
}

impl fmt::Debug for Did {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Did({:?})", self.to_string())
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod test {
    use super::*;

    #[test]
    fn test_did_encode_decode() {
        let input = "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK";
        let did = Did::decode(input).unwrap();

        assert_eq!(did.encode(), input);
        assert!(matches!(did, Did::Key(_)));
    }

    #[test]
    fn test_did_keri_encode_decode() {
        let input = "did:keri:EXq5YqaL6L48pf0fu7IUhL0JRaU2_RxFP0AL43wYn148";
        let did = Did::decode(input).unwrap();

        assert_eq!(did.encode(), input);
        assert!(matches!(did, Did::Keri(_)));
    }

    #[test]
    fn test_did_vectors() {
        Did::decode("did:key:z6MkiTBz1ymuepAQ4HEHYSF1H8quG5GLVVQR3djdX3mDooWp").unwrap();
        Did::decode("did:key:z6MkjchhfUsD6mmvni8mCdXHw216Xrm9bQe2mBH1P5RDjVJG").unwrap();
        Did::decode("did:key:z6MknGc3ocHs3zdPiJbnaaqDi58NGb4pk1Sp9WxWufuXSdxf").unwrap();
    }

    #[test]
    fn test_ref_component() {
        let key_did: Did = "did:key:z6MknSLrJoTcukLrE435hVNQT4JUhbvWLX4kUzqkEStBU8Vi"
            .parse()
            .unwrap();
        let keri_did: Did = "did:keri:EXq5YqaL6L48pf0fu7IUhL0JRaU2_RxFP0AL43wYn148"
            .parse()
            .unwrap();

        assert_eq!(
            key_did.to_ref_component(),
            "z6MknSLrJoTcukLrE435hVNQT4JUhbvWLX4kUzqkEStBU8Vi"
        );
        assert_eq!(
            keri_did.to_ref_component(),
            "did-keri-EXq5YqaL6L48pf0fu7IUhL0JRaU2_RxFP0AL43wYn148"
        );
    }
}
