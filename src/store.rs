use std::path::{Path, PathBuf};
use std::fs::create_dir;
use std::result::Result;
use hex;
use base32::{self, Alphabet};
use rand::{Rng, OsRng};

const ALPHABET: Alphabet = Alphabet::RFC4648{padding:false};
const RFC4648_ALPHABET: &'static [u8; 32] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";

pub struct Store {
    basedir: PathBuf,
}

pub fn relpath(key: &[u8]) -> PathBuf {
    let b32 = base32::encode(ALPHABET, key);
    PathBuf::from(b32)
}

pub fn relpath_2(key: &[u8]) -> PathBuf {
    let b32 = base32::encode(ALPHABET, key);
    let path = PathBuf::new();
    path.join(b32.get(0..2).unwrap()).join(b32.get(2..).unwrap())
}

pub fn random_id() -> String {
    let mut rng = match OsRng::new() {
        Ok(g) => g,
        Err(e) => panic!("Failed to obtain OsRng: {}", e),
    };
    let mut key = [0u8; 15];
    rng.fill_bytes(&mut key);
    base32::encode(ALPHABET, &key)
}

impl Store {
    pub fn new(basedir: &Path) -> Store {
        Store{basedir: PathBuf::from(basedir)}
    }

    pub fn temp_path(&self) -> PathBuf {
        self.basedir.join("tmp").join(random_id())
    }

    pub fn path_2(&self, key: &[u8]) -> PathBuf {
        self.basedir.join(relpath_2(key))
    }

    pub fn path(&self, key: &[u8]) -> PathBuf {
        return self.basedir.join(hex::encode(key));
    }

    pub fn sig_path(&self, sig: &[u8; 400]) -> PathBuf {
        return self.path(&sig[0..64]);
    }

    pub fn sig_path_2(&self, sig: &[u8; 400]) -> PathBuf {
        return self.path_2(&sig[0..64]);
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
    use std::fs::read_dir;
    use super::{Store};

    #[test]
    fn test_new() {
        let s = Store::new(Path::new("/nope"));
        assert_eq!(s.basedir.as_path(), Path::new("/nope/"));
    }

    #[test]
    fn test_temp_path() {
        let s = Store::new(Path::new("/nope"));

        let p1 = s.temp_path();
        assert_eq!(&(p1.to_str().unwrap()[0..10]), "/nope/tmp/");
        assert_eq!(p1.to_str().unwrap().len(), 34);

        let p2 = s.temp_path();
        assert_eq!(&(p2.to_str().unwrap()[0..10]), "/nope/tmp/");
        assert_eq!(p2.to_str().unwrap().len(), 34);

        assert_ne!(p1, p2);
        assert_ne!(p1.to_str().unwrap(), p2.to_str().unwrap());
        assert_eq!(p1.to_str().unwrap()[..10], p2.to_str().unwrap()[..10]);
        assert_nq!(p1.to_str().unwrap()[10..], p2.to_str().unwrap()[10..]);
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
    fn test_path_2() {
        let s = Store::new(Path::new("/b"));
        assert_eq!(
            s.path_2(&[0; 32]).as_path(),
            Path::new("/b/AA/AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA")
        );
        assert_eq!(
            s.path_2(&[255; 32]).as_path(),
            Path::new("/b/77/7777777777777777777777777777777777777777777777777Q")
        );
        assert_eq!(
            s.path_2(&[0; 48]).as_path(),
            Path::new("/b/AA/AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA")
        );
        assert_eq!(
            s.path_2(&[255; 48]).as_path(),
            Path::new("/b/77/777777777777777777777777777777777777777777777777777777777777777777777777776")
        );
        assert_eq!(
            s.path_2(&[0; 64]).as_path(),
            Path::new("/b/AA/AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA")
        );
        assert_eq!(
            s.path_2(&[255; 64]).as_path(),
            Path::new("/b/77/7777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777Y")
        );
    }

    #[test]
    fn test_sig_path_2() {
        let s = Store::new(Path::new("/b"));
        let sig1 = [0u8; 400];
        let sig2 = [255u8; 400];
        assert_eq!(
            s.sig_path_2(&sig1).as_path(),
            Path::new("/b/AA/AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA")
        );
        assert_eq!(
            s.sig_path_2(&sig2).as_path(),
            Path::new("/b/77/7777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777Y")
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

