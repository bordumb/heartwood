use core::fmt;
use core::str::FromStr;
use alloc::borrow::ToOwned;
use alloc::format;
use alloc::string::{String, ToString};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use radicle_crypto;
use radicle_crypto::PublicKey;

#[derive(Error, Debug)]
pub enum DidError {
    #[error("invalid did: {0}")]
    Did(String),
    #[error("invalid keri prefix: {0}")]
    Keri(String),
    #[error("invalid public key: {0}")]
    PublicKey(#[from] radicle_crypto::PublicKeyError),
}

#[derive(Serialize, Deserialize, Eq, PartialOrd, Ord, Hash, Clone)]
#[serde(into = "String", try_from = "String")]
pub enum Did {
    Key(PublicKey),
    Keri(String),
}

impl Did {
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
            if prefix.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
                Ok(Self::Keri(prefix.to_owned()))
            } else {
                Err(DidError::Keri(format!("invalid characters in prefix")))
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
