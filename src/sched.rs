use libc;

pub struct Realtime {
    orig_policy: libc::c_int,
}

impl Realtime {
    /// Enter a realtime scheduler context.
    ///
    /// Returns to the original scheduler on drop.
    pub fn enter() -> Realtime {
        let policy = unsafe {
            libc::sched_getscheduler(0)
        };
        assert!(policy >= 0);

        let param = libc::sched_param {
            sched_priority: 50,
        };

        unsafe {
            assert_eq!(0, libc::sched_setscheduler(0, libc::SCHED_FIFO, &param));
        }

        Realtime {
            orig_policy: policy,
        }
    }
}

impl Drop for Realtime {
    fn drop(&mut self) {
        let param = libc::sched_param {
            sched_priority: 0,
        };

        unsafe {
            assert_eq!(0, libc::sched_setscheduler(0, self.orig_policy, &param));
        }
    }
}
