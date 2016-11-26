# cargo-prune
Prune crate dependencies in "target" folder

When `cargo update` fetches a new version of a crate, that new version of the crate will be
re-compiled as a dependency. However the library corresponding to the previous version continues
to remain in the dependency folder. They are distinguished by adding a hash at the end of the
library name.  This makes the build cache grow in size in `Travis` etc. which is not desirable
as as both space and time to upload the cache are wasted. This utility allows for searching the
`deps` directory for duplicate libraries and prune them to contain only the latest.

By default `./target` will be searched but via cmd line arguments one could specify a different
target directory. The target directory can have any complex hierarchy - they will be
recursively searched and pruned of duplicate library dependencies.

Currently this only works for `.rlib` dependencies.

E.g.:
- `./cargo-prune`
- `./cargo-prune --target=some/path`
- `cargo prune` (if installed to cargo bin directory)
