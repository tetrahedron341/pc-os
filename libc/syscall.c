#include <stdint.h>
#include "syscall.h"

uint64_t __syscall0(uint64_t op)
{
  return __syscall4(op, 0, 0, 0, 0);
}

uint64_t __syscall1(uint64_t op, uint64_t arg0)
{
  return __syscall4(op, arg0, 0, 0, 0);
}

uint64_t __syscall2(uint64_t op, uint64_t arg0, uint64_t arg1)
{
  return __syscall4(op, arg0, arg1, 0, 0);
}

uint64_t __syscall3(uint64_t op, uint64_t arg0, uint64_t arg1, uint64_t arg2)
{
  return __syscall4(op, arg0, arg1, arg2, 0);
}

uint64_t __syscall4(uint64_t op, uint64_t arg0, uint64_t arg1, uint64_t arg2, uint64_t arg3)
{
  uint64_t rax_out;

  asm(
      "movq %1, %%rax \n"
      "movq %2, %%rdi \n"
      "movq %3, %%rsi \n"
      "movq %4, %%rdx \n"
      "movq %5, %%r8 \n"
      "syscall \n"
      "movq %%rax, %0 \n"

      : "=r"(rax_out)
      : "r"(op),
        "r"(arg0),
        "r"(arg1),
        "r"(arg2),
        "r"(arg3)
      : "rax", "rdi", "rsi", "rdx", "r8");

  return rax_out;
}