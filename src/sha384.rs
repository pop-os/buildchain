use hex::{self,FromHex};
use serde::{Serializer, Deserializer, Deserialize};
use sha2::{self, Digest};
use std::io::{self, Read};

/// Deserializes a lowercase hex string to a `Vec<u8>`.
fn from_hex<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Vec<u8>, D::Error> {
    use serde::de::Error;
    String::deserialize(deserializer).and_then(|string| {
        Vec::from_hex(&string).map_err(|err| {
            Error::custom(err.to_string())
        })
    })
}

/// Serializes `buffer` to a lowercase hex string.
fn to_hex<T: AsRef<[u8]>, S: Serializer>(buffer: &T, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(&hex::encode(&buffer.as_ref()))
}

/// A serializable representation of a Sha384 hash
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct Sha384(
    #[serde(deserialize_with = "from_hex", serialize_with = "to_hex")]
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
