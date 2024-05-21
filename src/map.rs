use core::str;
use std::{
    collections::BTreeMap,
    ffi::{OsStr, OsString},
    path::Path,
};

use futures::future::BoxFuture;
use smol::{
    fs::{self, File},
    io::AsyncReadExt,
    stream::StreamExt,
};

use crate::lang::{OsStrExt, Source};

async fn read_os_string(path: impl AsRef<Path>) -> std::io::Result<Vec<u8>> {
    let mut file = File::open(path).await?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).await?;
    Ok(buffer)
}

pub struct SourceMap {
    sources: BTreeMap<OsString, Source>,
}

impl SourceMap {
    fn new() -> Self {
        Self {
            sources: BTreeMap::new(),
        }
    }
    pub fn get(&self, path: &OsStr) -> Option<&Source> {
        self.sources.get(path)
    }
    pub fn remove(&mut self, path: &OsStr) -> Option<Source> {
        self.sources.remove(path)
    }
    pub async fn from_dir(path: impl AsRef<Path> + Send + 'static) -> SourceMap {
        let mut source_map = Self::new();
        let path = path.as_ref().to_path_buf();
        source_map.load_dir(&path).await;
        source_map
    }
    async fn load_file(&mut self, deps: &[&OsStr], path: &Path) {
        if path.ends_with("Test.java") {
            log::info!("skip test file {:?}", path);
            return;
        }
        log::info!("loading file {:?}", path);
        let content = read_os_string(path).await.expect("failed to read file");
        let mut source = Source::from_bytes(path.file_name().unwrap(), &content);
        deps.iter()
            .for_each(|x| source.insert_import(x.strip_extension()));
        self.sources
            .insert(source.get_path().to_os_string(), source);
    }
    fn load_dir<'a>(&'a mut self, path: &'a Path) -> BoxFuture<'a, ()> {
        Box::pin(async move {
            let entries = fs::read_dir(path).await.expect("not an directory");
            let entries = entries
                .map(|x| x.unwrap())
                .filter(|x| !x.file_name().ends_with("Test.java"))
                .collect::<Vec<_>>()
                .await;
            let deps = entries.iter().map(|x| x.path()).collect::<Vec<_>>();
            let deps = deps
                .iter()
                .map(|x| x.file_name().unwrap())
                .collect::<Vec<_>>();

            for entry in entries {
                let path = entry.path();
                if path.is_dir() {
                    self.load_dir(&path).await;
                } else {
                    if !path.extension().map_or(false, |ext| ext != "java") {
                        self.load_file(&deps, &path).await;
                    }
                }
            }
        })
    }
}
