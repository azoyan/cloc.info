use ordered_multimap::ListOrderedMultimap;
use std::collections::HashMap;

use super::info::RepositoryInfo;

pub(crate) enum Success {
    Rejected(RepositoryInfo),
    Done(Option<RepositoryInfo>),
}

#[derive(Debug, Clone)]
pub(crate) struct RepositoryCache {
    sizes: ListOrderedMultimap<u64, String>,
    urls: HashMap<String, u64>,
    repositories: HashMap<String, RepositoryInfo>,
    limit: usize,
}

impl RepositoryCache {
    pub fn new(limit: usize) -> Self {
        Self {
            sizes: ListOrderedMultimap::with_capacity(limit, limit),
            urls: HashMap::with_capacity(limit),
            repositories: HashMap::with_capacity(limit),
            limit,
        }
    }

    pub(crate) fn get(&self, url: &String) -> Option<&RepositoryInfo> {
        let result = self.repositories.get(url);
        log::debug!(
            "Try to get by url {url}: {result:?}. Cache size = {},{},{}",
            self.repositories.len(),
            self.sizes.values_len(),
            self.urls.len()
        );
        result
    }

    pub fn insert(&mut self, repository: RepositoryInfo) -> Success {
        let url = repository.to_url();
        log::debug!("Attempt to insert: {:?}, name: {}", repository.size, &url);

        // Ищем, есть ли у нас репозиторий с таким именем

        // Существует репозиторий с таким именем. Находим - удаляем его из коллекции размеров и добавляем заново
        let result = if let Some(repository_size) = self.urls.get(&url) {
            log::debug!("NON UNIQUE REPO size: {repository_size},  name: {}", &url);

            // сначала удаляем из коллекции размеров
            self.sizes
                .retain(|size, name| size.ne(repository_size) || url.ne(name));

            // добавляем с другим размером
            self.sizes.append(*repository_size, url.clone());
            // теперь добавляем новый репозиторий в коллекцию репозиториев
            let previous = self.repositories.insert(url.clone(), repository);
            Success::Done(previous)
        }
        // Добавляемый репозиторий уникальный
        else {
            // Если размер коллекции достиг предела
            if self.repositories.len() == self.limit {
                // Ищем самый маленький хранящейся репозиторий
                let (smallest_size, _smallest_name) = self.sizes.front().unwrap();
                // Если добавляемый репозиторий меньше самого маленького - отбрасываем его
                if repository.size.lt(smallest_size) {
                    log::debug!("Rejected size: {smallest_size}, name: {_smallest_name}");
                    Success::Rejected(repository)
                } else {
                    // Удаляемый самый маленький репозиторий
                    let (_smallest_size, smallest_name) = self.sizes.pop_front().unwrap();
                    log::debug!(
                        "Remove smallest, size: {:?}, name: {}",
                        _smallest_size,
                        &smallest_name
                    );
                    self.urls.remove(&smallest_name);
                    let removed = self.repositories.remove(&smallest_name);
                    self.sizes.append(repository.size, url.clone());
                    log::debug!(
                        "Insert size: {}, name: {}. self.sizes keys_len = {},  values_len = {}",
                        repository.size,
                        &url,
                        self.sizes.keys_len(),
                        self.sizes.values_len()
                    );
                    assert_eq!(None, self.urls.insert(url.clone(), repository.size));
                    assert_eq!(None, self.repositories.insert(url.clone(), repository));

                    Success::Done(removed)
                }
            }
            // Просто добавляем новый уникальный репозиторий
            else if self.repositories.len() < self.limit {
                self.sizes.append(repository.size, url.clone());
                assert_eq!(None, self.urls.insert(url.clone(), repository.size));
                assert_eq!(None, self.repositories.insert(url.clone(), repository));

                Success::Done(None)
            } else {
                unreachable!()
            }
        };

        assert_eq!(self.urls.len(), self.repositories.len());
        assert_eq!(self.urls.len(), self.sizes.values_len());
        assert!(self.urls.len() <= self.limit);
        match &result {
            Success::Rejected(_repo) => {}
            Success::Done(_repo) => {
                log::debug!("Insertion Ok. self.size = {}", self.urls.len())
            }
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use tempdir::TempDir;

    use super::RepositoryCache;
    use crate::repository::info::{LocalTempDir, RepositoryInfo};

    #[test]
    fn simple() {
        let mut provider = RepositoryCache::new(10);
        let max = 15_u64;
        for i in 0..max {
            let repository = RepositoryInfo {
                hostname: "github.com".to_string(),
                owner: "her".to_string(),
                repository_name: i.to_string(),
                branch: "master".to_string(),
                last_commit: "her".to_string(),
                local_dir: LocalTempDir::new(TempDir::new("").unwrap()),
                size: i,
                scc_output: vec![],
            };

            let _prev = provider.insert(repository);
        }

        let small_repo = RepositoryInfo {
            hostname: "github.com".to_string(),
            size: 1,
            owner: "her".to_string(),
            repository_name: "her".to_string(),
            branch: "master".to_string(),
            last_commit: "her".to_string(),
            local_dir: LocalTempDir::new(TempDir::new("").unwrap()),
            scc_output: vec![],
        };

        match provider.insert(small_repo.clone()) {
            super::Success::Rejected(rejected) => assert_eq!(rejected, small_repo),
            super::Success::Done(_) => assert!(false),
        }

        let big_repo = RepositoryInfo {
            hostname: "github.com".to_owned(),
            owner: "her".to_owned(),
            repository_name: "her".to_owned(),
            branch: "master".to_owned(),
            size: 10000,
            local_dir: LocalTempDir::new(TempDir::new("").unwrap()),
            last_commit: String::from("her"),
            scc_output: vec![],
        };

        match provider.insert(big_repo.clone()) {
            super::Success::Rejected(_rejected) => assert!(false),
            super::Success::Done(repo) => match repo {
                Some(repo) => assert_eq!(repo.size, 5),
                None => assert!(false),
            },
        }

        let big_repo = RepositoryInfo {
            size: 10000,
            local_dir: LocalTempDir::new(TempDir::new("").unwrap()),
            hostname: "github.com".to_owned(),
            owner: "her".to_string(),
            repository_name: "her2".to_string(),
            branch: "master".to_string(),
            last_commit: String::from("her"),
            scc_output: vec![],
        };

        match provider.insert(big_repo.clone()) {
            super::Success::Rejected(_rejected) => assert!(false),
            super::Success::Done(repo) => match repo {
                Some(repo) => assert_eq!(repo.size, 6),
                None => assert!(false),
            },
        }

        match provider.insert(big_repo.clone()) {
            super::Success::Rejected(_rejected) => assert!(false),
            super::Success::Done(repo) => match repo {
                Some(repo) => assert_eq!(repo.size, 10000),
                None => assert!(false),
            },
        }
    }
}
