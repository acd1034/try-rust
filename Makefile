# ----- test-ll-old -----

test-ll-old:
	cargo build
	test/test-ll.sh

# ----- test-ll -----

TEST_SRCS=$(wildcard test/*.c)
TESTS=$(TEST_SRCS:.c=.out)

test/%.out: test/%.c
	$(CC) -o- -E -P -C test/$*.c | ./target/debug/try-rust -ll -otest/$*.ll -
	$(CC) -otest/$*.out test/$*.ll -xc test/common -Wno-override-module

test-ll: $(TESTS)
	cargo build
	for i in $^; do echo $$i; ./$$i || exit 1; echo "  ... passed"; done
	test/driver.sh

# ----- test-ir1 -----

test-ir1:
	cargo build
	test/test-ir1.sh

.PHONY: test-ll-old test-ll test-ir1
