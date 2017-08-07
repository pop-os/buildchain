use std::collections::BTreeMap;
use std::fs::{File, read_dir};
use std::io::{Error, ErrorKind, Result};
use std::path::Path;

use super::Sha384;

/// A manifest of build artifacts
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct Manifest {
    /// The timestamp of the source control revision
    pub time: u64,
    /// A dictionary of filenames and their hashes
    pub files: BTreeMap<String, Sha384>,
}

impl Manifest {
    /// Create a new Manifest by reading the provided build directory
    ///
    /// # Arguments
    ///
    /// * `time` - the timestamp of the source control revision that was built
    /// * `path` - the directory containing the build artifacts
    ///
    /// # Return
    ///
    /// The Manifest of the provided build data
    ///
    /// # Errors
    ///
    /// Errors that are encountered while reading will be returned
    ///
    /// # Example
    ///
    /// ```
    /// use buildchain::Manifest;
    ///
    /// let manifest = Manifest::new(1500000000, "tests/res/artifacts").unwrap();
    /// ```
    pub fn new<P: AsRef<Path>>(time: u64, path: P) -> Result<Manifest> {
        let mut files = BTreeMap::new();

        for entry_res in read_dir(path.as_ref())? {
            let entry = entry_res?;

            let name = entry.file_name().into_string().map_err(|_| {
                Error::new(ErrorKind::InvalidData, "Filename is not UTF-8")
            })?;

            let file = File::open(entry.path())?;
            let sha = Sha384::new(file)?;

            files.insert(name, sha);
        }

        Ok(Manifest {
            time: time,
            files: files,
        })
    }
}


#[cfg(test)]
mod tests {
    use serde_json;
    use std::fs::File;
    use std::io::Read;

    use super::Manifest;

    #[test]
    fn test_artifacts() {
        let manifest_json = {
            let mut file = File::open("tests/res/manifest.json").unwrap();
            let mut json = String::new();
            file.read_to_string(&mut json).unwrap();
            serde_json::from_str::<Manifest>(&json).unwrap()
        };

        let manifest_dir = Manifest::new(1500000000, "tests/res/artifacts").unwrap();

        assert_eq!(manifest_json, manifest_dir);
    }
}
