CFLAGS?=-O2 -g
CFLAGS:=$(CFLAGS) -ffreestanding -nostdlib -Wall -Wextra -I../kernel-uapi/include -I. -I./include

OBJS=\
	string/memset.o \
	string/memcmp.o \
	string/memcpy.o \
	string/memmove.o \
	string/strlen.o \
	stdlib/abort.o \

LIBKERNEL_UAPI=../target/x86_64-pc_os/release/libkernel_uapi.a 

DEPS=\
	../kernel-uapi/include/uapi.h

.PHONY: all clean

BINARIES=libc.a

all: $(BINARIES)

libc.a: $(OBJS) $(LIBKERNEL_UAPI) $(DEPS)
	mkdir libkernel_objs; cd libkernel_objs; $(AR) x ../$(LIBKERNEL_UAPI)
	$(AR) rcs $@ $(OBJS) libkernel_objs/*
	rm -r libkernel_objs 

../kernel-uapi/include/uapi.h: 
	cd ../kernel-uapi; cbindgen -i include/uapi.h

$(LIBKERNEL_UAPI):
	cargo rustc --target ../x86_64-pc_os.json --release -p kernel-uapi -Zbuild-std=core --crate-type staticlib --features panic_handler

clean: 
	rm $(OBJS) $(BINARIES)
