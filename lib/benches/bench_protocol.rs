#![cfg(test)]
#![feature(test)]

extern crate test;

use hop_lib::command::protocol::Context;
use test::Bencher;

#[bench]
fn bench_simple_increment(b: &mut Bencher) {
    let mut ctx = Context::new();
    let mut input = test::black_box([0].to_vec());

    b.iter(|| {
        ctx.feed(&input);
    });
}
