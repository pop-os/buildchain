use plain;
use reqwest;
use serde_json;
use std::fs::File;
use std::io::{stdout, Read, Write};

use {err_str, Block, Manifest, Sha384};
use block::PackedBlock;
use store::b32dec;

pub struct DownloadArguments<'a> {
    pub project: &'a str,
    pub branch: &'a str,
    pub cert_opt: Option<&'a str>,
    pub cache_opt: Option<&'a str>,
    pub key: &'a str,
    pub url: &'a str,
    pub file_opt: Option<&'a str>,
}

struct Downloader {
    key: Vec<u8>,
    url: reqwest::Url,
    project: String,
    branch: String,
    client: reqwest::Client,
}

impl Downloader {
    fn new(key: &str, url: &str, project: &str, branch: &str, cert_opt: Option<&str>) -> Result<Downloader, String> {
        let key = b32dec(key).ok_or(format!("key not in base32 format"))?;

        let url = reqwest::Url::parse(url).map_err(err_str)?;

        let client = {
            let mut builder = reqwest::Client::builder();

            if let Some(cert) = cert_opt {
                let mut buf = Vec::new();
                {
                    let mut file = File::open(cert).map_err(err_str)?;
                    file.read_to_end(&mut buf).map_err(err_str)?;
                }

                builder.add_root_certificate(
                    reqwest::Certificate::from_pem(&buf).map_err(err_str)?
                );
            }

            builder.build().map_err(err_str)?
        };

        Ok(Downloader {
            key: key,
            url: url,
            project: project.to_string(),
            branch: branch.to_string(),
            client: client,
        })
    }

    fn download(&self, path: &str) -> Result<Vec<u8>, String> {
        let url = self.url.join(path).map_err(err_str)?;
        let mut response = self.client.get(url).send().map_err(err_str)?;
        if ! response.status().is_success() {
            return Err(format!("failed to download {}: {:?}", path, response.status()));
        }

        let mut data = Vec::new();
        response.read_to_end(&mut data).map_err(err_str)?;
        Ok(data)
    }

    fn object(&self, digest: &str) -> Result<Vec<u8>, String> {
        let path = format!("object/{}", digest);
        let data = self.download(&path)?;

        let sha = Sha384::new(data.as_slice()).map_err(err_str)?;
        if &sha.to_base32() != digest {
            return Err(format!("sha384 mismatch"));
        }

        Ok(data)
    }

    fn tail(&self) -> Result<Block, String> {
        let path = format!("tail/{}/{}", self.project, self.branch);
        let data = self.download(&path)?;

        let b: &PackedBlock = plain::from_bytes(&data).map_err(|_| format!("response too small"))?;
        b.verify(&self.key)
    }
}

pub fn download<'a>(args: DownloadArguments<'a>) -> Result<(), String> {
    let dl = Downloader::new(args.key, args.url, args.project, args.branch, args.cert_opt)?;

    let tail = dl.tail()?;

    let manifest_json = dl.object(&tail.digest)?;
    let manifest = serde_json::from_slice::<Manifest>(&manifest_json).map_err(err_str)?;

    if let Some(file) = args.file_opt {
        if let Some(digest) = manifest.files.get(file) {
            let data = dl.object(digest)?;
            stdout().write(&data).map_err(err_str)?;
        } else {
            return Err(format!("{} not found", file));
        }
    } else {
        for (file, digest) in manifest.files.iter() {
            println!("{}", file);
        }
    }

    Ok(())
}
