# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.

install:
    @echo "Nothing to install yet"

build-daemon:
    cargo build --release --bin monitord --no-default-features --features=daemon

build-control:
    cargo build --release --bin monitordctl --no-default-features --features=control

test TEST:
    RUST_LOG=debug,wgpu=warn cargo test {{ TEST }} --release --no-default-features --features=daemon -- --nocapture

test-all:
    RUST_LOG=debug,wgpu=warn cargo test --release --no-default-features --features=daemon -- --show-output

run-daemon:
    RUST_LOG=debug,wgpu=warn cargo run --release --no-default-features --features=daemon --bin monitord

test-client:
    RUST_LOG=debug,wgpu=warn cargo test client --release --no-default-features --features=client -- --nocapture

clippy:
    cargo clippy --release --no-default-features --features=daemon
