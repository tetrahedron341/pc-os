#include <stddef.h>

void *memmove(void *dst, const void *src, size_t len)
{
    char *dst2 = dst;
    const char *src2 = src;
    if (dst < src)
    {
        for (unsigned int i = 0; i < len; i++)
        {
            dst2[i] = src2[i];
        }
    }
    else
    {
        for (unsigned int i = len; i > 0; i--)
        {
            dst2[i - 1] = src2[i - 1];
        }
    }
    return dst;
}