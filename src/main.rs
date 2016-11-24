#[macro_use]
extern crate unwrap;

use std::cmp::Ordering;
use std::collections::HashMap;
use std::fs::{self, ReadDir};
use std::path::PathBuf;

fn main() {
    let target = unwrap!(fs::read_dir("./target"));

    for content in target {
        let entry = match content {
            Ok(entry) => entry,
            Err(e) => {
                println!("Could not evaluate dir: {:?}", e);
                continue;
            }
        };

        search_for_deps(entry.path());
    }
}

fn search_for_deps(path: PathBuf) {
    if !path.is_dir() {
        return;
    }

    if path.ends_with("deps") {
        let deps = unwrap!(path.read_dir());
        prune(deps);
    } else {
        search_for_deps(path);
    }
}

fn prune(dir: ReadDir) {
    let mut libs = HashMap::<String, Vec<PathBuf>>::with_capacity(100);

    for content in dir {
        let entry = match content {
            Ok(entry) => entry,
            Err(e) => {
                println!("Could not evaluate dir: {:?}", e);
                continue;
            }
        };

        let path = entry.path();

        match path.extension() {
            Some(ext) => {
                if ext != "rlib" {
                    continue;
                }
            }
            None => continue,
        }

        let lib = match path.file_stem() {
            Some(stem) => {
                match stem.to_str() {
                    Some(stem) => {
                        let splits: Vec<_> = stem.rsplitn(2, '-').collect();
                        if splits.len() != 2 {
                            continue;
                        }
                        splits[1].to_string()
                    }
                    None => continue,
                }
            }
            None => continue,
        };

        let lib_paths = libs.entry(lib).or_insert_with(|| Vec::with_capacity(2));
        lib_paths.push(path);
        lib_paths.sort_by(|a, b| {
            if unwrap!(unwrap!(a.metadata()).modified()) <
               unwrap!(unwrap!(b.metadata()).modified()) {
                Ordering::Less
            } else {
                Ordering::Greater
            }
        });
    }

    for (lib, mut lib_paths) in libs.into_iter() {
        if lib_paths.len() < 2 {
            continue;
        }

        println!("Pruning for lib {:?} ...", lib);
        let _ = lib_paths.pop();
        for lib_path in lib_paths {
            if let Err(e) = fs::remove_file(lib_path.clone()) {
                println!("Unable to delete {:?}: {:?}", lib_path, e);
            }
        }
        println!("Pruned for lib {:?}.", lib);
    }
}
