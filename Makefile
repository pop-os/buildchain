prefix ?= /usr/local
exec_prefix = $(prefix)
bindir = $(exec_prefix)/bin
libdir = $(exec_prefix)/lib
includedir = $(prefix)/include
datadir = $(prefix)/share

.PHONY: all clean distclean install uninstall update

BIN = buildchain
SRC = Cargo.toml Cargo.lock Makefile $(shell find src -type f -wholename '*src/*.rs')

ARGS = --release
VENDOR ?= 0
ifeq ($(VENDOR),1)
	ARGS += --frozen
endif

all: target/release/$(BIN)

clean:
	cargo clean

distclean: clean
	rm -rf .cargo vendor vendor.tar

install: all
	install -D -m 0755 "target/release/$(BIN)" "$(DESTDIR)$(bindir)/$(BIN)"

uninstall:
	rm -f "$(DESTDIR)$(bindir)/$(BIN)"

update:
	cargo update

.cargo/config: vendor_config
	mkdir -p .cargo
	cp $< $@

vendor: .cargo/config
	mkdir -p .cargo
	cargo vendor | head -n -1 > .cargo/config
	echo 'directory = "vendor"' >> .cargo/config
	tar cf vendor.tar vendor
	rm -rf vendor

target/release/$(BIN): $(SRC)
ifeq ($(VENDOR),1)
	tar pxf vendor.tar
endif
	cargo build $(ARGS)
