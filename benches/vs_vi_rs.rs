//! Head-to-head benchmark: Vigo vs vi-rs
//!
//! Compares the two Rust Vietnamese input engines across three dimensions:
//! 1. Batch transform (whole string at once)
//! 2. Incremental / character-by-character engine
//! 3. Throughput scaling with input length
//!
//! Run with: cargo bench --bench vs_vi_rs

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use vigo::action::InputMethod;

// Shared test inputs (Telex encoding)
const SIMPLE_WORD: &str = "vieetj";
const MEDIUM_WORD: &str = "thuwowngf";
const COMPLEX_WORD: &str = "nghieeeng";
const SHORT_SENTENCE: &str = "xin chaof";
const MEDIUM_SENTENCE: &str = "xin chaof cacs banj tooi laf nguwowif Vieetj Nam";
const LONG_SENTENCE: &str = "Hoom nay tooi ddi hocj tieengs Vieetj taji truwowngf ddaij hocj Basch khoa Haf Nooji vaf tooi raats vui vif dduowcj hocj cuungf vowis cacs banj";

// =============================================================================
// Batch transform: whole string at once
// =============================================================================

fn bench_batch_transform(c: &mut Criterion) {
    let mut group = c.benchmark_group("vs_vi_rs/batch");

    for (name, input) in [
        ("simple", SIMPLE_WORD),
        ("medium_word", MEDIUM_WORD),
        ("complex_word", COMPLEX_WORD),
        ("short_sentence", SHORT_SENTENCE),
        ("medium_sentence", MEDIUM_SENTENCE),
        ("long_sentence", LONG_SENTENCE),
    ] {
        group.bench_with_input(BenchmarkId::new("vigo", name), input, |b, input| {
            b.iter(|| vigo::transform_buffer(black_box(input)))
        });

        group.bench_with_input(BenchmarkId::new("vi_rs", name), input, |b, input| {
            b.iter(|| {
                let mut output = String::new();
                // vi-rs processes one word at a time, so split on spaces
                let words: Vec<&str> = input.split(' ').collect();
                for (i, word) in words.iter().enumerate() {
                    vi::transform_buffer(&vi::TELEX, black_box(word.chars()), &mut output);
                    if i < words.len() - 1 {
                        output.push(' ');
                    }
                }
                output
            })
        });
    }
    group.finish();
}

// =============================================================================
// Incremental: character-by-character engine
// =============================================================================

fn bench_incremental(c: &mut Criterion) {
    let mut group = c.benchmark_group("vs_vi_rs/incremental");

    for (name, input) in [
        ("simple", SIMPLE_WORD),
        ("medium_word", MEDIUM_WORD),
        ("short_sentence", SHORT_SENTENCE),
        ("medium_sentence", MEDIUM_SENTENCE),
        ("long_sentence", LONG_SENTENCE),
    ] {
        // Vigo SyllableEngine (char-by-char, handles spaces internally)
        group.bench_with_input(
            BenchmarkId::new("vigo_syllable", name),
            input,
            |b, input| {
                let mut engine = vigo::SyllableEngine::new(InputMethod::Telex);
                b.iter(|| {
                    engine.clear();
                    for ch in input.chars() {
                        black_box(engine.feed(ch));
                    }
                    black_box(engine.output())
                })
            },
        );

        // Vigo SmartEngine (char-by-char with validation)
        group.bench_with_input(
            BenchmarkId::new("vigo_smart", name),
            input,
            |b, input| {
                let mut engine = vigo::SmartEngine::telex();
                b.iter(|| {
                    engine.clear();
                    for ch in input.chars() {
                        if ch == ' ' {
                            black_box(engine.commit());
                        } else {
                            black_box(engine.feed(ch));
                        }
                    }
                    black_box(engine.output())
                })
            },
        );

        // vi-rs IncrementalBuffer (char-by-char, one word at a time)
        group.bench_with_input(
            BenchmarkId::new("vi_rs_incremental", name),
            input,
            |b, input| {
                let mut buffer = vi::methods::transform_buffer_incremental(&vi::TELEX);
                b.iter(|| {
                    let mut result = String::new();
                    buffer.clear();
                    for ch in input.chars() {
                        if ch == ' ' {
                            result.push_str(buffer.view());
                            result.push(' ');
                            buffer.clear();
                        } else {
                            buffer.push(ch);
                        }
                    }
                    result.push_str(buffer.view());
                    black_box(result)
                })
            },
        );
    }
    group.finish();
}

// =============================================================================
// Throughput: bytes/sec scaling
// =============================================================================

fn bench_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("vs_vi_rs/throughput");

    for (name, input) in [
        ("simple", SIMPLE_WORD),
        ("medium_word", MEDIUM_WORD),
        ("short_sentence", SHORT_SENTENCE),
        ("medium_sentence", MEDIUM_SENTENCE),
        ("long_sentence", LONG_SENTENCE),
    ] {
        group.throughput(Throughput::Bytes(input.len() as u64));

        group.bench_with_input(BenchmarkId::new("vigo", name), input, |b, input| {
            b.iter(|| vigo::transform_buffer(black_box(input)))
        });

        group.bench_with_input(BenchmarkId::new("vi_rs", name), input, |b, input| {
            b.iter(|| {
                let mut output = String::new();
                for word in input.split(' ') {
                    vi::transform_buffer(&vi::TELEX, black_box(word.chars()), &mut output);
                    output.push(' ');
                }
                output
            })
        });
    }
    group.finish();
}

// =============================================================================
// Criterion groups
// =============================================================================

criterion_group!(
    benches,
    bench_batch_transform,
    bench_incremental,
    bench_throughput,
);
criterion_main!(benches);
