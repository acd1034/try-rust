#!/bin/bash
# usage: LLVM_SYS_120_PREFIX=/opt/homebrew/opt/llvm@12 ./test-ll.sh
cat <<EOF | $LLVM_SYS_120_PREFIX/bin/clang -xc -c -o tmp2.o -
int ret3() { return 3; }
int ret5() { return 5; }
EOF

ESC=$(printf '\033')
assert() {
  expected="$1"
  input="$2"
  echo -en "$ESC[32m$input\n$ESC[m=> "

  echo "$input" | ./target/debug/try-rust "-ll" - > tmp.ll || exit
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
  echo -en "$ESC[31m$input\n$ESC[m=> "

  echo "$input" | ./target/debug/try-rust "-ll" - > /dev/null && echo "Error: unexpected success in compiling"
}

# TODO:
# postfix bug
# assert 0 'int main() { int a[3]; a[0]=0; a[1]=1; a[2]=2; int *p=a+1; (*(p--))--; return a[1]; }'
# assert 0 'int main() { int a[3]; a[0]=0; a[1]=1; a[2]=2; int *p=a+1; (*p++)--; return a[1]; }'
# assert 2 'int main() { int a[3]; a[0]=0; a[1]=1; a[2]=2; int *p=a+1; (*p++)--; return a[2]; }'
# assert 2 'int main() { int a[3]; a[0]=0; a[1]=1; a[2]=2; int *p=a+1; (*p++)--; return *p; }'
# assert_fail 'int main() { x=3; &+x; return x; }'
# num
assert 0 'int main() { return 0; }'
assert 42 'int main() { return 42; }'
# mul
assert 24 'int main() { return 1*2*3*4; }'
assert 4 'int main() { return 3*4/6*2; }'
# add
assert 10 'int main() { return 1+2+3+4; }'
assert 4 'int main() { return 1+2-3+4; }'
assert 44 'int main() { return 1*2+3*4+5*6; }'
assert 20 'int main() { return 1*2-6/3+4*5; }'
# primary
assert 15 'int main() { return 5*(9-6); }'
assert 4 'int main() { return (3+5)/2; }'
assert_fail 'int main() { (3+ )/2; }'
# unary
assert 10 'int main() { return -10 - -20; }'
assert 10 'int main() { return -(-10); }'
assert 10 'int main() { return - -10; }'
assert 10 'int main() { return +10; }'
# relational
assert 1 'int main() { return 0<1; }'
assert 0 'int main() { return 1<1; }'
assert 0 'int main() { return 2<1; }'
assert 1 'int main() { return 0<=1; }'
assert 1 'int main() { return 1<=1; }'
assert 0 'int main() { return 2<=1; }'
assert 1 'int main() { return 1>0; }'
assert 0 'int main() { return 1>1; }'
assert 0 'int main() { return 1>2; }'
assert 1 'int main() { return 1>=0; }'
assert 1 'int main() { return 1>=1; }'
assert 0 'int main() { return 1>=2; }'
# equality
assert 0 'int main() { return 0==1; }'
assert 1 'int main() { return 42==42; }'
assert 1 'int main() { return 0!=1; }'
assert 0 'int main() { return 42!=42; }'
assert 1 'int main() { return (1==1)==(1==1); }'
assert 1 'int main() { return (1==1)==1; }'
# assign
assert 42 'int main() { int foo123; return foo123=42; }'
assert 2 'int main() { int a; return a=a=2; }'
assert 2 'int main() { int x; return (x=1)=2; }'
assert_fail 'int main() { return 1=2; }'
assert_fail 'int main() { return a; }'
assert_fail 'int main() { return a=2; }'
# variable declaration
assert 42 'int main() { int _123=42; return _123; }'
assert 8 'int main() { int a=3; int b=5; return a+b; }'
assert 6 'int main() { int a=3; int b=a; return a+b; }'
assert 6 'int main() { int b; int a=b=3; return a+b; }'
assert 2 'int main() { int x=1; x=2; return x; }'
# return
assert 1 'int main() { return 1; 2; 3; }'
assert 2 'int main() { 1; return 2; 3; }'
assert 3 'int main() { 1; 2; return 3; }'
assert 0 'int main() { return 0; return 1; }'
assert 0 'int main() { int a=0; return a; a=1; }'
assert_fail 'int main() { 1; }'
# block
assert 3 'int main() { {1; {2;} return 3;} }'
# null
assert 5 'int main() { ;;; return 5; }'
# if
assert 1 'int main() { int x=0; if (1) x=1; return x; }'
assert 1 'int main() { int x=0; if (1) x=1; else x=2; return x; }'
assert 3 'int main() { if (0) return 2; return 3; }'
assert 3 'int main() { if (1-1) return 2; return 3; }'
assert 2 'int main() { if (1) return 2; return 3; }'
assert 2 'int main() { if (2-1) return 2; return 3; }'
assert 4 'int main() { if (0) { 1; 2; return 3; } return 4; }'
assert 4 'int main() { if (0) { 1; 2; return 3; } else { return 4; } }'
assert 1 'int main() { if (1) if (1) return 1; else return 2; return 3; }'
assert 2 'int main() { if (1) if (0) return 1; else return 2; return 3; }'
assert 3 'int main() { if (0) if (1) return 1; else return 2; return 3; }'
# no else follows first if
assert 0 'int main() { int x=0; if (x) { x=1; } return x; }'
assert 0 'int main() { int x=0; if (x) { if (x) x=1; } return x; }'
assert 0 'int main() { int x=0; if (x) { if (x) x=1; else x=2; } return x; }'
# else follows first if, nested if in first if
assert 2 'int main() { int x=0; if (x) { x=1; } else x=2; return x; }'
assert 2 'int main() { int x=0; if (x) { if (x) x=1; } else x=2; return x; }'
assert 3 'int main() { int x=0; if (x) { if (x) x=1; else x=2; } else x=3; return x; }'
# else follows first if, nested if in first else
assert 2 'int main() { int x=0; if (x) x=1; else { x=2; } return x; }'
assert 0 'int main() { int x=0; if (x) x=1; else { if (x) x=2; } return x; }'
assert 3 'int main() { int x=0; if (x) x=1; else { if (x) x=2; else x=3; } return x; }'
# for
assert 3 'int main() { for (;;) return 3; return 5; }'
assert 55 'int main() { int i; int j=0; for (i=0; i<=10; i=i+1) j=i+j; return j; }'
assert 2 'int main() { int x=0; for (;;) if (x) return 1; else return 2; }'
assert 5 'int main() { int x=0; for (;;) { if (x==5) return x; x=x+1; } }'
# while
assert 10 'int main() { int i=0; while(i<10) { i=i+1; } return i; }'
# break
assert 3 'int main() { int i=0; for(;i<10;i++) { if (i == 3) break; } return i; }'
assert 4 'int main() { int i=0; while (1) { if (i++ == 3) break; } return i; }'
assert 3 'int main() { int i=0; for(;i<10;i++) { for (;;) break; if (i == 3) break; } return i; }'
assert 4 'int main() { int i=0; while (1) { while(1) break; if (i++ == 3) break; } return i; }'
# continue
assert 10 'int main() { int i=0; int j=0; for (;i<10;i++) { if (i>5) continue; j++; } return i; }'
assert 6 'int main() { int i=0; int j=0; for (;i<10;i++) { if (i>5) continue; j++; } return j; }'
assert 11 'int main() { int i=0; int j=0; while (i++<10) { if (i>5) continue; j++; } return i; }'
assert 5 'int main() { int i=0; int j=0; while (i++<10) { if (i>5) continue; j++; } return j; }'
assert 10 'int main() { int i=0; int j=0; for(;i==0;) { for (;j!=10;j++) continue; break; } return j; }'
assert 11 'int main() { int i=0; int j=0; while(i==0) { while (j++!=10) continue; break; } return j; }'
# defunc
assert 6 'int sub() { return 4; } int main() { int b=3; int a=b; return a+b; }'
assert_fail 'int main() { return 4; } int main() { int b=3; int a=b; return a+b; }'
assert 0 'int sub(int a, int b, int c, int d, int e, int f) { return a+b+c+d+e+f; } int main() { return 0; }'
assert_fail 'int sub(int a, int a) { return a; } int main() { return 0; }'
# funcall
assert 8 'int sub() { return 4; } int main() { int b=sub(); int a=b; return a+b; }'
assert_fail 'int main() { int b=sub(); int a=b; return a+b; }'
assert 21 'int sub(int a, int b, int c, int d, int e, int f) { return a+b+c+d+e+f; } int main() { return sub(1,2,3,4,5,6); }'
# prototype
assert 3 'int ret3(); int main() { return ret3(); }'
assert 5 'int ret5(); int main() { return ret5(); }'
assert 4 'int sub(); int sub(); int sub() { return 4; } int main() { return sub(); }'
assert 0 'int sub(int a); int sub(int b) { return b; } int main() { return sub(0); }'
assert_fail 'int sub(); int sub(int a) { return a; } int main() { return 0; }'
assert 21 'int sub(int a, int b, int c, int d, int e, int f); int sub(int g, int h, int i, int j, int k, int l) { return g+h+i+j+k+l; } int main() { return sub(1,2,3,4,5,6); }'
# addr & deref
assert 3 'int main() { int x=3; return *&x; }'
assert 3 'int main() { int x=3; int* y=&x; int** z=&y; return **z; }'
assert 5 'int main() { int x=3; int* y=&x; *y=5; return x; }'
assert_fail 'int main() { int x=3; &-x; return x; }'
assert_fail 'int main() { int x=3; *x; return x; }'
assert_fail 'int main() { int x=3; int* y=&x; int** z=&y; z=&x; return *z; }'
assert_fail 'int main() { int x=3; int* y=&x; int** z=&y; z=&x; return **z; }'
assert 3 'int sub(int* a) { *a = 3; return 4; } int main() { int x=0; sub(&x); return x; }'
assert 3 'int* sub(int* a) { return a; } int main() { int x=0; *sub(&x) = 3; return x; }'
assert 3 'int** sub(int** a) { return a; } int main() { int x=3; int* y; *sub(&y) = &x; return *y; }'
assert_fail 'int sub(int a); int sub(int* b) { return *b; } int main() { return sub(3); }'
assert_fail 'int sub(int a) { return a; } int main() { int x=0; sub(&x); return x; }'
assert_fail 'int* sub(int* a) { return a; } int main() { int x=3; int* y; *sub(&y) = &x; return *y; }'
# pointer arithmetic
assert 5 'int main() { int x=3; int y=5; return *(&x+1); }'
assert 7 'int main() { int x=3; int y=5; *(&x+1)=7; return y; }'
assert 3 'int main() { int x=3; int y=5; return *(&y-1); }'
assert 7 'int main() { int x=3; int y=5; *(&y-1)=7; return x; }'
assert 5 'int main() { int x=3; int y=5; return *(&x+1); }'
assert 7 'int main() { int x=3; int y=5; *(&x+1)=7; return y; }'
assert 5 'int main() { int x=3; int y=5; return *(&x-(-1)); }'
assert 7 'int main() { int x=3; int y=5; *(&y-2+1)=7; return x; }'
assert 1 'int main() { int x=3; int y=5; return &y-&x; }'
assert 1 'int main() { int x=3; int y=5; return -(&x-&y); }'
assert 5 'int main() { int x=3; return (&x+2)-&x+3; }'
assert_fail 'int main() { int x=3; int y=5; return &x+&y; }'
assert_fail 'int main() { int x=3; int* y=&x; return x-y; }'
assert_fail 'int main() { int x=3; int* y=&x; return &x-&y; }'
# array
assert 2 'int main() { int a[3]; *(a+1)=2; return *(a+1); }'
assert 3 'int main() { int x[2]; int *y=&x; *y=3; return *x; }'
assert 3 'int main() { int x[3]; *x=3; *(x+1)=4; *(x+2)=5; return *x; }'
assert 4 'int main() { int x[3]; *x=3; *(x+1)=4; *(x+2)=5; return *(x+1); }'
assert 5 'int main() { int x[3]; *x=3; *(x+1)=4; *(x+2)=5; return *(x+2); }'
assert 3 'int main() { int x[3]; *x=3; x[1]=4; x[2]=5; return *x; }'
assert 4 'int main() { int x[3]; *x=3; x[1]=4; x[2]=5; return *(x+1); }'
assert 5 'int main() { int x[3]; *x=3; x[1]=4; x[2]=5; return *(x+2); }'
assert 5 'int main() { int x[3]; *x=3; x[1]=4; x[2]=5; return *(x+2); }'
assert 5 'int main() { int x[3]; *x=3; x[1]=4; 2[x]=5; return *(x+2); }'
assert 4 'int main() { int x[5]; int i=0; for(;i<5;i=i+1) x[i]=i; return x[i-1]; }'
# scope
assert 2 'int main() { int x=2; { int x=3; } return x; }'
assert 2 'int main() { int x=2; { int x=3; } { int y=4; return x; }}'
assert 3 'int main() { int x=2; { x=3; } return x; }'
# compound assignment
assert 7 'int main() { int i=2; i+=5; return i; }'
assert 7 'int main() { int i=2; return i+=5; }'
assert 3 'int main() { int i=5; i-=2; return i; }'
assert 3 'int main() { int i=5; return i-=2; }'
assert 6 'int main() { int i=3; i*=2; return i; }'
assert 6 'int main() { int i=3; return i*=2; }'
assert 3 'int main() { int i=6; i/=2; return i; }'
assert 3 'int main() { int i=6; return i/=2; }'
# prefix increment & decrement
assert 3 'int main() { int i=2; return ++i; }'
assert 5 'int main() { int i=0; return ++i=5; }'
assert 2 'int main() { int a[3]; a[0]=0; a[1]=1; a[2]=2; int *p=a+1; return ++*p; }'
assert 0 'int main() { int a[3]; a[0]=0; a[1]=1; a[2]=2; int *p=a+1; return --*p; }'
# postfix increment & decrement
assert 2 'int main() { int i=2; return i++; }'
assert 2 'int main() { int i=2; return i--; }'
assert 3 'int main() { int i=2; i++; return i; }'
assert 1 'int main() { int i=2; i--; return i; }'
assert 1 'int main() { int a[3]; a[0]=0; a[1]=1; a[2]=2; int *p=a+1; return *p++; }'
assert 1 'int main() { int a[3]; a[0]=0; a[1]=1; a[2]=2; int *p=a+1; return *p--; }'
assert 0 'int main() { int a[3]; a[0]=0; a[1]=1; a[2]=2; int *p=a+1; (*p++)--; return a[0]; }'
# assert 0 'int main() { int a[3]; a[0]=0; a[1]=1; a[2]=2; int *p=a+1; (*(p--))--; return a[1]; }'
assert 2 'int main() { int a[3]; a[0]=0; a[1]=1; a[2]=2; int *p=a+1; (*p)--; return a[2]; }'
assert 2 'int main() { int a[3]; a[0]=0; a[1]=1; a[2]=2; int *p=a+1; (*p)--; p++; return *p; }'
assert 0 'int main() { int a[3]; a[0]=0; a[1]=1; a[2]=2; int *p=a+1; (*p++)--; return a[0]; }'
# assert 0 'int main() { int a[3]; a[0]=0; a[1]=1; a[2]=2; int *p=a+1; (*p++)--; return a[1]; }'
# assert 2 'int main() { int a[3]; a[0]=0; a[1]=1; a[2]=2; int *p=a+1; (*p++)--; return a[2]; }'
# assert 2 'int main() { int a[3]; a[0]=0; a[1]=1; a[2]=2; int *p=a+1; (*p++)--; return *p; }'
# ternary
assert 2 'int main() { return 0?1:2; }'
assert 1 'int main() { return 1?1:2; }'
assert 1 'int main() { return 1 ? 1==1 : 0; }'
# practical
assert 55 'int fib(int x) { return x<=1 ? 1 : fib(x-1) + fib(x-2); } int main() { return fib(9); }'
assert 1 'int partition(int* a, int p, int r) { int piv = a[r]; int i = p - 1; int j; for (j = p; j < r; ++j) if (a[j] <= piv) { ++i; int tmp = a[i]; a[i] = a[j]; a[j] = tmp; } ++i; int tmp = a[i]; a[i] = a[j]; a[j] = tmp; return i; } int quicksort(int* a, int p, int r) { if (p < r) { int q = partition(a, p, r); quicksort(a, p, q - 1); quicksort(a, q + 1, r); } return 0; } int sorted(int* a, int n) { int i; for (i = 1; i < n; ++i) if (a[i - 1] > a[i]) return 0; return 1; } int main() { int a[9]; a[0] = 8; a[1] = 4; a[2] = 3; a[3] = 0; a[4] = 7; a[5] = 6; a[6] = 5; a[7] = 2; a[8] = 1; quicksort(a, 0, 9); return sorted(a, 9); }'
