# reference: https://github.com/dalance/procs/blob/master/Makefile

VERSION=$(patsubst "%",%, $(word 3, $(shell grep version Cargo.toml)))
BUILD_TIME=$(shell date +"%Y/%m/%d %H:%M:%S")
GIT_REVISION=$(shell git log -1 --format="%h")
RUST_VERSION=$(word 2, $(shell rustc -V))
LONG_VERSION="$(VERSION) ( rev: $(GIT_REVISION), rustc: $(RUST_VERSION), build at: $(BUILD_TIME) )"
BIN_NAME=ultraman
BASE_RELEASE_FILES := ./tmp/ultraman.1 README.md LICENSE

export LONG_VERSION

.PHONY: create_man man install_man test release_linux release_win release_mac

create_man:
	cargo run --bin man --features man > ./tmp/ultraman.1;

man: create_man
	man ./tmp/ultraman.1;

install_man: create_man
	install -Dm644 ./tmp/ultraman.1 /usr/local/share/man/man1/ultraman.1;

test:
	cargo test --locked
	cargo test --locked -- --ignored

test-no-default-features:
	cargo test --locked --no-default-features
	cargo test --locked --no-default-features -- --ignored

release_linux: create_man
	cargo build --locked --release --target=x86_64-unknown-linux-musl
	zip -j ${BIN_NAME}-v${VERSION}-x86_64-linux.zip target/x86_64-unknown-linux-musl/release/${BIN_NAME} $(strip $(BASE_RELEASE_FILES))

release_win: create_man
	cargo build --locked --release --target=x86_64-pc-windows-msvc
	7z a ${BIN_NAME}-v${VERSION}-x86_64-win.zip target/x86_64-pc-windows-msvc/release/${BIN_NAME}.exe $(strip $(BASE_RELEASE_FILES))

release_mac: create_man
	cargo build --locked --release --target=x86_64-apple-darwin
	zip -j ${BIN_NAME}-v${VERSION}-x86_64-mac.zip target/x86_64-apple-darwin/release/${BIN_NAME} $(strip $(BASE_RELEASE_FILES))

