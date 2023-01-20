#!/bin/bash
# usage: LLVM_SYS_120_PREFIX=/opt/homebrew/opt/llvm@12 ./test.sh
cat <<EOF | $LLVM_SYS_120_PREFIX/bin/clang -xc -c -o tmp2.o -
int ret3() { return 3; }
int ret5() { return 5; }
EOF

assert() {
  expected="$1"
  input="$2"
  echo -n "$input => "

  ./target/debug/try-rust "$input" > tmp.ll
  $LLVM_SYS_120_PREFIX/bin/clang -o tmp tmp.ll tmp2.o -Wno-override-module
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

# TODO:
# assert 0 'main() { return 0; return 1; }'
# assert 0 'main() { a = 0; return a; a = 1; }'
# assert_fail 'main() { x=3; &+x; return x; }'
# assert_fail 'main() { a=1; b=2; a=&b; return a; }'
# assert 5 'main() { x=3; y=5; return *(&x+8); }'
# assert 3 'main() { x=3; y=5; return *(&y-8); }'
# assert 7 'main() { x=3; y=5; *(&x+8)=7; return y; }'
# assert 7 'main() { x=3; y=5; *(&y-8)=7; return x; }'
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
assert 2 'main() { return (x=1)=2; }'
assert_fail 'main() { return 1=2; }'
assert_fail 'main() { return a; }'
# statements
assert 3 'main() { 1; 2; return 3; }'
assert 8 'main() { a=3; b=5; return a+b; }'
assert 6 'main() { a=3; b=a; return a+b; }'
assert 6 'main() { a=b=3; return a+b; }'
assert 2 'main() { x=1; x=2; return x; }'
assert 3 'main() { foo=3; return foo; }'
assert 8 'main() { foo123=3; bar=5; return foo123+bar; }'
# return
assert 1 'main() { return 1; 2; 3; }'
assert 2 'main() { 1; return 2; 3; }'
assert 3 'main() { 1; 2; return 3; }'
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
# for
assert 3 'main() { for (;;) {return 3;} return 5; }'
assert 55 'main() { i=0; j=0; for (i=0; i<=10; i=i+1) j=i+j; return j; }'
# defunc
assert 6 'sub() { return 4; } main() { a=b=3; return a+b; }'
assert_fail 'main() { return 4; } main() { a=b=3; return a+b; }'
assert 0 'sub(a,b,c,d,e,f) { return a+b+c+d+e+f; } main() { return 0; }'
# funcall
assert 8 'sub() { return 4; } main() { a=b=sub(); return a+b; }'
assert_fail 'main() { a=b=sub(); return a+b; }'
assert 21 'sub(a,b,c,d,e,f) { return a+b+c+d+e+f; } main() { return sub(1,2,3,4,5,6); }'
# prototype
assert 3 'ret3(); main() { return ret3(); }'
assert 5 'ret5(); main() { return ret5(); }'
assert 4 'sub(); sub(); sub() { return 4; } main() { return sub(); }'
assert 0 'sub(a); sub(b) { return b; } main() { return 0; }'
assert_fail 'sub(); sub(a) { return a; } main() { return 0; }'
assert 21 'sub(a,b,c,d,e,f); sub(g,h,i,j,k,l) { return g+h+i+j+k+l; } main() { return sub(1,2,3,4,5,6); }'
# addr & deref
assert 3 'main() { x=3; return *&x; }'
assert 3 'main() { x=3; y=&x; z=&y; return **z; }'
assert 5 'main() { x=3; y=&x; *y=5; return x; }'
assert_fail 'main() { x=3; &-x; return x; }'
assert_fail 'main() { x=3; *x; return x; }'
