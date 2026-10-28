use std::path::{Path, PathBuf};

use crate::error::{Error, Result};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KindleDevice {
    pub root: PathBuf,
    pub clippings_path: Option<PathBuf>,
}

pub fn scanned_locations() -> &'static str {
    if cfg!(target_os = "macos") {
        "/Volumes"
    } else if cfg!(target_os = "linux") {
        "/media、/run/media"
    } else if cfg!(windows) {
        "各盘符"
    } else {
        "常见挂载点"
    }
}

pub fn detect_kindles() -> Vec<KindleDevice> {
    let mut devices: Vec<KindleDevice> = candidate_roots()
        .iter()
        .filter_map(|root| recognize(root))
        .collect();
    devices.sort_by(|a, b| a.root.cmp(&b.root));
    devices.dedup_by(|a, b| a.root == b.root);
    devices
}

pub fn find_kindle_clippings() -> Result<PathBuf> {
    let devices = detect_kindles();
    let clippings: Vec<&Path> = devices
        .iter()
        .filter_map(|device| device.clippings_path.as_deref())
        .collect();

    match clippings.len() {
        1 => Ok(clippings[0].to_path_buf()),
        0 => {
            if devices.is_empty() {
                Err(Error::Device(format!(
                    "未检测到已连接的 Kindle 设备（已扫描：{}）。请用 USB 连接 Kindle 后重试",
                    scanned_locations()
                )))
            } else {
                Err(Error::Device(
                    "检测到 Kindle 设备，但其中没有 documents/My Clippings.txt（设备上可能还没有任何标注/笔记记录）"
                        .into(),
                ))
            }
        }
        _ => {
            let list = clippings
                .iter()
                .map(|path| format!("  - {}", path.display()))
                .collect::<Vec<_>>()
                .join("\n");
            Err(Error::Device(format!(
                "检测到多台 Kindle 设备，请在命令中手动指定要使用的文件：\n{list}"
            )))
        }
    }
}

fn recognize(root: &Path) -> Option<KindleDevice> {
    let documents = root.join("documents");
    let clippings = documents.join("My Clippings.txt");
    if clippings.is_file() {
        return Some(KindleDevice {
            root: root.to_path_buf(),
            clippings_path: Some(clippings),
        });
    }
    if documents.is_dir() && root.join("system").is_dir() {
        return Some(KindleDevice {
            root: root.to_path_buf(),
            clippings_path: None,
        });
    }
    None
}

fn candidate_roots() -> Vec<PathBuf> {
    let mut roots = Vec::new();

    #[cfg(target_os = "macos")]
    roots.extend(read_subdirs(Path::new("/Volumes")));

    #[cfg(target_os = "linux")]
    {
        for base in ["/media", "/run/media"] {
            for first in read_subdirs(Path::new(base)) {
                for second in read_subdirs(&first) {
                    roots.push(second);
                }
                roots.push(first);
            }
        }
    }

    #[cfg(windows)]
    roots.extend(('D'..='Z').map(|letter| PathBuf::from(format!("{letter}:\\"))));

    roots
}

fn read_subdirs(base: &Path) -> Vec<PathBuf> {
    let Ok(entries) = std::fs::read_dir(base) else {
        return Vec::new();
    };
    let mut dirs: Vec<PathBuf> = entries
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.is_dir())
        .filter(|path| {
            !path
                .file_name()
                .map(|name| name.to_string_lossy().starts_with('.'))
                .unwrap_or(false)
        })
        .collect();
    dirs.sort();
    dirs
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kindle_layout(dir: &Path, with_clippings: bool, with_system: bool) {
        std::fs::create_dir_all(dir.join("documents")).unwrap();
        if with_clippings {
            std::fs::write(dir.join("documents/My Clippings.txt"), "x").unwrap();
        }
        if with_system {
            std::fs::create_dir_all(dir.join("system")).unwrap();
        }
    }

    #[test]
    fn recognizes_kindle_with_clippings() {
        let dir = tempfile::tempdir().unwrap();
        kindle_layout(dir.path(), true, true);
        let device = recognize(dir.path()).expect("应识别为 Kindle");
        assert_eq!(
            device.clippings_path,
            Some(dir.path().join("documents/My Clippings.txt"))
        );
    }

    #[test]
    fn recognizes_kindle_without_clippings() {
        let dir = tempfile::tempdir().unwrap();
        kindle_layout(dir.path(), false, true);
        let device = recognize(dir.path()).expect("应识别为无剪辑文件的 Kindle");
        assert!(device.clippings_path.is_none());
    }

    #[test]
    fn ignores_ordinary_directories() {
        let dir = tempfile::tempdir().unwrap();
        kindle_layout(dir.path(), false, false);
        assert!(recognize(dir.path()).is_none());
    }
}
