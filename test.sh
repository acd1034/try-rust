#!/bin/bash
LLVM_SYS_120_PREFIX=/opt/homebrew/opt/llvm@12 cargo build

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

assert 0 '0'
assert 42 '42'
assert 21 '5+20-4'
assert 41 '12 + 34 - 5'
assert 47 '5+6*7'
assert 15 '5*(9-6)'
assert 4 '(3+5)/2'
