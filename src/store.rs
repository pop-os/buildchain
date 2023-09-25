// SPDX-License-Identifier: GPL-3.0-only

use std::collections::BTreeMap;
use std::fs::{create_dir, read_dir, remove_dir, rename, File, OpenOptions};
use std::io::{self, Read, Write};
use std::os::unix::fs::{symlink, OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};

use base32::{self, Alphabet};
use rand::rngs::OsRng;
use rand::RngCore;
use sha2::{Digest, Sha384};

use crate::Manifest;

const B32_ALPHABET: Alphabet = Alphabet::RFC4648 { padding: false };

pub fn b32enc(bin: &[u8]) -> String {
    base32::encode(B32_ALPHABET, bin)
}

pub fn b32dec(txt: &str) -> Option<Vec<u8>> {
    base32::decode(B32_ALPHABET, txt)
}

fn block_relpath(sig: &[u8; 64]) -> PathBuf {
    PathBuf::from("block").join(b32enc(sig))
}

fn object_relpath(key: &[u8; 48]) -> PathBuf {
    PathBuf::from("object").join(b32enc(key))
}

/* tail/PROJECT/BRANCH --> ../../block/B32SIGNATURE */
fn tail_to_block(sig: &[u8; 64]) -> PathBuf {
    PathBuf::from("../..").join(block_relpath(sig))
}

pub fn random_id() -> String {
    let mut key = [0u8; 15];
    OsRng.fill_bytes(&mut key);
    b32enc(&key)
}

fn create_dir_if_needed<P: AsRef<Path>>(path: P) -> io::Result<()> {
    if path.as_ref().is_dir() {
        return Ok(());
    }
    create_dir(path.as_ref())
}

fn to_canonical<P: AsRef<Path>, Q: AsRef<Path>>(src: P, dst: Q) -> io::Result<()> {
    let parent = dst.as_ref().parent().unwrap();
    create_dir_if_needed(parent)?;
    rename(src.as_ref(), dst.as_ref())
}

pub struct Store {
    basedir: PathBuf,
}

impl Store {
    pub fn new<P: AsRef<Path>>(basedir: P) -> Store {
        Store {
            basedir: PathBuf::from(basedir.as_ref()),
        }
    }

    pub fn remove_tmp_dir(&self) -> io::Result<()> {
        let tmp = self.basedir.join("tmp");
        remove_dir(tmp)
    }

    pub fn temp_path(&self) -> PathBuf {
        self.basedir.join("tmp").join(random_id())
    }

    pub fn object_path(&self, key: &[u8; 48]) -> PathBuf {
        self.basedir.join(object_relpath(key))
    }

    pub fn block_path(&self, sig: &[u8; 64]) -> PathBuf {
        self.basedir.join(block_relpath(sig))
    }

    fn _write_content(&self, content: &[u8]) -> io::Result<PathBuf> {
        let tmp = self.temp_path();
        create_dir_if_needed(tmp.parent().unwrap())?;
        {
            let mut opt = OpenOptions::new();
            let opt = opt.create_new(true).write(true).mode(0o400);
            let mut file = opt.open(tmp.as_path())?;
            file.write_all(content)?;
            file.sync_all()?;
        }
        Ok(tmp)
    }

    pub fn import_object<P: AsRef<Path>>(&self, src: P) -> io::Result<[u8; 48]> {
        let key = {
            let mut file = File::open(src.as_ref())?;

            {
                // Set mode to 0o400
                let mut perm = file.metadata().unwrap().permissions();
                perm.set_mode(0o400);
                file.set_permissions(perm)?;
                file.sync_all()?;
            }

            let mut hasher = Sha384::default();
            let mut buf = [0u8; 4096];
            loop {
                let len = file.read(&mut buf)?;
                if len == 0 {
                    break;
                }
                hasher.update(&buf[..len]);
            }

            let mut key = [0u8; 48];
            key.copy_from_slice(hasher.finalize().as_slice());
            key
        };

        let dst = self.object_path(&key);
        to_canonical(src, dst)?;
        Ok(key)
    }

    pub fn import_artifacts(&self, time: u64) -> io::Result<Manifest> {
        let artifacts = self.basedir.join("artifacts");
        let mut files = BTreeMap::new();

        let entries = read_dir(artifacts.as_path())?;
        for entry in entries {
            let entry = entry?;

            let name = entry
                .file_name()
                .into_string()
                .map_err(|err| io::Error::new(io::ErrorKind::InvalidInput, format!("{:?}", err)))?;

            let key = self.import_object(entry.path())?;

            files.insert(name, b32enc(&key[..]));

            let target = PathBuf::from("..").join(object_relpath(&key));
            let link = entry.path();
            symlink(target.as_path(), link.as_path())?;
        }

        Ok(Manifest { time, files })
    }

