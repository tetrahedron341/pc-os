CFLAGS?=-O2 -g
CFLAGS:=$(CFLAGS) -ffreestanding -nostdlib -Wall -Wextra -I../kernel/include -I. -I./include

OBJS=\
	string/memset.o \
	string/memcmp.o \
	string/memcpy.o \
	string/memmove.o \
	string/strlen.o \
	stdlib/abort.o \
	syscall.o \
	sys/ping.o \

.PHONY: all clean

BINARIES=libc.a

all: $(BINARIES)

libc.a: $(OBJS)
	$(AR) rcs $@ $(OBJS)

clean: 
	rm $(OBJS) $(BINARIES)