BINS = init
FILES = $(BINS) hello.txt
LDFLAGS = -T link.ld -nostdlib -L../libc -lc
CFLAGS = -I../libc/include 

.PHONY: clean

initrd.tar: $(FILES)
	tar c $^ -f initrd.tar

clean:
	@rm $(BINS) *.o initrd.tar

$(BINS) : % : %.o
	ld $< -o $@ $(LDFLAGS)

%.o: %.c
	cc -c $(CFLAGS) -o $*.o $*.c