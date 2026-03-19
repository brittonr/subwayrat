#!/bin/bash
export PATH="/nix/store/yx30bd3n73w18d3r6pdxw1ira1sdfq12-rust-default-1.95.0-nightly-2026-02-06/bin:/nix/store/a245z3cvf9x9sn0xlk6k8j9xhxbhda1z-gcc-wrapper-15.2.0/bin:$PATH"
cd /home/brittonr/git/subwayrat
cargo "$@"
