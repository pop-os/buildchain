#!/usr/bin/env bash
set -e

export LC=C.UTF-8
export SOURCE_DATE_EPOCH="$(git log -1 --format='%ct')"

rm -rf target/sysroot target/sysroot.tar.xz
mkdir -p target/sysroot
cargo install --root target/sysroot
tar --create \
    --mtime="@${SOURCE_DATE_EPOCH}" --owner=0 --group=0 --numeric-owner --sort=name \
    --xz --file target/sysroot.tar.xz --directory target/sysroot .
