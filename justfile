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
    RUST_LOG=debug,wgpu=warn cargo test {{ TEST }} --release --features=collector -- --show-output

test-all:
    RUST_LOG=debug,wgpu=warn cargo test --release --features=collector -- --show-output

clippy:
    cargo clippy --release --features=daemon
