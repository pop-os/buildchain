use std::fs::{File, OpenOptions, create_dir, rename};
use std::io::{self, Write};
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};
use std::result::Result;

use base32::{self, Alphabet};
use rand::{Rng, OsRng};
use sha2::{Sha384, Digest};

const ALPHABET: Alphabet = Alphabet::RFC4648{padding:false};

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


fn create_dir_if_needed<P: AsRef<Path>>(path: P) ->Result<(), String> {
    if path.as_ref().is_dir() {
        return Ok(());
    }
    create_dir(path.as_ref()).map_err(|err| {
        format!("create_dir failed: {:?}: {}", path.as_ref(), err)
    })
}


fn to_canonical<P: AsRef<Path>, Q: AsRef<Path>>(src: P, dst: Q) -> Result<(), String> {
    let parent = dst.as_ref().parent().unwrap();
    create_dir_if_needed(parent.parent().unwrap())?;
    create_dir_if_needed(parent)?;
    rename(src.as_ref(), dst.as_ref()).map_err(|err| {
        format!("rename failed: {:?} -> {:?}: {}", src.as_ref(), dst.as_ref(), err)
    })
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

    pub fn open_object(&self, key: &[u8; 48]) -> io::Result<File> {
        File::open(self.object_path(key))
    }

    pub fn write_object(&self, content: &[u8]) -> Result<[u8; 48], String> {
        let key = {
            let mut key = [0u8; 48];
            let digest = Sha384::digest(content);
            key.copy_from_slice(digest.as_slice());
            key
        };

        let tmp = self.temp_path();
        create_dir_if_needed(tmp.parent().unwrap())?;
        {
            let mut opt = OpenOptions::new();
            let opt = opt.create_new(true).write(true).mode(0o400);
            let mut file = opt.open(tmp.as_path()).map_err(|err| {
                format!("failed to create file {:?}: {}", tmp.as_path(), err)
            })?;
            file.write_all(&content).map_err(|err| {
                format!("failed to write {:?}: {}", tmp.as_path(), err)
            })?;
            file.sync_all().map_err(|err| {
                format!("failed to sync {:?}: {}", tmp.as_path(), err)
            })?;
        }

        let dst = self.object_path(&key);
        to_canonical(tmp.as_path(), dst.as_path())?;

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
    fn test_object_path() {
        let s = Store::new(Path::new("/p"));
        assert_eq!(
            s.object_path(&[0u8; 48]).as_path(),
            Path::new("/p/object/AA/AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA")
        );
        assert_eq!(
            s.object_path(&[255u8; 48]).as_path(),
            Path::new("/p/object/77/777777777777777777777777777777777777777777777777777777777777777777777777776")
        );
    }

    #[test]
    fn test_block_path() {
        let s = Store::new(Path::new("/p"));
        let sig1 = [0u8; 64];
        let sig2 = [255u8; 64];
        assert_eq!(
            s.block_path(&sig1).as_path(),
            Path::new("/p/block/AA/AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA")
        );
        assert_eq!(
            s.block_path(&sig2).as_path(),
            Path::new("/p/block/77/7777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777Y")
        );
    }

    #[test]
    fn test_write_object() {
        let temp_dir = TempDir::new("buildchain-test").unwrap();
        let store = Store::new(&temp_dir);

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

