use std::path::PathBuf;

/// `Pool` holds data on the file pool the server has to maintain.
#[derive(Debug, Default, Clone)]
pub struct Pool {
    /// `paths` holds a list of all paths
    pub(crate) paths: Vec<PathBuf>,
}

impl From<&Vec<String>> for Pool {
    /// Build a `Pool` from a list of paths.
    fn from(files: &Vec<String>) -> Self {
        // If files are empty, we return
        // the default implementation of the pool.
        if files.is_empty() {
            log::warn!("no `--watch` argument found in start command. No files will be watched until receiving an order.");
            return Pool::default();
        }

        log::info!("creating pool from {} path(s).", &files.len());

        let mut paths = Vec::<PathBuf>::new();

        for path in files {
            log::debug!("trying to parse path as glob. path={}", &path);

            if let Ok(glob) = glob::glob(path) {
                for entry in glob {
                    match entry {
                        Ok(file) => {
                            log::debug!(
                                "adding path to pool. path={}, glob={}",
                                &file.display(),
                                &path
                            );
                            paths.push(file)
                        }
                        Err(e) => {
                            log::warn!("failed to decode the glob format with error={}", e)
                        }
                    }
                }
            }
        }

        Self { paths }
    }
}

#[cfg(test)]
mod tests {
    use crate::watcher::pool::Pool;
    use std::fs::{remove_dir_all, File};
    use tempfile::tempdir;

    #[test]
    fn test_it_parse_pool_correctly() {
        let tmp = tempdir().expect("Failed to create temporary directory");

        let f1_path = tmp.path().join("test.log");
        let f2_path = tmp.path().join("test.yml");

        File::create(f1_path).expect("Failed to create 'test.log' file");
        File::create(f2_path).expect("Failed to create 'test.yml' file");

        let pool = Pool::from(&vec![
            format!("{}/**/*.log", tmp.path().display()),
            format!("{}/**/*.yml", tmp.path().display()),
        ]);

        assert_eq!(pool.paths.len(), 2);

        remove_dir_all(tmp.path()).expect("Failed to clean up test folder")
    }
}
