#![deny(warnings)]

extern crate libc;
extern crate time;

use std::env;
use time::precise_time_ns;
use ffi::{gpio_init, read};

mod sched;
mod ffi;

// Time out if anything takes more than this long.
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

// Bail out if we record more than this many edges (noise?)
const MAX_EDGES: usize = 3 * BITS;

// Signature for a good sample.
const SIGNATURE: u16 = 0xB400;

// Mask for signature.
const SIGNATURE_MASK: u16 = 0xFC00;

// Value of first resistor: BAT to input
const R1: f64 = 477000.0;

// Value of second resistor: input to GND
const R2: f64 = 118400.0;

// Input voltage of the microcontroller. This is the top of the ADC range.
const VCC: f64 = 3.30;

// Max ADC reading.
const MAX_ADC: f64 = 1023.0;

// Number of samples to average.
const SAMPLES: usize = 10;

// Error out if we take more than this many bad samples.
const MAX_BAD_SAMPLES: usize = 20;

#[derive(Debug)]
enum Error {
    TimeoutIdle,
    TimeoutWait,
    TooManyEdges,
    TooFewEdges,
    PrematureEnd,
    BadSignature,
}

// Take a single voltage reading from the sensor.
fn read_voltage() -> Result<f64, Error> {
    // We don't really need to allocate a new Vec for each reading.
    // But it's out of the realtime path and the time spent in this
    // function is dominated by waiting for data, anyway.
    let mut edges = Vec::with_capacity(MAX_EDGES);
    edges.push(0);

    //
    // REALTIME SECTION BEGINS HERE
    //
    // Keep this code tight because we're trying to capture
    // precise timings.
    {
        let _realtime = sched::Realtime::enter();

        // Wait for idle bus.
        let t0 = precise_time_ns();
        let mut tlast = t0;
        loop {
            let t = precise_time_ns();
            if (t - t0) >= TIMEOUT_NS {
                return Err(Error::TimeoutIdle);
            }

            if read() {
                tlast = t;
            } else if (t - tlast) >= IDLE_NS {
                break;
            }
        }

        // Wait for bus to go high: start of transmission.
        let mut t0 = precise_time_ns();
        loop {
            let t = precise_time_ns();
            if (t - t0) >= TIMEOUT_NS {
                return Err(Error::TimeoutWait);
            }

            if read() {
                t0 = t;
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
                edges.push(t - t0);
                if edges.len() > MAX_EDGES {
                    return Err(Error::TooManyEdges);
                }
                state = !state;
            }
        }
    }
    //
    // REALTIME SECTION ENDS HERE
    //

    if edges.len() < BITS {
        return Err(Error::TooFewEdges);
    }

    // Clock recovery. Some intervals between edges will be 2x the others;
    // average the short intervals.
    let deltas: Vec<_> = edges.windows(2).map(|x| x[1] - x[0]).collect();
    let threshold = (deltas.iter().max().unwrap() + deltas.iter().min().unwrap()) / 2;
    let (sum_low, num_low) = deltas.iter()
        .filter(|&&x| x < threshold)
        .fold((0, 0), |(sum, num), x| (sum + x, num + 1));
    let clock = sum_low / num_low;

    // Decoded value.
    let mut val = 0u16;

    // Sample in the middle of the first half of each bit,
    // skipping the first (always 1).
    // i.e. 2*clock + clock/2, 4*clock + clock/2, ...
    let mut bit = false;
    let mut t = 5*clock/2;
    let mut it = edges.iter().peekable();
    for _ in 0..BITS-1 {
        loop {
            match it.peek() {
                None => return Err(Error::PrematureEnd),
                Some(&&p) if p > t => break,
                _ => {
                    bit = !bit;
                    it.next().unwrap();  // must succeed, .peek() was Some
                }
            }
        }
        val >>= 1;
        if bit {
            val |= 0x8000;
        }
        t += 2*clock;
    }

    if (val & SIGNATURE_MASK) != SIGNATURE {
        return Err(Error::BadSignature);
    }
    val &= !SIGNATURE_MASK;

    Ok(((val as f64) / MAX_ADC) * ((R1 + R2) / R2) * VCC)
}

const MUNIN_CONFIG: &'static str
    = include_str!("munin.cfg");

fn main() {
    let mut args = env::args();
    if let None = args.next() {
        panic!("no argv[0]???");
    }
    match args.next() {
        Some(ref s) if s == "config" => {
            print!("{}", MUNIN_CONFIG);
            return;
        }
        Some(_) => panic!("unrecognized command line option"),
        None => (),
    }

    unsafe {
        gpio_init();
    }

    let mut sample_sum = 0.0;
    let mut good_samples = 0;
    let mut bad_samples = 0;
    loop {
        match read_voltage() {
            Ok(v) => {
                sample_sum += v;
                good_samples += 1;
                if good_samples >= SAMPLES {
                    break;
                }
            }
            Err(e) => {
                bad_samples += 1;
                if bad_samples > MAX_BAD_SAMPLES {
                    panic!("Too many bad samples! Last error: {:?}", e);
                }
            }
        }
    }

    let avg_voltage = sample_sum / (good_samples as f64);

    println!("battery_voltage.value {:6.3}", avg_voltage);
}
