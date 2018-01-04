use serde::{Serializer, Deserializer, Deserialize};
use sha2::{self, Digest};
use std::io::{self, Read};

use store::{b32enc, b32dec};

/// Deserializes a lowercase hex string to a `Vec<u8>`.
fn from_base32<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Vec<u8>, D::Error> {
    use serde::de::Error;
    String::deserialize(deserializer).and_then(|string| {
        b32dec(&string).ok_or(Error::custom("b32dec failed"))
    })
}

/// Serializes `buffer` to a lowercase hex string.
fn to_base32<T: AsRef<[u8]>, S: Serializer>(buffer: &T, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(&b32enc(&buffer.as_ref()))
}

/// A serializable representation of a Sha384 hash
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct Sha384(
    #[serde(deserialize_with = "from_base32", serialize_with = "to_base32")]
    Vec<u8>
);

impl Sha384 {
    /// Create a new Sha384 by reading the provided input
    ///
    /// # Arguments
    ///
    /// * `input` - A `std::io::Read` object that will be used as input to the Sha384 algorithm
    ///
    /// # Return
    ///
    /// The Sha384 of the provided input data
    ///
    /// # Errors
    ///
    /// Errors that are encountered while reading will be returned
    ///
    /// # Example
    ///
    /// ```
    /// use buildchain::Sha384;
    ///
    /// let data = "Input";
    /// let sha = Sha384::new(data.as_bytes()).unwrap();
    /// ```
    pub fn new<R: Read>(mut input: R) -> io::Result<Sha384> {
        let mut hasher = sha2::Sha384::default();

        loop {
            let mut data = [0; 4096];
            let count = input.read(&mut data)?;
            if count == 0 {
                break;
            }

            hasher.input(&data[..count]);
        }

        Ok(Sha384(
            hasher.result().as_slice().to_vec()
        ))
    }

    pub fn to_base32(&self) -> String {
        let key = {
            let mut key = [0u8; 48];
            key.copy_from_slice(&self.0);
            key
        };
        b32enc(&key)
    }
}

#[cfg(test)]
mod tests {
    use super::Sha384;

    #[test]
    fn test_match() {
        let a = "Input";
        let b = "Input";
        let sha_a = Sha384::new(a.as_bytes()).unwrap();
        let sha_b = Sha384::new(b.as_bytes()).unwrap();

        assert_eq!(sha_a, sha_b);
    }

    #[test]
    fn test_mismatch() {
        let a = "Input A";
        let b = "Input B";
        let sha_a = Sha384::new(a.as_bytes()).unwrap();
        let sha_b = Sha384::new(b.as_bytes()).unwrap();

        assert_ne!(sha_a, sha_b);
    }
}