    pub fn write_object(&self, object: &[u8]) -> io::Result<[u8; 48]> {
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

    pub fn write_manifest(&self, object: &[u8]) -> io::Result<[u8; 48]> {
        let key = self.write_object(object)?;
        let link = self.basedir.join("manifest.json");
        let target = object_relpath(&key);
        symlink(target.as_path(), link.as_path())?;
        Ok(key)
    }

    pub fn open_object(&self, key: &[u8; 48]) -> io::Result<File> {
        File::open(self.object_path(key))
    }

    pub fn write_block(&self, block: &[u8; 400]) -> io::Result<[u8; 64]> {
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

    pub fn write_tail(
        &self,
        project: &str,
        branch: &str,
        block: &[u8; 400],
    ) -> io::Result<[u8; 64]> {
        let sig = self.write_block(block)?;
        let mut pb = self.basedir.join("tail");
        create_dir_if_needed(&pb)?;
        pb.push(project);
        create_dir_if_needed(&pb)?;
        pb.push(branch);
        let target = tail_to_block(&sig);
        symlink(target.as_path(), pb.as_path())?;
        Ok(sig)
    }

    pub fn open_block(&self, sig: &[u8; 64]) -> io::Result<File> {
        File::open(self.block_path(sig))
    }
}

#[cfg(test)]
mod tests {
    use std::fs::{create_dir, File};
    use std::io::{Read, Write};
    use std::os::unix::fs::PermissionsExt;
    use std::path::{Path, PathBuf};

    use rand::{rngs::OsRng, RngCore};
    use tempdir::TempDir;

    use super::{tail_to_block, Store};

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
            Path::new("/p/object/AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA")
        );
        assert_eq!(
            s.object_path(&[255u8; 48]).as_path(),
            Path::new("/p/object/77777777777777777777777777777777777777777777777777777777777777777777777777776")
        );
    }

    #[test]
    fn test_block_path() {
        let s = Store::new(Path::new("/p"));
        let sig1 = [0u8; 64];
        let sig2 = [255u8; 64];
        assert_eq!(
            s.block_path(&sig1).as_path(),
            Path::new("/p/block/AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA")
        );
        assert_eq!(
            s.block_path(&sig2).as_path(),
            Path::new("/p/block/777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777Y")
        );
    }

    #[test]
    fn test_import_object() {
        let temp_dir = TempDir::new("buildchain-test").unwrap();
        let store = Store::new(&temp_dir);

        let content = {
            let mut content = [0u8; 34969];
            OsRng.fill_bytes(&mut content);
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
            assert!(!tmp.exists());
        }

        temp_dir.close().unwrap();
    }

    #[test]
    fn test_write_object() {
        let temp_dir = TempDir::new("buildchain-test").unwrap();
        let store = Store::new(&temp_dir);

        let content = {
            let mut content = [0u8; 1776];
            OsRng.fill_bytes(&mut content);
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
            assert!(!tmp.exists());
        }

        temp_dir.close().unwrap();
    }

    #[test]
    fn test_write_block() {
        let temp_dir = TempDir::new("buildchain-test").unwrap();
        let store = Store::new(&temp_dir);

        let block = {
            let mut block = [0u8; 400];
            OsRng.fill_bytes(&mut block);
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
            assert!(!tmp.exists());
        }

        temp_dir.close().unwrap();
    }

    #[test]
    fn test_write_tail() {
        let temp_dir = TempDir::new("buildchain-test").unwrap();
        let store = Store::new(&temp_dir);

        let block = {
            let mut block = [0u8; 400];
            OsRng.fill_bytes(&mut block);
            block
        };

        let sig = store.write_tail("stuff", "junk", &block).unwrap();
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
            let mut pb = temp_dir.path().join("tail");
            assert!(pb.is_dir());
            pb.push("stuff");
            assert!(pb.is_dir());
            pb.push("junk");
            assert!(pb.is_file());
            assert_eq!(pb.read_link().unwrap(), tail_to_block(&sig));
        }

        {
            let tmp = temp_dir.path().join("tmp");
            assert!(tmp.is_dir());
        }

        temp_dir.close().unwrap();
    }
}
