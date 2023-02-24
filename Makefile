try-rust:
	cargo build

test: try-rust
	./test.sh

test2: try-rust
	./test2.sh

.PHONY: test
