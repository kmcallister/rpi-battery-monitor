extern crate libc;
extern crate time;

use std::io;
use std::io::prelude::*;
use time::precise_time_ns;
use ffi::{gpio_init, read};

mod sched;
mod ffi;

// Timeout if anything takes more than this long.
const TIMEOUT_NS: u64 = 1_000_000_000;

// Bus is idle if low for this long.
const IDLE_NS: u64 = 10_000_000;

// Number of bits per transmission.
const BITS: usize = 17;

// Bits per second.
const BITRATE: u64 = 1000;

// Length of recording for each transmission.
// Double the ideal transmission time to account for clock inaccuracy.
const RECORD_NS: u64 = 2 * (1_000_000_000 / BITRATE) * (BITS as u64);

// Value of first resistor: BAT to input
const R1: f64 = 477000.0;

// Value of second resistor: input to GND
const R2: f64 = 118400.0;

// Input voltage of the microcontroller. This is the top of the ADC range.
const VCC: f64 = 3.30;

// Take a single voltage reading from the sensor.
fn read_voltage() -> Result<f64, ()> {
    let mut edges = Vec::with_capacity(2 * BITS);

    // Wait for idle bus.
    let t0 = precise_time_ns();
    let mut tlast = t0;
    loop {
        let t = precise_time_ns();
        if (t - t0) >= TIMEOUT_NS {
            return Err(());
        }

        if read() {
            tlast = t;
        } else if (t - tlast) >= IDLE_NS {
            break;
        }
    }

    // Wait for bus to go high: start of transmission.
    let t0 = precise_time_ns();
    loop {
        let t = precise_time_ns();
        if (t - t0) >= TIMEOUT_NS {
            return Err(());
        }

        if read() {
            break;
        }
    }

    // Record for the designated time period.
    let mut state = true;
    loop {
        let t = precise_time_ns();
        if (t - t0) >= RECORD_NS {
            break;
        }

        if read() != state {
            edges.push(t);
            state = !state;
        }
    }

    println!("{:?}", edges);

    Ok((0.0))
}

fn main() {
    sched::set_realtime();

    unsafe {
        gpio_init();
    }

    println!("{:?}", read_voltage());
}
