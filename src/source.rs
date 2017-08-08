use git2::{self, Repository};
use std::io;
use std::path::Path;

fn git_err(err: git2::Error) -> io::Error {
    io::Error::new(
        io::ErrorKind::Other,
        format!("Git error: {}", err)
    )
}

/// A source code repository
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct Source {
    pub kind: String,
    pub url: String,
}

impl Source {
    /// Download the source code repository to the given directory
    //TODO: More documentation, code example
    pub fn download<P: AsRef<Path>>(&self, directory: P) -> io::Result<u64> {
        match self.kind.as_str() {
            "git" => {
                let repo = Repository::clone(&self.url, directory).map_err(git_err)?;

                let mut walk = repo.revwalk().map_err(git_err)?;
                walk.set_sorting(git2::SORT_TIME);
                walk.push_head().map_err(git_err)?;

                if let Some(id_res) = walk.next() {
                    let id = id_res.map_err(git_err)?;
                    let commit = repo.find_commit(id).map_err(git_err)?;
                    let time = commit.time();
                    Ok(time.seconds() as u64)
                } else {
                    Err(io::Error::new(
                        io::ErrorKind::Other,
                        format!("Git error: no commits found")
                    ))
                }
            },
            _ => {
                Err(io::Error::new(
                    io::ErrorKind::Other,
                    format!("Unknown source kind: {}", self.kind)
                ))
            }
        }
    }
}
