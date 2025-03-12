#![feature(test)]

extern crate test;

use clap::Parser;
use fontgrep::cli;
use test::Bencher;

#[bench]
fn bench_name_ofl(b: &mut Bencher) {
    b.iter(|| {
        let args = cli::Cli::parse_from(["-n", "OFL", "testdata"]);
        cli::execute(args).unwrap();
    });
}
