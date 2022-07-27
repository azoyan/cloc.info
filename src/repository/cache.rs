use ordered_multimap::ListOrderedMultimap;
use std::{collections::HashMap, sync::Arc};
use tempfile::TempDir;

pub(crate) enum Success {
    Rejected(Arc<TempDir>),
    Done(Option<Arc<TempDir>>),
}
type RepositortSize = u64;
type Url = String;

#[derive(Debug, Clone)]
struct TemporaryDirectory(Arc<TempDir>);

#[derive(Debug, Clone)]
pub(crate) struct RepositoryCache {
    sizes: ListOrderedMultimap<RepositortSize, Url>,
    urls: HashMap<Url, RepositortSize>,
    repositories: HashMap<Url, Arc<TempDir>>,
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

    pub(crate) fn get(&self, path: &String) -> Option<&Arc<TempDir>> {
        let result = self.repositories.get(path);
        tracing::debug!(
            "Get repository {path}: {result:?}. Cache size = {},{},{}",
            self.repositories.len(),
            self.sizes.values_len(),
            self.urls.len()
        );
        result
    }

    pub fn insert(&mut self, repository: Arc<TempDir>) -> Success {
        let url = repository.path().as_os_str().to_str().unwrap();
        let repository_size = fs_extra::dir::get_size(repository.path()).unwrap();
        tracing::debug!(
            "Attempt to insert to storage cache: {:?}, name: {}",
            repository_size,
            url
        );

        // Ищем, есть ли у нас репозиторий с таким именем

        // Существует репозиторий с таким именем. Находим - удаляем его из коллекции размеров и добавляем заново
        let result = if let Some(repository_size) = self.urls.get(url) {
            tracing::debug!("NON UNIQUE REPO size: {repository_size},  name: {}", url);

            // сначала удаляем из коллекции размеров
            self.sizes
                .retain(|size, name| size.ne(repository_size) || url.ne(name));

            // добавляем с другим размером
            self.sizes.append(*repository_size, url.to_string());
            // теперь добавляем новый репозиторий в коллекцию репозиториев
            let previous = self.repositories.insert(url.to_string(), repository);
            Success::Done(previous)
        }
        // Добавляемый репозиторий уникальный
        else {
            match self.repositories.len().cmp(&self.limit) {
                std::cmp::Ordering::Less => {
                    self.sizes.append(repository_size, url.to_string());
                    assert_eq!(None, self.urls.insert(url.to_string(), repository_size));
                    assert!(self
                        .repositories
                        .insert(url.to_string(), repository)
                        .is_none());

                    Success::Done(None)
                }
                std::cmp::Ordering::Equal => {
                    // Ищем самый маленький хранящейся репозиторий
                    let (smallest_size, _smallest_name) = self.sizes.front().unwrap();
                    // Если добавляемый репозиторий меньше самого маленького - отбрасываем его
                    if repository_size.lt(smallest_size) {
                        tracing::debug!("Rejected size: {smallest_size}, name: {_smallest_name}");
                        Success::Rejected(repository)
                    } else {
                        // Удаляемый самый маленький репозиторий
                        let (_smallest_size, smallest_name) = self.sizes.pop_front().unwrap();
                        tracing::debug!(
                            "Remove smallest, size: {:?}, name: {}",
                            _smallest_size,
                            &smallest_name
                        );
                        self.urls.remove(&smallest_name);
                        let removed = self.repositories.remove(&smallest_name);
                        self.sizes.append(repository_size, url.to_string());
                        tracing::debug!(
                            "Insert size: {}, name: {}. self.sizes keys_len = {},  values_len = {}",
                            repository_size,
                            &url,
                            self.sizes.keys_len(),
                            self.sizes.values_len()
                        );
                        assert_eq!(None, self.urls.insert(url.to_string(), repository_size));
                        assert!(self
                            .repositories
                            .insert(url.to_string(), repository)
                            .is_none());

                        Success::Done(removed)
                    }
                }
                std::cmp::Ordering::Greater => unreachable!(),
            }
        };

        assert_eq!(self.urls.len(), self.repositories.len());
        assert_eq!(self.urls.len(), self.sizes.values_len());
        assert!(self.urls.len() <= self.limit);
        match &result {
            Success::Rejected(_repo) => {}
            Success::Done(_repo) => {
                tracing::debug!("Insertion Ok. self.size = {}", self.urls.len())
            }
        }
        result
    }
}

// #[cfg(test)]
// mod tests {
//     use tempdir::TempDir;

//     use super::RepositoryCache;
//     use crate::repository::info::{LocalTempDir, RepositoryInfo};

//     #[test]
//     fn simple() {
//         let mut provider = RepositoryCache::new(10);
//         let max = 15_u64;
//         for i in 0..max {
//             let repository = RepositoryInfo {
//                 hostname: "github.com".to_string(),
//                 owner: "her".to_string(),
//                 repository_name: i.to_string(),
//                 branch: "master".to_string(),
//                 last_commit: "her".to_string(),
//                 local_dir: LocalTempDir::new(TempDir::new("").unwrap()),
//                 size: i,
//                 scc_output: vec![],
//             };

//             let _prev = provider.insert(repository);
//         }

//         let small_repo = RepositoryInfo {
//             hostname: "github.com".to_string(),
//             size: 1,
//             owner: "her".to_string(),
//             repository_name: "her".to_string(),
//             branch: "master".to_string(),
//             last_commit: "her".to_string(),
//             local_dir: LocalTempDir::new(TempDir::new("").unwrap()),
//             scc_output: vec![],
//         };

//         match provider.insert(small_repo.clone()) {
//             super::Success::Rejected(rejected) => assert_eq!(rejected, small_repo),
//             super::Success::Done(_) => assert!(false),
//         }

//         let big_repo = RepositoryInfo {
//             hostname: "github.com".to_owned(),
//             owner: "her".to_owned(),
//             repository_name: "her".to_owned(),
//             branch: "master".to_owned(),
//             size: 10000,
//             local_dir: LocalTempDir::new(TempDir::new("").unwrap()),
//             last_commit: String::from("her"),
//             scc_output: vec![],
//         };

//         match provider.insert(big_repo.clone()) {
//             super::Success::Rejected(_rejected) => assert!(false),
//             super::Success::Done(repo) => match repo {
//                 Some(repo) => assert_eq!(repo.size, 5),
//                 None => assert!(false),
//             },
//         }

//         let big_repo = RepositoryInfo {
//             size: 10000,
//             local_dir: LocalTempDir::new(TempDir::new("").unwrap()),
//             hostname: "github.com".to_owned(),
//             owner: "her".to_string(),
//             repository_name: "her2".to_string(),
//             branch: "master".to_string(),
//             last_commit: String::from("her"),
//             scc_output: vec![],
//         };

//         match provider.insert(big_repo.clone()) {
//             super::Success::Rejected(_rejected) => assert!(false),
//             super::Success::Done(repo) => match repo {
//                 Some(repo) => assert_eq!(repo.size, 6),
//                 None => assert!(false),
//             },
//         }

//         match provider.insert(big_repo.clone()) {
//             super::Success::Rejected(_rejected) => assert!(false),
//             super::Success::Done(repo) => match repo {
//                 Some(repo) => assert_eq!(repo.size, 10000),
//                 None => assert!(false),
//             },
//         }
//     }
// }
