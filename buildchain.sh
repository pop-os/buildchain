#!/usr/bin/env bash
set -e

export LC=C.UTF-8
export SOURCE_DATE_EPOCH="$(git log -1 --format='%ct')"

rm -rf target/sysroot target/sysroot.tar.xz
mkdir -p target/sysroot
cargo install --root target/sysroot
tar --create --verbose --xz \
    --mtime="@${SOURCE_DATE_EPOCH}" --sort=name \
    --owner=0 --group=0 --numeric-owner \
    --file target/sysroot.tar.xz --directory target/sysroot .
