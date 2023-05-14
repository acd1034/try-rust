#include "test.h"

int main()
{
  ASSERT(0, (int)""[0]);
  // ASSERT(1, sizeof(""));

  ASSERT(97, (int)"abc"[0]);
  ASSERT(98, (int)"abc"[1]);
  ASSERT(99, (int)"abc"[2]);
  ASSERT(0, (int)"abc"[3]);
  // ASSERT(4, sizeof("abc"));

  ASSERT(7, (int)"\a"[0]);
  ASSERT(8, (int)"\b"[0]);
  ASSERT(9, (int)"\t"[0]);
  ASSERT(10, (int)"\n"[0]);
  ASSERT(11, (int)"\v"[0]);
  ASSERT(12, (int)"\f"[0]);
  ASSERT(13, (int)"\r"[0]);

  ASSERT(106, (int)"\j"[0]);
  ASSERT(107, (int)"\k"[0]);
  ASSERT(108, (int)"\l"[0]);

  ASSERT(7, (int)"\ax\ny"[0]);
  ASSERT(120, (int)"\ax\ny"[1]);
  ASSERT(10, (int)"\ax\ny"[2]);
  ASSERT(121, (int)"\ax\ny"[3]);

  // ASSERT(0, (int)"\0"[0]);
  // ASSERT(16, (int)"\20"[0]);
  // ASSERT(65, (int)"\101"[0]);
  // ASSERT(104, (int)"\1500"[0]);
  // ASSERT(0, (int)"\x00"[0]);
  // ASSERT(119, (int)"\x77"[0]);

  return 0;
}
