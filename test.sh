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
# LLVM_SYS_120_PREFIX=/opt/homebrew/opt/llvm@12 cargo --explain E0499 E0502
if [ $? -ne 0 ]; then
  exit 1
fi

# num
assert 0 'return 0;'
assert 42 'return 42;'
# term
assert 24 'return 1 * 2 * 3 * 4;'
assert 4 'return 3 * 4 / 6 * 2;'
# expr
assert 10 'return 1 + 2 + 3 + 4;'
assert 4 'return 1 + 2 - 3 + 4;'
assert 44 'return 1 * 2 + 3 * 4 + 5 * 6;'
assert 20 'return 1 * 2 - 6 / 3 + 4 * 5;'
# primary
assert 15 'return 5*(9-6);'
assert 4 'return (3+5)/2;'
assert_fail '(3+ )/2;'
# unary
assert 10 'return -10 - -20;'
assert 10 'return -(-10);'
assert 10 'return - -10;'
assert 10 'return +10;'
# relational
assert 1 'return 0<1;'
assert 0 'return 1<1;'
assert 0 'return 2<1;'
assert 1 'return 0<=1;'
assert 1 'return 1<=1;'
assert 0 'return 2<=1;'
assert 1 'return 1>0;'
assert 0 'return 1>1;'
assert 0 'return 1>2;'
assert 1 'return 1>=0;'
assert 1 'return 1>=1;'
assert 0 'return 1>=2;'
# equality
assert 0 'return 0==1;'
assert 1 'return 42==42;'
assert 1 'return 0!=1;'
assert 0 'return 42!=42;'
# assign
assert 42 'return foo123=42;'
assert 42 'return _123=42;'
# statements
assert 3 '1; 2; return 3;'
assert 8 'a=3; z=5; return a+z;'
assert 6 'a=b=3; return a+b;'
# assert 2 'return a=a=2;'
# assert 2 '(x=1)=2; return x;'
