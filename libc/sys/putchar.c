#include <stdint.h>
#include "syscall.h"
#include "kernel/uapi/uapi_syscall.h"

int putchar(unsigned char c) {
    uint64_t err = syscall(SYS_PUTCHAR | (uint64_t)c << 8, (char*)0);
    if (err) {
	    return 1;
    } else {
	    return 0;
    }
}
