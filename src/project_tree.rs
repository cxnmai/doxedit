use gpui::SharedString;
use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};

pub struct ProjectTree {
    pub root: PathBuf,
    pub entries: Vec<TreeEntry>,
}

impl ProjectTree {
    pub fn load(root: PathBuf) -> Self {
        let entries = read_tree_entries(&root).unwrap_or_default();
        Self { root, entries }
    }

    pub fn root_name(&self) -> String {
        self.root
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("Project")
            .to_string()
    }

    pub fn folder_paths(&self) -> HashSet<PathBuf> {
        let mut paths = HashSet::new();

        for entry in &self.entries {
            collect_folder_paths(entry, &mut paths);
        }

        paths
    }
}

pub enum TreeEntry {
    Folder {
        name: SharedString,
        path: PathBuf,
        children: Vec<TreeEntry>,
    },
    File {
        name: SharedString,
        path: PathBuf,
    },
}

fn read_tree_entries(root: &Path) -> std::io::Result<Vec<TreeEntry>> {
    let mut entries = Vec::new();

    for entry in std::fs::read_dir(root)? {
        let entry = entry?;
        let path = entry.path();
        let file_name = entry.file_name();
        let name = file_name.to_string_lossy();

        if should_skip_path_name(&name) {
            continue;
        }

        if path.is_dir() {
            let children = read_tree_entries(&path)?;

            entries.push(TreeEntry::Folder {
                name: name.to_string().into(),
                path,
                children,
            });
        } else if is_db8_file(&path) {
            entries.push(TreeEntry::File {
                name: name.to_string().into(),
                path,
            });
        }
    }

    entries.sort_by(|left, right| match (left, right) {
        (TreeEntry::Folder { name: left, .. }, TreeEntry::Folder { name: right, .. })
        | (TreeEntry::File { name: left, .. }, TreeEntry::File { name: right, .. }) => {
            left.cmp(right)
        }
        (TreeEntry::Folder { .. }, TreeEntry::File { .. }) => std::cmp::Ordering::Less,
        (TreeEntry::File { .. }, TreeEntry::Folder { .. }) => std::cmp::Ordering::Greater,
    });

    Ok(entries)
}

fn collect_folder_paths(entry: &TreeEntry, paths: &mut HashSet<PathBuf>) {
    if let TreeEntry::Folder { path, children, .. } = entry {
        paths.insert(path.clone());

        for child in children {
            collect_folder_paths(child, paths);
        }
    }
}

fn should_skip_path_name(name: &str) -> bool {
    name.starts_with('.') || matches!(name, "target" | "node_modules" | "dist" | "build")
}

fn is_db8_file(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("db8"))
}
