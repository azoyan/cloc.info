use std::process::Command;

use super::utils;

pub enum Success {
    Rejected(String),
    Done(Option<String>),
}

#[derive(Debug, Clone)]
struct LocalDir {
    size: u64,
    path: String,
}

#[derive(Debug)]
pub struct DiskCache {
    storage: Vec<LocalDir>,
    size: u64,
    capacity: u64,
}

impl DiskCache {
    pub fn new(limit: u64) -> Self {
        Self {
            storage: Default::default(),
            capacity: limit,
            size: 0,
        }
    }

    pub fn contains(&self, path: &str) -> bool {
        self.storage.iter().any(|d| d.path.eq(path))
    }

    pub fn insert(&mut self, path: &str) -> Success {
        loop {
            let repository_size = utils::dir_size(path).unwrap();
            {
                if self.storage.iter().any(|el| el.path == path) {
                    panic!("Repository must be unique!");
                }
            }

            if repository_size > self.capacity {
                tracing::debug!(
                    "Rejected newcomer {path} ({repository_size}) greater than capacity {}",
                    self.capacity
                );
                return Success::Rejected(path.to_string());
            }

            if self.capacity >= (self.size + repository_size) {
                tracing::debug!(
                    "Storage Insertion {path} ({}/{}/{})",
                    repository_size,
                    self.size,
                    self.capacity
                );
                let result = Some(path.to_string());
                let dir = LocalDir {
                    size: repository_size,
                    path: path.to_string(),
                };
                self.size += repository_size;

                self.storage.push(dir);
                // greatest at beign, smallest at end
                self.storage.sort_by(|el1, el2| el2.size.cmp(&el1.size));

                return Success::Done(result);
            } else {
                if let Some(to_remove) = self.storage.last() {
                    if to_remove.size > repository_size {
                        tracing::debug!(
                        "Rejected: Newcomer {path} ({repository_size}) smaller than last {} ({})",
                        to_remove.path,
                        to_remove.size
                    );
                    } else {
                        self.size -= to_remove.size;
                        tracing::debug!(
                            "Remove from disk {} ({}/{}/{})",
                            to_remove.path,
                            to_remove.size,
                            self.size,
                            self.capacity
                        );
                        self.remove(&to_remove.path).unwrap();
                        continue;
                    }
                } else {
                    tracing::debug!("Storage cache is Empty");
                }
                return Success::Rejected(path.to_string());
            }
        }
    }

    pub fn remove(&self, path: &str) -> Result<(), std::io::Error> {
        let empty_dir = format!("empty_{path}");
        assert!(Command::new("mkdir")
            .args(["-p", &empty_dir])
            .output()?
            .status
            .success());
        assert!(Command::new("rsync")
            .args(["-a", "--delete", &empty_dir, path])
            .output()?
            .status
            .success());
        assert!(Command::new("rmdir")
            .arg(&empty_dir)
            .output()?
            .status
            .success());

        Ok(())
    }
}

impl Drop for DiskCache {
    fn drop(&mut self) {
        tracing::debug!("Storage cahce droped");
    }
}
