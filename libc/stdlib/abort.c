#include <stdlib.h>
#include <stdint.h>
#include <kernel/uapi/uapi_syscall.h>
#include "syscall.h"

__attribute__((__noreturn__)) void abort(void)
{
    __syscall1(SYS_EXIT, 0);
    while (1)
    {
    }
}