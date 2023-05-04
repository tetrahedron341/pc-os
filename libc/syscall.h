#pragma once
#include <stdint.h>

uint64_t __syscall0(uint64_t op);
uint64_t __syscall1(uint64_t op, uint64_t arg0);
uint64_t __syscall2(uint64_t op, uint64_t arg0, uint64_t arg1);
uint64_t __syscall3(uint64_t op, uint64_t arg0, uint64_t arg1, uint64_t arg2);
uint64_t __syscall4(uint64_t op, uint64_t arg0, uint64_t arg1, uint64_t arg2, uint64_t arg3);