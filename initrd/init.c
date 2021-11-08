#include <stdlib.h>
#include <sys/ping.h>

void _start()
{
    for (int i = 0; i < 3; i++)
    {
        ping();
    }

    abort();
}