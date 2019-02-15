# cargo-prune
Prune crate dependencies in "target" folder

When `cargo update` fetches a new version of a crate, that new version of the crate will be
re-compiled as a dependency. However the library corresponding to the previous version continues
to remain in the dependency folder. They are distinguished by adding a hash at the end of the
library name.  This makes the build cache grow in size in `Travis` etc. which is not desirable
as as both space and time to upload the cache are wasted. This utility allows for searching the
`deps` directory for duplicate libraries and prune them to contain only a required few.

At this point, the question is why "a few" instead of exactly one. This is because consider the
case where we have deps `A, B, C and D` and each of them internally depend on `Z` as one of the
deps. The situation is made tricky by the fact that each of them depend on a different version
of `Z`. `A` on `Z-ver-x0`, `B` on `Z-ver-x1`, `C` on `Z-ver-x2` and `D` on `Z-ver-x3`. Now all
of these are needed. If we tried to keep only one `Z` lying around (probably the latest) then
the next build would need re-compilation of the other 3 immediately. This is not what we want.
This is further complicated by the fact that merely reading `Cargo.lock` file and "guessing"
how many to leave around can sometimes not work as some CIs run `cargo clippy` first and that
seems to create a few other rlibs for `Z`. This only happens for some libs and not every lib.
So instead of working out what the pattern here is which can change in future, what is done is
the prune preserves all `Zs` if they don't differ in timestamp by more than 2 hrs. This is a
constant and can be changed and recompiled. This is assuming that the run between `cargo clippy`
and `cargo test` etc., don't differ by more than 2 hrs. If a library is huge and it differs by
more than this amount of time then the constant should be increased appropriately and the crate
recompiled. If `cargo prune` gets enough popularity and this feature wanted where we can specify
the value at run time via a flag or config, then code will be changed upon request and version
published. However, until then, just keeping this simple with a constant defined in-code.

By default `./target` will be searched but via cmd line arguments one could specify a different
target directory. The target directory can have any complex hierarchy - they will be
recursively searched and pruned of duplicate library dependencies.

Currently this only works for `.rlib` dependencies.

You will need to cargo install it (i.e. should be in `~/.cargo/bin/` in linux etc.) for it to work.

E.g.:

- `cargo prune` (if installed to cargo bin directory)
- `cargo prune --target some/path` (if installed to cargo bin directory)
