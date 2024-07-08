PINNED_TOOLCHAIN := $(shell cat rust-toolchain)

prepare:
	rustup target add wasm32-unknown-unknown
	rustup component add clippy --toolchain ${PINNED_TOOLCHAIN}
	rustup component add rustfmt --toolchain ${PINNED_TOOLCHAIN}
	rustup component add rust-src --toolchain ${PINNED_TOOLCHAIN}

.PHONY:	build-contract
build-contract:
	cd migration-test-contract && RUSTFLAGS="-C target-cpu=mvp" cargo build --release --target wasm32-unknown-unknown -Z build-std=std,panic_abort
	cd owner-contract && RUSTFLAGS="-C target-cpu=mvp" cargo build --release --target wasm32-unknown-unknown -Z build-std=std,panic_abort
	wasm-strip target/wasm32-unknown-unknown/release/migration_test_contract_pre.wasm
	wasm-strip target/wasm32-unknown-unknown/release/owner_contract.wasm

create-fixture:
	cd tests && cargo run --release

cargo-update:
	cd migration-test-contract && cargo update
	cd owner-contract && cargo update

setup-test: build-contract
	mkdir -p tests/wasm
	cp ./target/wasm32-unknown-unknown/release/migration_test_contract_pre.wasm tests/wasm
	cp ./target/wasm32-unknown-unknown/release/owner_contract.wasm tests/wasm

test: setup-test
	cd tests && cargo test

clippy:
	cd migration-test-contract && cargo clippy --all-targets -- -D warnings
	cd owner-contract && cargo clippy --all-targets -- -D warnings
	cd tests && cargo clippy --all-targets -- -D warnings

check-lint: clippy
	cd migration-test-contract && cargo fmt -- --check
	cd owner-contract && cargo fmt -- --check
	cd tests && cargo fmt -- --check

lint: clippy
	cd migration-test-contract && cargo fmt
	cd owner-contract && cargo fmt
	cd tests && cargo fmt

clean:
	cd migration-test-contract && cargo clean
	cd owner-contract && cargo clean
	cd tests && cargo clean
	rm -rf tests/wasm
	rm -rf Cargo.lock
