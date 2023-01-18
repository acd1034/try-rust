#!/bin/bash
assert() {
  expected="$1"
  input="$2"
  echo -n "$input => "

  ./target/debug/try-rust "$input" > tmp.ll
  $LLVM_SYS_120_PREFIX/bin/clang -o tmp tmp.ll
  ./tmp
  actual="$?"

  if [ "$actual" = "$expected" ]; then
    echo "$actual"
  else
    echo "unexpected $actual, expecting $expected"
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

cargo build
if [ $? -ne 0 ]; then
  exit 1
fi

# num
assert 0 'main() { return 0; }'
assert 42 'main() { return 42; }'
# term
assert 24 'main() { return 1 * 2 * 3 * 4; }'
assert 4 'main() { return 3 * 4 / 6 * 2; }'
# expr
assert 10 'main() { return 1 + 2 + 3 + 4; }'
assert 4 'main() { return 1 + 2 - 3 + 4; }'
assert 44 'main() { return 1 * 2 + 3 * 4 + 5 * 6; }'
assert 20 'main() { return 1 * 2 - 6 / 3 + 4 * 5; }'
# primary
assert 15 'main() { return 5*(9-6); }'
assert 4 'main() { return (3+5)/2; }'
assert_fail 'main() { (3+ )/2; }'
# unary
assert 10 'main() { return -10 - -20; }'
assert 10 'main() { return -(-10); }'
assert 10 'main() { return - -10; }'
assert 10 'main() { return +10; }'
# relational
assert 1 'main() { return 0<1; }'
assert 0 'main() { return 1<1; }'
assert 0 'main() { return 2<1; }'
assert 1 'main() { return 0<=1; }'
assert 1 'main() { return 1<=1; }'
assert 0 'main() { return 2<=1; }'
assert 1 'main() { return 1>0; }'
assert 0 'main() { return 1>1; }'
assert 0 'main() { return 1>2; }'
assert 1 'main() { return 1>=0; }'
assert 1 'main() { return 1>=1; }'
assert 0 'main() { return 1>=2; }'
# equality
assert 0 'main() { return 0==1; }'
assert 1 'main() { return 42==42; }'
assert 1 'main() { return 0!=1; }'
assert 0 'main() { return 42!=42; }'
# assign
assert 42 'main() { return foo123=42; }'
assert 42 'main() { return _123=42; }'
assert 2 'main() { return a=a=2; }'
assert_fail 'main() { return 1=2; }'
# statements
assert 3 'main() { 1; 2; return 3; }'
assert 8 'main() { a=3; b=5; return a+b; }'
assert 6 'main() { a=3; b=a; return a+b; }'
assert 6 'main() { a=b=3; return a+b; }'
assert 2 'main() { (x=1)=2; return x; }'
assert 2 'main() { x=1; x=2; return x; }'
assert 3 'main() { foo=3; return foo; }'
assert 8 'main() { foo123=3; bar=5; return foo123+bar; }'
# return
assert 1 'main() { return 1; 2; 3; }'
assert 2 'main() { 1; return 2; 3; }'
assert 3 'main() { 1; 2; return 3; }'
assert 6 'sub() { return 4; } main() { a=b=3; return a+b; }'
# block
assert 3 'main() { {1; {2;} return 3;} }'
# null
assert 5 'main() { ;;; return 5; }'
# if
assert 1 'main() { x=0; if (1) x=1; return x; }'
assert 1 'main() { x=0; if (1) x=1; else x=2; return x; }'
assert 3 'main() { if (0) return 2; return 3; }'
assert 3 'main() { if (1-1) return 2; return 3; }'
assert 2 'main() { if (1) return 2; return 3; }'
assert 2 'main() { if (2-1) return 2; return 3; }'
assert 4 'main() { if (0) { 1; 2; return 3; } return 4; }'
assert 4 'main() { if (0) { 1; 2; return 3; } else { return 4; } }'
assert 3 'main() { if (1) { 1; 2; return 3; } else { return 4; } }'
assert 4 'main() { if (0) { 1; 2; return 3; } return 4; }'
assert 4 'main() { if (0) { 1; 2; return 3; } else { return 4; } }'
assert 3 'main() { if (1) { 1; 2; return 3; } else { return 4; } }'
assert 1 'main() { if (1) if (1) return 1; else return 2; return 3; }'
assert 2 'main() { if (1) if (0) return 1; else return 2; return 3; }'
assert 3 'main() { if (0) if (1) return 1; else return 2; return 3; }'
