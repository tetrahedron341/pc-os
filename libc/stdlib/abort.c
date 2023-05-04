#include <stdlib.h>
#include <stdint.h>
#include <kernel/uapi/uapi_syscall.h>
#include "syscall.h"

__attribute__((__noreturn__)) void abort(void)
{
    syscall(SYS_EXIT, (char *)0);
    while (1)
    {
    }
}