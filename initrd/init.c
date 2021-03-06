#include <stdlib.h>
#include <stddef.h>
#include <sys/ping.h>
#include <sys/putchar.h>
#include <sys/sleep.h>

void print(const char *s, size_t len);
size_t itoa(int n, char *buf, size_t buflen);

void _start()
{
    for (int i = 0; i < 3; i++)
    {
        ping();
    }

    char text[] = "Hello from userland C!\n";
    print(text, sizeof text);

    for (int seconds = 0;; seconds++)
    {
        char numbuf[8];
        size_t numlen = itoa(seconds, numbuf, 8);
        print(numbuf, numlen);
        char suffix[] = " seconds\n";
        print(suffix, sizeof suffix);
        sleep_ms(1000);
    }

    abort();
}

void print(const char *s, size_t len)
{
    for (int i = 0; i < len; i++)
    {
        putchar(s[i]);
    }
}

size_t itoa(int n, char *buf, size_t buflen)
{
    size_t out_len = 0;
    for (int i = 0; i < buflen; i++)
    {
        char d;
        if (n == 0 && i != 0)
        {
            d = 0;
        }
        else
        {
            d = (n % 10) + 0x30;
            n /= 10;
            out_len += 1;
        }
        buf[i] = d;
    }
    // Reverse the order of digits
    for (int i = 0, j = out_len - 1; i <= j; i++, j--)
    {
        char tmp = buf[i];
        buf[i] = buf[j];
        buf[j] = tmp;
    }

    return out_len;
}