#include "test.h"

int main()
{
  ASSERT(1, ({ struct {int a; int b;} x; x.a=1; x.b=2; x.a; }));
  ASSERT(2, ({ struct {int a; int b;} x; x.a=1; x.b=2; x.b; }));
  ASSERT(1, ({ struct {char a; int b; char c;} x; x.a=(char)1; x.b=2; x.c=(char)3; (int)x.a; }));
  ASSERT(2, ({ struct {char a; int b; char c;} x; x.b=1; x.b=2; x.c=(char)3; x.b; }));
  ASSERT(3, ({ struct {char a; int b; char c;} x; x.a=(char)1; x.b=2; x.c=(char)3; (int)x.c; }));

  ASSERT(3, ({ struct {int a; int b;} x[3]; x[0].a=1; x[1].b=2; x[0].a + x[1].b; }));
  // ASSERT(0, ({ struct {int a; int b;} x[3]; int *p=x; p[0]=0; x[0].a; }));
  // ASSERT(1, ({ struct {int a; int b;} x[3]; int *p=x; p[1]=1; x[0].b; }));
  // ASSERT(2, ({ struct {int a; int b;} x[3]; int *p=x; p[2]=2; x[1].a; }));
  // ASSERT(3, ({ struct {int a; int b;} x[3]; int *p=x; p[3]=3; x[1].b; }));

  // ASSERT(6, ({ struct {int a[3]; int b[5];} x; int *p=&x; x.a[0]=6; p[0]; }));
  // ASSERT(7, ({ struct {int a[3]; int b[5];} x; int *p=&x; x.b[0]=7; p[3]; }));

  ASSERT(6, ({ struct { struct { int b; } a; } x; x.a.b=6; x.a.b; }));

  // ASSERT(8, ({ struct {int a;} x; sizeof(x); }));
  // ASSERT(16, ({ struct {int a; int b;} x; sizeof(x); }));
  // ASSERT(16, ({ struct {int a, b;} x; sizeof(x); }));
  // ASSERT(24, ({ struct {int a[3];} x; sizeof(x); }));
  // ASSERT(32, ({ struct {int a;} x[4]; sizeof(x); }));
  // ASSERT(48, ({ struct {int a[3];} x[2]; sizeof(x); }));
  // ASSERT(2, ({ struct {char a; char b;} x; sizeof(x); }));
  // ASSERT(9, ({ struct {char a; int b;} x; sizeof(x); }));
  // ASSERT(0, ({ struct {} x; sizeof(x); }));

  return 0;
}
