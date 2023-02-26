use crate::unwrap_or_continue;
use std::path::{self, Path, PathBuf};

// search for a file by listing parent folders until find or hit home, this will not enter in any
// folder for a deep list
pub fn search_file_backwards(base_path: &Path, rule: fn(&Path) -> bool) -> Option<PathBuf> {
    for f in base_path.read_dir().ok()? {
        let p = match f {
            Ok(v) => v.path(),
            _ => continue,
        };

        if rule(&p) {
            return Some(p);
        }
    }

    let parent = base_path.parent()?;
    if parent.ends_with("/home") {
        return None;
    }

    search_file_backwards(parent, rule)
}

/// search for a path, going back to each parent dir and going deep on the children
pub fn search_file_backwards_deep(base_path: &Path, rule: fn(&Path) -> bool) -> Option<PathBuf> {
    search_file_backwards_deep_in(10, &mut vec![], base_path, rule)
}

fn search_file_backwards_deep_in(
    mut safe: u32,
    visited: &mut Vec<PathBuf>,
    base_path: &Path,
    rule: fn(&Path) -> bool,
) -> Option<PathBuf> {
    safe -= 1;

    if safe == 0 {
        return None;
    }

    // println!("start {:?}", base_path);

    visited.push(base_path.into());

    let mut children = vec![];
    for f in base_path.read_dir().ok()? {
        let p = match f {
            Ok(v) => v.path(),
            _ => continue,
        };

        if p.as_path() == base_path {
            // println!("same as base");
            continue;
        }

        if visited.iter().find(|ip| ip.as_path() == &p).is_some() {
            // println!("skipping {:?}", p);
            continue;
        }

        // println!("- check {:?}", p);
        if rule(&p) {
            return Some(p);
        }

        if p.is_dir() {
            children.push(p);
        }
    }

    // search childrens
    for p in children {
        // println!("- children {:?}", base_path);
        if let Some(p) = search_file_backwards_deep_in(safe, visited, &p, rule) {
            return Some(p);
        }
    }

    // search parent
    let parent = base_path.parent()?;
    if parent.ends_with("/home") {
        return None;
    }

    search_file_backwards_deep_in(safe, visited, parent, rule)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_search_file_backwards() {
        let r = search_file_backwards(std::env::current_dir().unwrap().as_path(), |path| {
            path.ends_with(".git")
        });
        assert!(r.is_some(), "{:?}", r);
    }

    #[test]
    fn test_search_file_backwards_and_deep() {
        let r = search_file_backwards_deep(std::env::current_dir().unwrap().as_path(), |path| {
            path.ends_with(".git/index")
        });
        assert!(r.is_some(), "{:?}", r);
    }
}
