#include <kernel/uapi/uapi_syscall.h>
#include <stdint.h>
#include "syscall.h"

void sleep_ms(uint32_t ms)
{
    syscall(SYS_SLEEP_MS | (uint64_t)ms << 32, (char *)0);
}
