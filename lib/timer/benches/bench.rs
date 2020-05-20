#![cfg(test)]
#![feature(test)]

extern crate test;

use hop_internal_timer::Timer;
use test::Bencher;

#[bench]
#[cfg(feature = "__internal_test")]
fn bench_now(b: &mut Bencher) {
    b.iter(|| {
        hop_internal_timer::test_now();
    });
}
#[bench]
fn bench_run(b: &mut Bencher) {
    let mut timer = Timer::new();
    b.iter(|| {
        timer.start();
        timer.stop();
        timer.reset();
    });
}
