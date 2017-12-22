use std::fs::{File, OpenOptions, create_dir, remove_dir, rename};
use std::io::{self, Write, Read};
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::result::Result;

use base32::{self, Alphabet};
use rand::{Rng, OsRng};
use sha2::{Sha384, Digest};


const B32_ALPHABET: Alphabet = Alphabet::RFC4648{padding:false};

pub fn b32enc(bin: &[u8]) -> String {
    base32::encode(B32_ALPHABET, bin)
}

pub fn b32dec(txt: &str) -> Option<Vec<u8>> {
    base32::decode(B32_ALPHABET, txt)
}


pub fn relpath_2(key: &[u8]) -> PathBuf {
    let b32 = b32enc(key);
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
    b32enc(&key)
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


pub struct Store {
    basedir: PathBuf,
}

impl Store {
    pub fn new<P: AsRef<Path>>(basedir: P) -> Store {
        Store{basedir: PathBuf::from(basedir.as_ref())}
    }

    pub fn remove_tmp_dir(&self) -> Result<(), String> {
        let tmp = self.basedir.join("tmp");
        remove_dir(&tmp).map_err(|err| {
            format!("remove_dir failed {:?}: {}", tmp, err)
        })
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

    fn _write_content(&self, content: &[u8]) -> Result<PathBuf, String> {
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
        Ok(tmp)
    }

    pub fn import_object<P: AsRef<Path>>(&self, src: P) -> Result<[u8; 48], String> {
        let key = {
            let mut file =  File::open(src.as_ref()).map_err(|err| {
                format!("failed to open file {:?}: {}", src.as_ref(), err)
            })?;

            { // Set mode to 0o400
                let mut perm = file.metadata().unwrap().permissions();
                perm.set_mode(0o400);
                file.set_permissions(perm).map_err(|err| {
                    format!("failed to set perms {:?}: {}", src.as_ref(), err)
                })?;
                file.sync_all().map_err(|err| {
                    format!("failed to sync {:?}: {}", src.as_ref(), err)
                })?;
            }

            let mut hasher = Sha384::default();
            let mut buf = [0u8; 4096];
            loop {
                let len = file.read(&mut buf).map_err(|err| {
                    format!("failed to read from {:?}: {}", src.as_ref(), err)
                })?;
                if len == 0 {
                    break;
                }
                hasher.input(&buf[..len]);
            }

            let mut key = [0u8; 48];
            key.copy_from_slice(hasher.result().as_slice());
            key
        };

        let dst = self.object_path(&key);
        to_canonical(src, dst)?;
        Ok(key)
    }

    pub fn write_object(&self, object: &[u8]) -> Result<[u8; 48], String> {
        let key = {
            let mut key = [0u8; 48];
            let digest = Sha384::digest(object);
            key.copy_from_slice(digest.as_slice());
            key
        };
        let tmp = self._write_content(object)?;
        let dst = self.object_path(&key);
        to_canonical(tmp, dst)?;
        Ok(key)
    }

    pub fn open_object(&self, key: &[u8; 48]) -> io::Result<File> {
        File::open(self.object_path(key))
    }

    pub fn write_block(&self, block: &[u8; 400]) -> Result<[u8; 64], String> {
        let sig = {
            let mut sig = [0u8; 64];
            sig.copy_from_slice(&block[0..64]);
            sig
        };
        let tmp = self._write_content(block)?;
        let dst = self.block_path(&sig);
        to_canonical(tmp, dst)?;
        Ok(sig)
    }

    pub fn open_block(&self, sig: &[u8; 64]) -> io::Result<File> {
        File::open(self.block_path(sig))
    }
}


#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};
    use std::fs::{File, create_dir, read_dir};
    use std::io::{Read, Write};
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
    fn test_import_object() {
        let temp_dir = TempDir::new("buildchain-test").unwrap();
        let store = Store::new(&temp_dir);

        let content = {
            let mut content = [0u8; 34969];
            let mut rng = OsRng::new().unwrap();
            rng.fill_bytes(&mut content);
            content
        };

        let example = {
            let artifacts = PathBuf::from(temp_dir.path()).join("artifacts");
            create_dir(artifacts.as_path()).unwrap();
            artifacts.join("example")
        };

        {
            let mut file = File::create(example.as_path()).unwrap();
            file.write_all(&content).unwrap();
        }

        let key: [u8; 48] = store.import_object(example.as_path()).unwrap();

        {
            let mut file = store.open_object(&key).unwrap();

            let perm = file.metadata().unwrap().permissions();
            assert_eq!(perm.mode() & 511, 0o400);

            let mut buf = [0u8; 34969];
            assert_eq!(file.read(&mut buf).unwrap(), buf.len());
            assert_eq!(content.to_vec(), buf.to_vec());
        }

        {
            let tmp = temp_dir.path().join("tmp");
            assert!(! tmp.exists());
        }

        temp_dir.close().unwrap();
    }

    #[test]
    fn test_write_object() {
        let temp_dir = TempDir::new("buildchain-test").unwrap();
        let store = Store::new(&temp_dir);

        let content = {
            let mut content = [0u8; 1776];
            let mut rng = OsRng::new().unwrap();
            rng.fill_bytes(&mut content);
            content
        };
        let key: [u8; 48] = store.write_object(&content).unwrap();

        {
            let mut file = store.open_object(&key).unwrap();

            let perm = file.metadata().unwrap().permissions();
            assert_eq!(perm.mode() & 511, 0o400);

            let mut buf = [0u8; 1776];
            assert_eq!(file.read(&mut buf).unwrap(), buf.len());
            assert_eq!(content.to_vec(), buf.to_vec());
        }

        {
            let tmp = temp_dir.path().join("tmp");
            assert!(tmp.is_dir());
            store.remove_tmp_dir().unwrap();
            assert!(! tmp.exists());
        }

        temp_dir.close().unwrap();
    }

    #[test]
    fn test_write_block() {
        let temp_dir = TempDir::new("buildchain-test").unwrap();
        let store = Store::new(&temp_dir);

        let block = {
            let mut block = [0u8; 400];
            let mut rng = OsRng::new().unwrap();
            rng.fill_bytes(&mut block);
            block
        };

        let sig: [u8; 64] = store.write_block(&block).unwrap();
        assert_eq!(sig.to_vec(), block[0..64].to_vec());

        {
            let mut file = store.open_block(&sig).unwrap();

            let perm = file.metadata().unwrap().permissions();
            assert_eq!(perm.mode() & 511, 0o400);

            let mut buf = [0u8; 400];
            assert_eq!(file.read(&mut buf).unwrap(), buf.len());
            assert_eq!(block.to_vec(), buf.to_vec());
        }

        {
            let tmp = temp_dir.path().join("tmp");
            assert!(tmp.is_dir());
            store.remove_tmp_dir().unwrap();
            assert!(! tmp.exists());
        }

        temp_dir.close().unwrap();
    }
}

