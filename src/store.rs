use std::path::PathBuf;
use hex;

pub struct Store {
    basedir: PathBuf,
}

impl Store {
    pub fn new(basedir: &str) -> Store {
        Store{basedir: PathBuf::from(basedir)}
    }

    pub fn path(&self, key: &[u8]) -> PathBuf {
        return self.basedir.join(hex::encode(key));
    }

}


#[cfg(test)]
mod tests {
    use std::path::Path;
    use super::{Store};

    #[test]
    fn test_path() {
        let s = Store::new("/nope");
        assert_eq!(s.path(&[]).as_path(), Path::new("/nope/"));
        assert_eq!(s.path(&[0]).as_path(), Path::new("/nope/00"));
        assert_eq!(s.path(&[255]).as_path(), Path::new("/nope/ff"));
        assert_eq!(s.path(&[0, 255]).as_path(), Path::new("/nope/00ff"));
        assert_eq!(s.path(&[255, 0]).as_path(), Path::new("/nope/ff00"));
    }
}
