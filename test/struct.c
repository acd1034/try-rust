#include "test.h"

int main()
{
  ASSERT(0, ({ struct {int a; int b;} x; 0; }));
  ASSERT(0, ({ struct {char a; char b;} x[3]; 0; }));

  return 0;
}
