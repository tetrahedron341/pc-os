#include <kernel/syscall.h>

void _start()
{
    for (int i = 0; i < 7; i++)
    {
        syscall(0, (char *)0);
    }

    syscall(127, (char *)0);

    while (0)
    {
    }
}