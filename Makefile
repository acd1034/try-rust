try-rust:
	cargo build

test: try-rust
	./test.sh

.PHONY: test
