#include <stdlib.h>
#include <sys/ping.h>
#include <sys/putchar.h>

void _start()
{
    for (int i = 0; i < 3; i++)
    {
        ping();
    }

    char *text = "Hello from userland C!\n";
    for (int i = 0; text[i]; i++) {
	putchar(text[i]);
    }

    abort();
}
