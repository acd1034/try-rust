#!/bin/bash
assert() {
  expected="$1"
  input="$2"

  ./target/debug/try-rust "$input" > tmp.ll
  /opt/homebrew/opt/llvm@12/bin/clang -o tmp tmp.ll
  ./tmp
  actual="$?"

  if [ "$actual" = "$expected" ]; then
    echo "$input => $actual"
  else
    echo "$input => unexpected $actual, expecting $expected"
    exit 1
  fi
}
assert_fail() {
  input="$1"
  echo -n "$input => "
  ./target/debug/try-rust "$input" > /dev/null
  if [ $? -eq 0 ]; then
    echo "Error: unexpected success in compiling"
    exit 1
  fi
}

LLVM_SYS_120_PREFIX=/opt/homebrew/opt/llvm@12 cargo build
if [ $? -ne 0 ]; then
  exit 1
fi

# num
assert 0 '0'
assert 42 '42'
# term
assert 24 '1 * 2 * 3 * 4'
assert 4 '3 * 4 / 6 * 2'
# expr
assert 10 '1 + 2 + 3 + 4'
assert 4 '1 + 2 - 3 + 4'
assert 44 '1 * 2 + 3 * 4 + 5 * 6'
assert 20 '1 * 2 - 6 / 3 + 4 * 5'
# primary
assert 15 '5*(9-6)'
assert 4 '(3+5)/2'
assert_fail '(3+ )/2'
# unary
assert 10 '-10+20'
assert 10 '-(-10)'
assert 2 '10 + -8'
