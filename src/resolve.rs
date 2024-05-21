use std::path::Path;

use crate::{lang::OsStrExt, map::SourceMap};

pub async fn resolve(path: impl AsRef<Path> + Send + 'static) -> Vec<u8> {
    let dir_path = path.as_ref().parent().unwrap().to_path_buf();

    let mut map = SourceMap::from_dir(dir_path).await;

    let target_import = path.as_ref().file_name().unwrap().strip_extension();

    let mut exports = vec![target_import.to_os_string()];
    let mut dfs = vec![target_import];
    while let Some(import) = dfs.pop() {
        if let Some(source) = map.get(import) {
            for import in source.get_import_files() {
                let import_owned = import.to_os_string();
                if !exports.contains(&import_owned) {
                    exports.push(import_owned.to_os_string());
                    dfs.push(import);
                } else {
                    log::trace!("skipping import {:?}", import_owned);
                }
            }
        } else {
            log::warn!("failed to find source for {:?}", import);
        }
    }

    let mut import_content = Vec::new();

    let mut content = Vec::new();
    for export in exports {
        if let Some(mut source) = map.remove(&export) {
            content.extend(source.into_content());
            import_content.extend(source.get_import_content());
        } else {
            log::warn!("failed to find source for {:?}", export);
        }
    }

    [import_content, content].concat()
}
