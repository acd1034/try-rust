#!/bin/bash
tmp=`mktemp -d /tmp/try-rust-test-XXXXXX`
trap 'rm -rf $tmp' INT TERM HUP EXIT
echo > $tmp/empty.c

check() {
    if [ $? -eq 0 ]; then
        echo "testing $1 ... passed"
    else
        echo "testing $1 ... failed"
        exit 1
    fi
}

# -o
rm -f $tmp/out
./target/debug/try-rust -o $tmp/out $tmp/empty.c
[ -f $tmp/out ]
check -o

# --help
./target/debug/try-rust --help 2>&1 | grep -q try-rust
check --help

echo OK
