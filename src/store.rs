use std::fs::{File, OpenOptions, create_dir, rename};
use std::io::{self, Write, Read};
use std::os::unix::fs::{OpenOptionsExt,PermissionsExt};
use std::path::{Path, PathBuf};
use std::result::Result;

use base32::{self, Alphabet};
use rand::{Rng, OsRng};
use sha2::{Sha384, Digest};

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


fn create_dir_if_needed<P: AsRef<Path>>(path: P) ->io::Result<()> {
    if path.as_ref().is_dir() {
        return Ok(());
    }
    create_dir(path.as_ref())
}


fn to_canonical<P: AsRef<Path>, Q: AsRef<Path>>(src: P, dst: Q) -> io::Result<()> {
    let parent = dst.as_ref().parent().unwrap();
    create_dir_if_needed(parent.parent().unwrap())?;
    create_dir_if_needed(parent)?;
    rename(src.as_ref(), dst.as_ref())?;
    Ok(())
}

impl Store {
    pub fn new<P: AsRef<Path>>(basedir: P) -> Store {
        Store{basedir: PathBuf::from(basedir.as_ref())}
    }

    pub fn temp_path(&self) -> PathBuf {
        self.basedir.join("tmp").join(random_id())
    }

    pub fn object_path(&self, key: &[u8; 48]) -> PathBuf {
        self.basedir.join("object").join(relpath_2(key))
    }

    pub fn block_path(&self, sig: &[u8; 64]) -> PathBuf {
        self.basedir.join("block").join(relpath_2(sig))
    }

    pub fn path_2(&self, key: &[u8]) -> PathBuf {
        self.basedir.join(relpath_2(key))
    }

    pub fn sig_path_2(&self, sig: &[u8; 400]) -> PathBuf {
        return self.path_2(&sig[0..64]);
    }

    pub fn init_dirs(&self) {
        create_dir(self.basedir.join("tmp").as_path()).unwrap();
        let mut ab  = [0u8; 2];
        for a in RFC4648_ALPHABET.iter() {
            ab[0] = *a;
            for b in RFC4648_ALPHABET.iter() {
                ab[1] = *b;
                let name = String::from_utf8(ab.to_vec()).unwrap();
                create_dir(self.basedir.join(name).as_path()).unwrap();
            }
        }
    }

    pub fn open_object(&self, key: &[u8; 48]) -> io::Result<File> {
        File::open(self.path_2(&key[..]))
    }

    pub fn write_object(&self, content: &[u8]) -> Result<[u8; 48], String> {
        let mut key = [0; 48];
        let digest = Sha384::digest(content);
        key.copy_from_slice(digest.as_slice());

        let tmp = self.temp_path();
        let dst = self.path_2(&key[..]);
        {
            let mut file = OpenOptions::new()
                .create_new(true)
                .mode(0o400)
                .write(true)
                .open(tmp.as_path())
            .map_err(|err| {
                format!("failed to create file {:?}: {}", tmp.as_path(), err)
            })?;

            file.write_all(&content).map_err(|err| {
                format!("failed to write {:?}: {}", tmp.as_path(), err)
            })?;
            file.sync_all().map_err(|err| {
                format!("failed to sync {:?}: {}", tmp.as_path(), err)
            })?;
        }

        to_canonical(tmp.as_path(), dst.as_path()).map_err(|err| {
            format!("failed to rename {:?} to {:?}: {}", tmp.as_path(), dst.as_path(), err)
        })?;

        Ok(key)
    }
}


#[cfg(test)]
mod tests {
    use std::path::Path;
    use std::fs::read_dir;
    use std::io::Read;
    use std::os::unix::fs::PermissionsExt;

    use tempdir::TempDir;
    use rand::{Rng, OsRng};

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
        assert_ne!(p1.to_str().unwrap()[10..], p2.to_str().unwrap()[10..]);
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
        let store = Store::new(&temp_dir);
        store.init_dirs();

        let mut count = 0;
        for entry in read_dir(temp_dir.path()).unwrap() {
            count += 1;
        }
        assert_eq!(count, 1025);
        temp_dir.close().unwrap();
    }

    #[test]
    fn test_write_object() {
        let temp_dir = TempDir::new("buildchain-test").unwrap();
        let store = Store::new(&temp_dir);
        store.init_dirs();

        let mut rng = match OsRng::new() {
            Ok(g) => g,
            Err(e) => panic!("Failed to obtain OsRng: {}", e),
        };
        let mut content = [0u8; 1776];
        rng.fill_bytes(&mut content);
        let content = content;
        let key: [u8; 48] = match store.write_object(&content) {
            Ok(g) => g,
            Err(e) => panic!("Error: {}", e),
        };

        {
            let mut file = store.open_object(&key).unwrap();

            let perm = file.metadata().unwrap().permissions();
            assert_eq!(perm.mode() & 511, 0o400);

            let mut buf = [0u8; 1776];
            assert_eq!(file.read(&mut buf).unwrap(), buf.len());
            assert_eq!(content.to_vec(), buf.to_vec());
        }

        temp_dir.close().unwrap();
    }
}

