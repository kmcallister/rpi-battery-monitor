use libc;

pub fn set_realtime() {
    let param = libc::sched_param {
        sched_priority: 50,
    };

    unsafe {
        assert_eq!(0, libc::sched_setscheduler(0, libc::SCHED_FIFO, &param));
    }
}
