prefix ?= /usr/local
exec_prefix = $(prefix)
bindir = $(exec_prefix)/bin
libdir = $(exec_prefix)/lib
includedir = $(prefix)/include
datarootdir = $(prefix)/share
datadir = $(datarootdir)

.PHONY: all clean distclean install uninstall update

all: target/release/buildchain

clean:
	cargo clean

distclean: clean

install: all
	install -D -m 0755 "target/release/buildchain" "$(DESTDIR)$(bindir)/buildchain"

uninstall:
	rm -f "$(DESTDIR)$(bindir)/buildchain"

update:
	cargo update

vendor:
	cargo vendor
	touch vendor

target/release/buildchain: vendor
	cargo build --frozen --release
