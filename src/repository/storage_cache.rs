use std::sync::Arc;
use tempfile::TempDir;

pub enum Success {
    Rejected(Arc<TempDir>),
    Done(Option<Arc<TempDir>>),
}
type Url = String;

#[derive(Debug, Clone)]
struct TemporaryDirectory(Arc<TempDir>);

#[derive(Debug, Clone)]
struct LocalDir {
    size: u64,
    url: Url,
    directory: Arc<TempDir>,
}

#[derive(Debug, Clone)]
pub struct StorageCache {
    storage: Vec<LocalDir>,
    size: u64,
    capacity: u64,
}

impl StorageCache {
    pub fn new(limit: u64) -> Self {
        Self {
            storage: Default::default(),
            capacity: limit,
            size: 0,
        }
    }

    pub fn take(&mut self, url: &str) -> Option<Arc<TempDir>> {
        match self.storage.iter().position(|el| el.url.eq(url)) {
            Some(pos) => {
                let local_dir = self.storage.swap_remove(pos);
                self.size -= local_dir.size;

                self.storage.sort_by(|el1, el2| el2.size.cmp(&el1.size));
                Some(local_dir.directory)
            }
            None => None,
        }
    }

    pub fn insert(&mut self, url: &str, repository_directory: Arc<TempDir>) -> Success {
        let repository_size = fs_extra::dir::get_size(repository_directory.path()).unwrap();
        if self.storage.iter().any(|el| el.url == url) {
            panic!("Repository must be unique!");
        }

        if repository_size > self.capacity {
            tracing::debug!(
                "Rejected newcomer {url} ({repository_size}) greater than capacity {}",
                self.capacity
            );
            return Success::Rejected(repository_directory);
        }

        if self.capacity >= (self.size + repository_size) {
            tracing::debug!(
                "Storage Insertion {url} ({}/{}/{})",
                repository_size,
                self.size,
                self.capacity
            );
            self.storage.push(LocalDir {
                size: repository_size,
                url: url.to_string(),
                directory: repository_directory.clone(),
            });
            self.size += repository_size;

            // greaters at beign, smallest at end
            self.storage.sort_by(|el1, el2| el2.size.cmp(&el1.size));

            Success::Done(Some(repository_directory))
        } else {
            if let Some(poped) = self.storage.pop() {
                if poped.size > repository_size {
                    tracing::debug!(
                        "Rejected newcomer {url} ({repository_size}) smaller than last {} ({})",
                        poped.url,
                        poped.size
                    );
                    self.storage.push(poped);
                } else {
                    self.size -= poped.size;
                    tracing::debug!(
                        "Pop from stoage {} ({}/{}/{})",
                        poped.url,
                        poped.size,
                        self.size,
                        self.capacity
                    );
                    return self.insert(url, repository_directory);
                }
            } else {
                tracing::debug!("Storage cache is Empty");
            }
            Success::Rejected(repository_directory)
        }
    }
}

impl Drop for StorageCache {
    fn drop(&mut self) {
        tracing::debug!("Storage cahce droped");
    }
}
