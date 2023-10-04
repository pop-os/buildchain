#!/bin/sh

sudo apt -q update
sudo apt -q install --no-install-recommends --yes \
    build-essential \
    libssl-dev \
    pkgconf
