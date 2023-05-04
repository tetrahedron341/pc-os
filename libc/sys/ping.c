#include <sys/ping.h>
#include <kernel/uapi/uapi_syscall.h>
#include <stdint.h>
#include "syscall.h"

void ping()
{
    __syscall0(SYS_PING);
}