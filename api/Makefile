OBJS = syscall.o
CFLAGS = -Iinclude -I../kernel/include -nostdlib -nodefaultlibs -c

.PHONY: all

all: libkernel.a

libkernel.a: $(OBJS)
	ar rcs libkernel.a $(OBJS)

%.o: %.c
	cc $(CFLAGS) -o $*.o $*.c