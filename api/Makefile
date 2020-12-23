OBJS = syscall.o
CFLAGS = -Iinclude -nostdlib -nodefaultlibs -c

.PHONY: all

all: libkernel.a

libkernel.a: $(OBJS)
	ar rcs libkernel.a $(OBJS)

%.o: %.c
	cc $(CFLAGS) -o $*.o $*.c