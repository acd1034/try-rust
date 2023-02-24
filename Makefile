try-rust:
	cargo build

test-ll: try-rust
	./test-ll.sh

test-c: try-rust
	./test-c.sh

.PHONY: test
