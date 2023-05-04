BINS = init
FILES = $(BINS) hello.txt
LDFLAGS = -nostdlib -L../libc -lc 
CFLAGS = -I../libc/include -I../kernel-uapi/include -ffreestanding

.PHONY: clean libc

initrd.tar: $(FILES)
	tar c $^ -f initrd.tar

clean:
	@rm $(BINS) *.o initrd.tar

$(BINS) : % : %.o
	ld $< -o $@ $(LDFLAGS)

%.o: libc %.c
	cc -c $(CFLAGS) -o $*.o $*.c

libc:
	make -C ../libc