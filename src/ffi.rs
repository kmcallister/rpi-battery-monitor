use libc;

extern "C" {
    pub fn gpio_init();
    fn gpio_read() -> libc::c_int;
}

pub fn read() -> bool {
    unsafe {
        gpio_read() != 0
    }
}
