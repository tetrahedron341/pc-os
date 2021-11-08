#include <string.h>

int memcmp(const void *ptr1, const void *ptr2, size_t len)
{
    const unsigned char *a = ptr1;
    const unsigned char *b = ptr2;

    for (unsigned int i = 0; i < len; i++)
    {
        if (a[i] > b[i])
        {
            return 1;
        }
        else if (a[i] < b[i])
        {
            return -1;
        }
    }
    return 0;
}