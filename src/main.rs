#[macro_use]

extern crate lazy_static;
extern crate primes;
extern crate rug;

use crate::rug::ops::Pow;
use primes::PrimeSet;
use rug::Integer;

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
    .take_while(|i| i < &BASE)
    .collect();
  // static ref PRIMES : Vec<SmallInt> = vec![2, 3, 7];
}

fn digits(x: LargeInt) -> Vec<SmallInt> {
    let mut vec = Vec::new();
    let mut remaining = x.clone();
    while remaining > 0 {
        vec.push(large_to_small(remaining.clone() % small_to_large(BASE)));
        remaining /= small_to_large(BASE);
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

fn resistance(x: LargeInt) -> Resistance {
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

fn assemble(xs: Bins) -> LargeInt {
    let mut result = small_to_large(0);
    let mut mult = small_to_large(1);

    let mut bins = xs.clone();
    for i in (2..BASE).rev() {
        let factors = factor(i);
        while bins.iter().zip(factors.iter()).all(|(a, b)| a >= b) {
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
    let mut n = 1;
    let mut max_res = 0;
    let mut assembled_max = small_to_large(0);
    let mut iterations = 0;
    'outer: loop {
        println!("Starting new iteration: {}", n);

        for bins in StarsBars::from(n) {
            iterations += 1;
            let mut a = small_to_large(1);
            for (exponent, base) in bins.iter().zip(PRIMES.iter()) {
                a *= small_to_large(*base).pow(*exponent as u32);
            }

            // if (a > milestone) {
            //   println!("New milestone! {} {}", milestone_exponent, a);
            //   milestone *= 10;
            //   milestone_exponent += 1;
            // }

            let res = resistance(a);
            if res >= max_res {
                let assembled = assemble(bins);

                if res > max_res {
                    max_res = res;
                    println!("Found new max!          {}\t{}", res, assembled);
                    assembled_max = assembled;
                } else if assembled < assembled_max {
                    println!("Found better assembly:  {}\t{}", res, assembled);
                    assembled_max = assembled;
                }
            }
        }
        n += 1;

        if n > 60 {
            break;
        }
    }
    println!("Tested {} candidates.", iterations)
}
