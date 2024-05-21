use std::{
    ffi::{OsStr, OsString},
    fmt::Display,
    mem,
    os::unix::ffi::OsStrExt as _,
};
pub trait OsStrExt {
    fn strip_extension<'a>(&'a self) -> &'a OsStr;
    fn strip_suffix<'a>(&'a self, suffix: &str) -> &'a OsStr;
    fn strip_prefix<'a>(&'a self, prefix: &str) -> &'a OsStr;
    fn ends_with(&self, suffix: &str) -> bool;
    fn starts_with(&self, prefix: &str) -> bool;
    fn trim_start(&self) -> &OsStr;
}

impl OsStrExt for OsStr {
    fn strip_extension<'a>(&'a self) -> &'a OsStr {
        let s = self.as_bytes();
        if let Some(pos) = s.iter().rposition(|&x| x == b'.') {
            OsStr::from_bytes(&s[..pos])
        } else {
            OsStr::from_bytes(s)
        }
    }
    fn strip_suffix<'a>(&'a self, suffix: &str) -> &'a OsStr {
        let s = self.as_bytes();
        let suffix = suffix.as_bytes();
        if s.ends_with(suffix) {
            OsStr::from_bytes(&s[..s.len() - suffix.len()])
        } else {
            OsStr::from_bytes(s)
        }
    }
    fn strip_prefix<'a>(&'a self, prefix: &str) -> &'a OsStr {
        let s = self.as_bytes();
        let prefix = prefix.as_bytes();
        if s.starts_with(prefix) {
            OsStr::from_bytes(&s[prefix.len()..])
        } else {
            OsStr::from_bytes(s)
        }
    }
    fn ends_with(&self, suffix: &str) -> bool {
        self.as_bytes().ends_with(suffix.as_bytes())
    }
    fn starts_with(&self, prefix: &str) -> bool {
        self.as_bytes().starts_with(prefix.as_bytes())
    }
    fn trim_start(&self) -> &OsStr {
        let s = self.as_bytes();
        let start = s.iter().position(|&x| x != b' ').unwrap_or(0);
        OsStr::from_bytes(&s[start..])
    }
}

pub enum LineAST<'a> {
    StaticImport(&'a OsStr),
    Import(&'a OsStr),
    Package(&'a OsStr),
    Decorater,
    Comment,
    Other(&'a [u8]),
    Visibility(&'a OsStr),
}

impl<'a> Display for LineAST<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                LineAST::StaticImport(x) => "static import",
                LineAST::Import(x) => "import",
                LineAST::Package(x) => "package",
                LineAST::Decorater => "decorater",
                LineAST::Comment => "comment",
                LineAST::Other(x) => "other",
                LineAST::Visibility(x) => "visibility",
            }
        )
    }
}

impl<'a> LineAST<'a> {
    pub fn from_line(line: &'a [u8]) -> LineAST<'a> {
        let get_token = |n: usize| {
            OsStr::from_bytes(line.split(|&c| c == b' ').nth(n).expect("token not found"))
                .strip_suffix(";")
        };
        if line.starts_with(b"import static") {
            return LineAST::StaticImport(get_token(2));
        } else if line.starts_with(b"import") {
            return LineAST::Import(get_token(1));
        } else if line.starts_with(b"package") {
            return LineAST::Package(get_token(1));
        } else if OsStr::from_bytes(line).trim_start().starts_with("@") {
            return LineAST::Decorater;
        } else if OsStr::from_bytes(line).trim_start().starts_with("//") {
            return LineAST::Comment;
        } else if OsStr::from_bytes(line)
            .trim_start()
            .starts_with("public interface")
            || OsStr::from_bytes(line)
                .trim_start()
                .starts_with("public class")
            || OsStr::from_bytes(line)
                .trim_start()
                .starts_with("public enum")
        {
            return LineAST::Visibility(
                OsStr::from_bytes(line).trim_start().strip_prefix("public"),
            );
        }
        LineAST::Other(line)
    }
}
pub struct Source {
    module_path: OsString,
    content: Vec<u8>,
    foreign_imports: Vec<u8>,
    local_imports: Vec<OsString>,
}

impl Source {
    pub fn get_path(&self) -> &OsStr {
        &self.module_path
    }
    pub fn get_import_files<'a>(&'a self) -> impl Iterator<Item = &OsStr> + 'a {
        self.local_imports.iter().map(|x| x.as_os_str())
    }
    pub fn get_import_content(&mut self) -> Vec<u8> {
        mem::take(&mut self.foreign_imports)
    }
    pub fn into_content(&mut self) -> Vec<u8> {
        mem::take(&mut self.content)
    }
    pub fn insert_import(&mut self, import: &OsStr) {
        let module_path = self
            .module_path
            .as_bytes()
            .split(|x| *x == b'.')
            .last()
            .unwrap();
        let parent = [
            self.module_path
                .as_bytes()
                .strip_suffix(module_path)
                .unwrap(),
            import.as_bytes(),
        ]
        .concat();
        self.local_imports
            .push(OsStr::from_bytes(&parent).to_os_string());
    }
    pub fn from_bytes(module_path: &OsStr, raw_content: &[u8]) -> Self {
        let module_name = module_path.strip_suffix(".java");

        let mut local_imports = Vec::new();
        let mut foreign_imports = Vec::new();

        let mut content = Vec::new();

        let mut module_path = OsStr::from_bytes(b"").to_os_string();

        for raw in raw_content.split(|&c| c == b'\n') {
            let line = LineAST::from_line(raw);
            log::trace!("get line kind: {}", line);
            match line {
                LineAST::StaticImport(x) | LineAST::Import(x) => {
                    if !x.starts_with("java") {
                        local_imports.push(x.to_os_string());
                    } else {
                        foreign_imports.extend_from_slice(raw);
                    }
                }
                LineAST::Package(x) => {
                    module_path = x.to_os_string();
                    module_path.push(OsStr::from_bytes(b"."));
                }
                LineAST::Other(_) => {
                    content.extend_from_slice(raw);
                }
                LineAST::Visibility(x) => {
                    content.extend_from_slice(x.as_bytes());
                }
                _ => {}
            }
        }

        module_path.push(module_name);

        log::debug!("module path: {:?}", module_path);

        Self {
            module_path,
            content,
            foreign_imports,
            local_imports,
        }
    }
}
