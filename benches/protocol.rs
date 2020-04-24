#![feature(test)]

extern crate imms;
extern crate test;

#[cfg(test)]
mod benches {
    use imms::protocol::*;
    use test::Bencher;

    fn command(amount: u8, bytes: &[u8]) -> String {
        let mut cmd = Vec::with_capacity(1 + bytes.len());
        cmd.insert(0, amount);
        cmd.extend(bytes);

        unsafe { String::from_utf8_unchecked(cmd) }
    }

    #[bench]
    fn decr(b: &mut Bencher) {
        let command = command(1, b"\ndecr\nfoo");

        b.iter(|| {
            parse_command(&command).unwrap();
        });
    }

    #[bench]
    fn test_100m(b: &mut Bencher) {
        let cmd = command(1, b"\nincr\nfoo\n");
        let r = cmd.as_ref();

        let now = ::std::time::Instant::now();

        for _ in 0 .. 100_000_000 {
            parse_command(r);
        }

        panic!("elapsed: {:?}", now.elapsed());
    }
}
