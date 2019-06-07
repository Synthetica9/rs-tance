#[macro_use]

extern crate lazy_static;
extern crate primes;
extern crate rug;
extern crate ctrlc;

use std::time::Instant;
use crate::rug::ops::Pow;
use primes::PrimeSet;
use rug::Integer;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::env;

type SmallInt = u8;
type LargeInt = Integer;

type Resistance = i32;
type Bins = Vec<usize>;

static BASE: SmallInt = 10;

fn small_to_large(x: SmallInt) -> LargeInt {
    LargeInt::from(x)
}

fn large_to_small(x: LargeInt) -> SmallInt {
    (x.to_u32_wrapping()) as SmallInt
}

struct StarsBars {
    bins: Bins,
    finished: bool,
}

fn empty_bins() -> Bins {
    vec![0; PRIMES.len()]
}

impl StarsBars {
    fn from(n: usize) -> StarsBars {
        let mut vec = empty_bins();
        vec[0] = n;
        StarsBars {
            bins: vec,
            finished: false,
        }
    }
}

impl Iterator for StarsBars {
    type Item = Vec<usize>;

    fn next(&mut self) -> Option<Bins> {
        if self.finished {
            None
        } else {
            let old_bins = self.bins.clone();
            let mut broken = false;
            for (i, val) in self.bins[..self.bins.len() - 1].iter().enumerate() {
                if *val != 0 {
                    self.bins[i + 1] += 1;
                    self.bins[i] -= 1;
                    let mut sum = 0;
                    for j in 0..i + 1 {
                        sum += self.bins[j];
                        self.bins[j] = 0;
                    }
                    self.bins[0] = sum;
                    broken = true;
                    break;
                }
            }
            if !broken {
                self.finished = true;
            }
            Some(old_bins)
        }
    }
}

lazy_static! {
  static ref PRIMES : Vec<SmallInt> = PrimeSet::new()
    .iter()
    .map(|i| i as SmallInt)
    .take_while(|i| i <= &BASE)
    .collect();
}

fn digits(x: LargeInt) -> Vec<SmallInt> {
    let mut vec = Vec::new();
    let mut remaining = x.clone();
    while remaining > 0 {
        let mut quot = small_to_large(BASE);
        LargeInt::div_rem_mut(&mut remaining, &mut quot);
        vec.push(large_to_small(quot));
    }
    vec.reverse();
    return vec;
}

fn digit_product(x: LargeInt) -> LargeInt {
    let mut result = small_to_large(1);
    for digit in digits(x) {
        result *= small_to_large(digit);
    }
    return result;
}

fn resistance(x: &LargeInt) -> Resistance {
    let mut res = 0;
    let mut remaining = x.clone();

    while remaining >= small_to_large(BASE) {
        remaining = digit_product(remaining);
        res += 1;
    }

    return res;
}

fn factor(x: SmallInt) -> Bins {
    let mut remaining = x;
    let mut bins = empty_bins();
    for (i, p) in PRIMES.iter().enumerate() {
        while (remaining % p) == 0 {
            remaining /= p;
            bins[i] += 1;
        }
    }
    bins
}

lazy_static! {
    static ref BASE_FACTORS : Bins = factor(BASE);
}

fn divides(xs: &Bins, ys: &Bins) -> bool {
    xs.iter().zip(ys.iter()).all(|(a, b)| a >= b)
}

fn assemble(xs: &Bins) -> LargeInt {
    let mut result = small_to_large(0);
    let mut mult = small_to_large(1);

    let mut bins = xs.clone();
    for i in (2..BASE).rev() {
        let factors = factor(i);
        while divides(&bins, &factors) {
            result += mult.clone() * small_to_large(i);
            mult *= small_to_large(BASE);
            for (j, val) in factors.iter().enumerate() {
                bins[j] -= val;
            }
        }
    }
    result
}

fn main() {
    println!("Starting");

    let running = Arc::new(AtomicBool::new(true));
    {
        let r = running.clone();

        ctrlc::set_handler(move || {
            r.store(false, Ordering::SeqCst);
        }).expect("Error setting Ctrl-C handler");
    }

    let mut max_res = 0;
    let mut assembled_max = small_to_large(0);
    let mut iterations = 0;
    let mut discriminations = 0;

    let now = Instant::now();

    let break_on = {
        let argv : Vec<String> = env::args().collect();
        match argv.len() {
            1 => !0,
            2 => argv[1].parse::<usize>().unwrap(),
            _ => panic!("at the disco"),
        }
    };

    let mut n = 1;
    'outer: while n < break_on {
        for bins in StarsBars::from(n) {
            iterations += 1;

            // Like the 5 and 2 example, will get canceled out quickly, so skip immediately.
            // Achieves a huge speedup, ~4x
            if divides(&bins, &BASE_FACTORS) {
                discriminations += 1;
                continue;
            }

            let mut a = small_to_large(1);
            for (exponent, base) in bins.iter().zip(PRIMES.iter()) {
                a *= small_to_large(*base).pow(*exponent as u32);
            }

            let res = resistance(&a) + 1; // One pass is done above.
            if res >= max_res {
                let assembled = assemble(&bins);
                let native_assembled = assembled.to_string_radix(BASE as i32);

                if res > max_res {
                    println!("Found new max!          {}\t{} ({})", res, native_assembled, assembled);
                    max_res = res;
                    assembled_max = assembled;
                } else if assembled < assembled_max {
                    println!("Found better assembly:  {}\t{} ({})", res, native_assembled, assembled);
                    assembled_max = assembled;
                }

            }
            if !running.load(Ordering::SeqCst) {
                println!("Investigating before break: {}", a);
                break 'outer;
            };
        }
        n += 1;
    }

    let duration = now.elapsed();
    println!();
    println!("Ran for {} cycles", n);
    println!("Tested {} candidates.", iterations);
    println!("Discriminated: {} ({:.2}%)", discriminations, (100. * discriminations as f64)/ (iterations as f64));
    println!("Total: {}ms", duration.as_millis());
    println!("Per candidate: {}ns", duration.as_nanos()/iterations);
    println!("Per nondiscriminated candidate: {}ns", duration.as_nanos()/(iterations - discriminations));
}
