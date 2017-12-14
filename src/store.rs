use std::path::PathBuf;
use hex::encode;

pub struct Store {
    basedir: PathBuf,
}

impl Store {
    pub fn path(&self, key: &[u8]) -> PathBuf {
        let encoded = encode(key);
        return self.basedir.join(encoded);
    }

}
