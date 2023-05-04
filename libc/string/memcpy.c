#include <stddef.h>

void *memcpy(void *__restrict dst, const void *__restrict src, size_t len)
{
    char *__restrict dst2 = dst;
    const char *__restrict src2 = src;
    for (unsigned int i = 0; i < len; i++)
    {
        dst2[i] = src2[i];
    }
    return dst;
}