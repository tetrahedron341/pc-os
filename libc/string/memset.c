#include <string.h>

void *memset(void *ptr, int v, size_t len)
{
    unsigned char *bufptr = ptr;
    for (unsigned int i = 0; i < len; i++)
    {
        bufptr[i] = (unsigned char)v;
    }
    return ptr;
}