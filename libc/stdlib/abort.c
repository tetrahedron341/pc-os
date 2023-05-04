#include <stdint.h>
#include "uapi.h"
#include "stdlib.h"

__attribute__((__noreturn__)) void abort(void)
{
    while (1)
    {
        exit(0, NULL);
    }
}