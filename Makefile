.PHONY: all build test check clean release install sync sign lint

NAME = agent-eyes

all: check build

build:
	cargo build --release -p $(NAME)

test:
	cargo test --release -p $(NAME)

check:
	cargo check -p $(NAME)

lint:
	cargo clippy --workspace -- -D warnings

clean:
	cargo clean

release: build
	./scripts/build-release-macos.sh

install: build
	./scripts/sync-local-release.sh

sync: install

sign: build
	./scripts/sign-macos.sh
