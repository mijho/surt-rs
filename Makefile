.PHONY: run dev test coverage fmt lint

run:
	@cargo run --release

dev:
	@cargo run

test:
	@cargo test

coverage:
	@cargo tarpaulin --lib

fmt:
	@cargo fmt

fmt-check:
	@cargo fmt -- --check

lint:
	@cargo clippy

lint-check:
	@cargo clippy -- -D warnings