pub fn loop_forever() -> ! {
    loop {
        x86_64::instructions::hlt()
    }
}
