//! Identificador opaco de blob (`BlobHash`), independente do tipo de hash usado por qualquer adapter.

use std::fmt;

use serde::{de, Deserialize, Deserializer, Serialize, Serializer};

/// Identificador content-addressed de um blob.
///
/// Wrapper opaco sobre 32 bytes — não amarrado ao tipo `Hash` de nenhum adapter específico
/// (ex: `iroh_blobs::Hash`), para não vazar detalhe de implementação na assinatura pública de
/// `P2pBlobStore`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlobHash([u8; 32]);

impl BlobHash {
    /// Constrói a partir dos 32 bytes brutos do hash.
    pub const fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    /// Os 32 bytes brutos do hash.
    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

impl From<[u8; 32]> for BlobHash {
    fn from(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }
}

impl fmt::Display for BlobHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in &self.0 {
            write!(f, "{byte:02x}")?;
        }
        Ok(())
    }
}

/// Erro ao interpretar uma string como `BlobHash` hexadecimal.
#[derive(Debug, thiserror::Error)]
pub enum BlobHashParseError {
    /// A string não tem exatamente 64 caracteres hexadecimais (32 bytes).
    #[error("invalid blob hash length: expected 64 hex chars, got {0}")]
    InvalidLength(usize),
    /// A string contém caracteres fora do alfabeto hexadecimal.
    #[error("invalid blob hash encoding: {0}")]
    InvalidHex(String),
}

impl std::str::FromStr for BlobHash {
    type Err = BlobHashParseError;

    fn from_str(hex: &str) -> Result<Self, Self::Err> {
        if hex.len() != 64 {
            return Err(BlobHashParseError::InvalidLength(hex.len()));
        }

        let mut bytes = [0u8; 32];
        for (index, chunk) in hex.as_bytes().as_chunks::<2>().0.iter().enumerate() {
            let pair = std::str::from_utf8(chunk)
                .map_err(|_| BlobHashParseError::InvalidHex("non-utf8 byte pair".to_string()))?;
            bytes[index] = u8::from_str_radix(pair, 16)
                .map_err(|err| BlobHashParseError::InvalidHex(err.to_string()))?;
        }

        Ok(Self(bytes))
    }
}

impl Serialize for BlobHash {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for BlobHash {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let hex = String::deserialize(deserializer)?;
        hex.parse().map_err(de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    #[test]
    fn hex_round_trip_preserves_bytes() {
        let hash = BlobHash::from_bytes([0x42; 32]);
        let hex = hash.to_string();
        let parsed = BlobHash::from_str(&hex).unwrap();
        assert_eq!(hash, parsed);
    }

    #[test]
    fn display_is_lowercase_hex_of_expected_length() {
        let hash = BlobHash::from_bytes([0xab; 32]);
        let hex = hash.to_string();
        assert_eq!(hex.len(), 64);
        assert!(hex.chars().all(|c| c.is_ascii_hexdigit() && !c.is_uppercase()));
    }

    #[test]
    fn from_str_rejects_wrong_length() {
        assert!(matches!(BlobHash::from_str("abcd"), Err(BlobHashParseError::InvalidLength(4))));
    }

    #[test]
    fn from_str_rejects_non_hex_characters() {
        let invalid = "z".repeat(64);
        assert!(matches!(BlobHash::from_str(&invalid), Err(BlobHashParseError::InvalidHex(_))));
    }

    #[test]
    fn serde_round_trip_via_json() {
        let hash = BlobHash::from_bytes([0x07; 32]);
        let json = serde_json::to_string(&hash).unwrap();
        let parsed: BlobHash = serde_json::from_str(&json).unwrap();
        assert_eq!(hash, parsed);
    }

    #[test]
    fn distinct_bytes_produce_distinct_hashes() {
        let a = BlobHash::from_bytes([0x01; 32]);
        let b = BlobHash::from_bytes([0x02; 32]);
        assert_ne!(a, b);
    }
}
