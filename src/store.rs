use std::path::PathBuf;
use hex;
use base32::{self, Alphabet};


const ALPHABET: Alphabet = Alphabet::RFC4648{padding:false};

pub struct Store {
    basedir: PathBuf,
}

impl Store {
    pub fn new(basedir: &str) -> Store {
        Store{basedir: PathBuf::from(basedir)}
    }

    pub fn path2(&self, key: &[u8]) -> PathBuf {
        let b32 = base32::encode(ALPHABET, key);
        return self.basedir.join(b32.get(0..2).unwrap()).join(b32.get(2..).unwrap());
    }

    pub fn path(&self, key: &[u8]) -> PathBuf {
        return self.basedir.join(hex::encode(key));
    }

    pub fn sig_path(&self, sig: &[u8; 400]) -> PathBuf {
        return self.path(&sig[0..64]);
    }

    pub fn sig_path2(&self, sig: &[u8; 400]) -> PathBuf {
        return self.path2(sig);
    }
}


#[cfg(test)]
mod tests {
    use std::path::Path;
    use super::{Store};

    #[test]
    fn test_path2() {
        let s = Store::new("/b");
        assert_eq!(s.path2(&[0; 32]).as_path(), Path::new("/b/AA/AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"));
        assert_eq!(s.path2(&[0; 48]).as_path(), Path::new("/b/AA/AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"));
    }

    #[test]
    fn test_path() {
        let s = Store::new("/nope");
        assert_eq!(s.path(&[]).as_path(), Path::new("/nope/"));
        assert_eq!(s.path(&[0]).as_path(), Path::new("/nope/00"));
        assert_eq!(s.path(&[255]).as_path(), Path::new("/nope/ff"));
        assert_eq!(s.path(&[0, 255]).as_path(), Path::new("/nope/00ff"));
        assert_eq!(s.path(&[255, 0]).as_path(), Path::new("/nope/ff00"));
    }

    #[test]
    fn test_sig_path() {
        let s = Store::new("/nope");
        let sig = [0u8; 400];
        assert_eq!(
            s.sig_path(&sig).as_path(),
            Path::new("/nope/00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000")
        );
    }

    #[test]
    fn test_sig_path2() {
        let s = Store::new("/b");
        let sig = [0u8; 400];
        assert_eq!(
            s.sig_path2(&sig).as_path(),
            Path::new("/b/AA/AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA")
        );
    }
}
