try-rust:
	cargo build

test-ll: try-rust
	test/test-ll.sh

test-c: try-rust
	test/test-c.sh

TEST_SRCS=$(wildcard test/*.c)
TESTS=$(TEST_SRCS:.c=.out)

test/%.out: try-rust test/%.c
	$(CC) -o- -E -P -C test/$*.c | ./target/debug/try-rust -ll -otest/$*.ll -
	$(CC) -otest/$*.out test/$*.ll -xc test/common -Wno-override-module

test: $(TESTS)
	for i in $^; do echo $$i; ./$$i || exit 1; echo "  ... passed"; done
	test/driver.sh

.PHONY: test test-ll test-c
