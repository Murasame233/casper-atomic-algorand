prepare:
	rustup target add wasm32-unknown-unknown

build-erc20:
	cd erc20_token && cargo build --release --target wasm32-unknown-unknown
	wasm-strip erc20_token/target/wasm32-unknown-unknown/release/erc20_token.wasm 2>/dev/null | true

build-atomic-swap:
	cd atomic_swap && cargo build --release --target wasm32-unknown-unknown
	wasm-strip atomic_swap/target/wasm32-unknown-unknown/release/atomic_swap.wasm 2>/dev/null | true

test: build-erc20 build-atomic-swap
	mkdir -p tests/wasm
	cp erc20_token/target/wasm32-unknown-unknown/release/erc20_token.wasm tests/wasm
	cp atomic_swap/target/wasm32-unknown-unknown/release/atomic_swap.wasm tests/wasm
	cd tests && cargo test

clippy:
	cd atomic_swap && cargo clippy --all-targets -- -D warnings
	cd erc20_token && cargo clippy --all-targets -- -D warnings
	cd tests && cargo clippy --all-targets -- -D warnings

check-lint: clippy
	cd atomic_swap && cargo fmt -- --check
	cd erc20_token && cargo fmt -- --check
	cd tests && cargo fmt -- --check

lint: clippy
	cd atomic_swap && cargo fmt
	cd erc20_token && cargo fmt
	cd tests && cargo fmt

clean:
	cd atomic_swap && cargo clean
	cd erc20_token && cargo clean
	cd tests && cargo clean
	rm -rf tests/wasm


prepare-test: build-erc20 build-atomic-swap
	cd algorand && bash compile.sh && mv contract ../test-net/
	mkdir -p test-net/wasm
	cp erc20_token/target/wasm32-unknown-unknown/release/erc20_token.wasm test-net/wasm
	cp atomic_swap/target/wasm32-unknown-unknown/release/atomic_swap.wasm test-net/wasm