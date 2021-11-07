#include "kernel/syscall.h"

uint64_t syscall(uint64_t a, char *target)
{
  long long r14_out;
  char *r15_out;

  asm(
      "movq %2, %%r14 \n"
      "movq %3, %%r15 \n"
      "syscall \n"
      "movq %%r14, %0 \n"
      "movq %%r15, %1"

      : "=r"(r14_out),
        "=r"(r15_out)
      : "r"(a),
        "r"(target));

  return r14_out;
}