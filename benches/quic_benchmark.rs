// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
use aws_lc_ring_facade::{test, test_file};
use criterion::{criterion_group, criterion_main, Criterion};

#[derive(Debug)]
pub enum QuicAlgorithm {
    Aes128Gcm,
    Aes256Gcm,
    Chacha20,
}

pub struct QuicConfig {
    algorithm: QuicAlgorithm,
    key: Vec<u8>,
    sample: Vec<u8>,
    description: String,
}

impl QuicConfig {
    pub fn new(
        algorithm: QuicAlgorithm,
        key: &[u8],
        sample: &[u8],
        description: &str,
    ) -> QuicConfig {
        QuicConfig {
            algorithm,
            key: Vec::from(key),
            sample: Vec::from(sample),
            description: String::from(description),
        }
    }
}
macro_rules! benchmark_quic
{( $pkg:ident ) =>
{
    paste::item! {
        mod [<$pkg _benchmarks>]  {

            use $pkg::aead;
            use aead::quic;
            use criterion::black_box;

            fn algorithm(config: &crate::QuicConfig) -> &'static quic::Algorithm {
                black_box(match &config.algorithm {
                    crate::QuicAlgorithm::Aes128Gcm => &quic::AES_128,
                    crate::QuicAlgorithm::Aes256Gcm => &quic::AES_256,
                    crate::QuicAlgorithm::Chacha20 => &quic::CHACHA20,
                })
            }

            pub fn header_protection_key(config: &crate::QuicConfig) -> quic::HeaderProtectionKey {
                let algorithm = algorithm(config);
                quic::HeaderProtectionKey::new(algorithm, config.key.as_slice()).unwrap()
            }

            pub fn new_mask(key: &quic::HeaderProtectionKey, sample: &[u8]) {
                key.new_mask(sample).unwrap();
            }
        }
}}}

benchmark_quic!(ring);
benchmark_quic!(aws_lc_ring_facade);

fn test_new_mask(c: &mut Criterion, config: &QuicConfig) {
    let sample = config.sample.as_slice();

    let aws_key = aws_lc_ring_facade_benchmarks::header_protection_key(config);
    let aws_bench_name = format!(
        "aws-lc-{:?}-quic-new-mask: {} ({} bytes)",
        config.algorithm,
        config.description,
        sample.len()
    );
    c.bench_function(&aws_bench_name, |b| {
        b.iter(|| {
            let _result = aws_lc_ring_facade_benchmarks::new_mask(&aws_key, sample);
        })
    });

    let ring_key = ring_benchmarks::header_protection_key(config);
    let ring_bench_name = format!(
        "ring-{:?}-quic-new-mask: {} ({} bytes)",
        config.algorithm,
        config.description,
        sample.len()
    );
    c.bench_function(&ring_bench_name, |b| {
        b.iter(|| {
            let _result = ring_benchmarks::new_mask(&ring_key, sample);
        })
    });
}

fn test_aes_128(c: &mut Criterion) {
    test::run(
        test_file!("data/quic_aes_128_tests.txt"),
        |_section, test_case| {
            let config = QuicConfig::new(
                QuicAlgorithm::Aes128Gcm,
                test_case.consume_bytes("KEY").as_slice(),
                test_case.consume_bytes("SAMPLE").as_slice(),
                test_case.consume_string("DESC").as_str(),
            );
            println!("Testcase: {:?}", test_case);
            test_new_mask(c, &config);
            Ok(())
        },
    );
}

criterion_group!(benches, test_aes_128,);
criterion_main!(benches);
