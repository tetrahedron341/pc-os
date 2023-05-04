long long syscall(long long a, char ** target) {
    long long r14_out;
    char * r15_out;

    if (target == 0) {
        static char * null = (char *) 0;
        target = &null;
    }

    asm (
        "mov %2, %%r14 \n"
        "mov %3, %%r15 \n"
        "syscall \n"
        "mov %%r14, %0 \n"
        "mov %%r15, %1"

        : "=r" (r14_out),
          "=r" (r15_out)
        : "r" (a),
          "r" (*target)
    );

    *target = r15_out;
    return r14_out;
}