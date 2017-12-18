use std::path::{Path, PathBuf};
use std::fs::create_dir;
use std::result::Result;
use hex;
use base32::{self, Alphabet};


const ALPHABET: Alphabet = Alphabet::RFC4648{padding:false};
const RFC4648_ALPHABET: &'static [u8; 32] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";

pub struct Store {
    basedir: PathBuf,
}

impl Store {
    pub fn new(basedir: &Path) -> Store {
        Store{basedir: PathBuf::from(basedir)}
    }

    pub fn path2(&self, key: &[u8]) -> PathBuf {
        let b32 = base32::encode(ALPHABET, key);
        self.basedir.join(b32.get(0..2).unwrap()).join(b32.get(2..).unwrap())
    }

    pub fn path(&self, key: &[u8]) -> PathBuf {
        return self.basedir.join(hex::encode(key));
    }

    pub fn sig_path(&self, sig: &[u8; 400]) -> PathBuf {
        return self.path(&sig[0..64]);
    }

    pub fn sig_path2(&self, sig: &[u8; 400]) -> PathBuf {
        return self.path2(&sig[0..64]);
    }

    pub fn init_dirs(&self) {
        let mut ab  = [0u8; 2];
        let mut count: usize = 0;
        for a in RFC4648_ALPHABET.iter() {
            ab[0] = *a;
            for b in RFC4648_ALPHABET.iter() {
                ab[1] = *b;
                let name = String::from_utf8(ab.to_vec()).unwrap();
                create_dir(self.basedir.join(name).as_path()).unwrap();
            }
        }
    }
}


#[cfg(test)]
mod tests {
    use std::path::Path;
    use tempdir::TempDir;
    use std::thread::sleep_ms;
    use std::fs::read_dir;
    use super::{Store};

    #[test]
    fn test_path2() {
        let s = Store::new(Path::new("/b"));
        assert_eq!(s.path2(&[0; 32]).as_path(), Path::new("/b/AA/AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"));
        assert_eq!(s.path2(&[0; 48]).as_path(), Path::new("/b/AA/AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"));
    }

    #[test]
    fn test_path() {
        let s = Store::new(Path::new("/nope"));
        assert_eq!(s.path(&[]).as_path(), Path::new("/nope/"));
        assert_eq!(s.path(&[0]).as_path(), Path::new("/nope/00"));
        assert_eq!(s.path(&[255]).as_path(), Path::new("/nope/ff"));
        assert_eq!(s.path(&[0, 255]).as_path(), Path::new("/nope/00ff"));
        assert_eq!(s.path(&[255, 0]).as_path(), Path::new("/nope/ff00"));
    }

    #[test]
    fn test_sig_path() {
        let s = Store::new(Path::new("/nope"));
        let sig = [0u8; 400];
        assert_eq!(
            s.sig_path(&sig).as_path(),
            Path::new("/nope/00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000")
        );
    }

    #[test]
    fn test_sig_path2() {
        let s = Store::new(Path::new("/b"));
        let sig = [0u8; 400];
        assert_eq!(
            s.sig_path2(&sig).as_path(),
            Path::new("/b/AA/AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA")
        );
    }

    #[test]
    fn test_init_dirs() {
        let temp_dir = TempDir::new("buildchain-test").unwrap();
        let store = Store::new(temp_dir.path());
        store.init_dirs();

        let mut count = 0;
        for entry in read_dir(temp_dir.path()).unwrap() {
            count += 1;
        }
        assert_eq!(count, 1024);
        temp_dir.close().unwrap();
    }
}

