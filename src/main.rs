//! When `cargo update` fetches a new version of a crate, that new version of the crate will be
//! re-compiled as a dependency. However the library corresponding to the previous version continues
//! to remain in the dependency folder. They are distinguished by adding a hash at the end of the
//! library name.  This makes the build cache grow in size in `Travis` etc. which is not desirable
//! as as both space and time to upload the cache are wasted. This utility allows for searching the
//! `deps` directory for duplicate libraries and prune them to contain only the latest.
//!
//! By default `./target` will be searched but via cmd line arguments one could specify a different
//! target directory. The target directory can have any complex hierarchy - they will be
//! recursively searched and pruned of duplicate library dependencies.
//!
//! Currently this only works for `.rlib` dependencies.

extern crate docopt;
extern crate rustc_serialize;
#[macro_use]
extern crate unwrap;

use docopt::Docopt;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::fs::{self, ReadDir};
use std::path::PathBuf;

static USAGE: &'static str = "
Usage:
  cargo prune [options]

Options:
  --target <path>  Custom target directory to search for dependencies.
  -v, --version    Display the version and exit.
  -h, --help       Display this help message and exit.
";

const DEFAULT_TARGET: &'static str = "./target";
/// Allowed duration from the latest for duplicates to exist. Make it 0 for there to be always 0
/// duplicates.
const ALLOWED_DURATION_SEC: u64 = 2 * 3600; // 2hrs

#[derive(Debug, RustcDecodable)]
struct Args {
    flag_target: Option<String>,
    flag_version: bool,
    flag_help: bool,
}

macro_rules! dir_content_path {
    ($content_res:expr) => {{
        let entry = match $content_res {
            Ok(entry) => entry,
            Err(e) => {
                println!("WARN: Could not evaluate a dir content: {:?}", e);
                continue;
            }
        };

        entry.path()
    }};
}

fn main() {
    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.decode())
        .unwrap_or_else(|e| e.exit());

    if args.flag_version {
        return println!("{}", env!("CARGO_PKG_VERSION"));
    }

    let target = match args.flag_target {
        Some(path) => PathBuf::from(path),
        None => PathBuf::from(DEFAULT_TARGET),
    };

    search_for_deps(&target);
}

fn search_for_deps(path: &PathBuf) {
    if !path.is_dir() {
        return;
    }

    let dir = unwrap!(path.read_dir());
    if path.ends_with("deps") {
        println!("* Processing {:?}", path);
        prune(dir);
    } else {
        for content in dir {
            search_for_deps(&dir_content_path!(content));
        }
    }
}

fn prune(dir: ReadDir) {
    let mut libs = HashMap::<String, Vec<PathBuf>>::with_capacity(100);

    for content in dir {
        let path = dir_content_path!(content);

        let lib = match path.file_stem() {
            Some(stem) => match stem.to_str() {
                Some(stem) => {
                    let splits: Vec<_> = stem.rsplitn(2, '-').collect();
                    if splits.len() != 2 {
                        continue;
                    }
                    splits[1].to_string()
                }
                None => continue,
            },
            None => continue,
        };

        let lib_paths = libs.entry(lib).or_insert_with(|| Vec::with_capacity(2));
        lib_paths.push(path);
        lib_paths.sort_by(|a, b| {
            if unwrap!(unwrap!(a.metadata()).modified()) < unwrap!(unwrap!(b.metadata()).modified())
            {
                Ordering::Less
            } else {
                Ordering::Greater
            }
        });
    }

    for (lib, mut lib_paths) in libs {
        if lib_paths.len() < 2 {
            println!("    No duplicates for {:?}", lib);
            continue;
        }

        println!("    Pruning for lib {:?}", lib);
        let latest = unwrap!(unwrap!(unwrap!(lib_paths.last()).metadata()).modified());
        let rm = ALLOWED_DURATION_SEC == 0
            || lib_paths.iter().any(|lib_path| {
                let earlier = unwrap!(unwrap!(lib_path.metadata()).modified());
                match latest.duration_since(earlier) {
                    Ok(dur) => dur.as_secs() > ALLOWED_DURATION_SEC,
                    Err(_) => false, // means the delta was negative. Ignore this as it's
                                     // practically only possible for files created/touched almost
                                     // simultaneously.
                }
            });
        if rm {
            if ALLOWED_DURATION_SEC == 0 {
                let _ = lib_paths.pop();
            }
            lib_paths.iter().for_each(|lib_path| {
                print!("      Deleting {:?} ... ", lib_path);
                if let Err(e) = fs::remove_file(lib_path) {
                    println!("ERROR: {:?}", e);
                } else {
                    println!("ok");
                }
            });
        } else {
            print!(
                "      NOT Deleting any duplicates for {:?} as difference with duplicate is <
                {}secs... ",
                lib, ALLOWED_DURATION_SEC
            );
        }
    }
}
