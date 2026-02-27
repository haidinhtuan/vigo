//! Head-to-head benchmark: Vigo vs uvie-rs
//!
//! Both engines have an incremental feed(char) API, making this a direct
//! apples-to-apples comparison of core transformation speed.
//!
//! Run with: cargo bench --bench vs_uvie_rs

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
    let mut group = c.benchmark_group("vs_uvie/batch");

    for (name, input) in [
        ("simple", SIMPLE_WORD),
        ("medium_word", MEDIUM_WORD),
        ("complex_word", COMPLEX_WORD),
        ("short_sentence", SHORT_SENTENCE),
        ("medium_sentence", MEDIUM_SENTENCE),
        ("long_sentence", LONG_SENTENCE),
    ] {
        // Vigo batch transform
        group.bench_with_input(BenchmarkId::new("vigo", name), input, |b, input| {
            b.iter(|| vigo::transform_buffer(black_box(input)))
        });

        // Vigo FastEngine via feed loop
        group.bench_with_input(BenchmarkId::new("vigo_fast", name), input, |b, input| {
            let mut engine = vigo::FastEngine::telex();
            b.iter(|| {
                engine.clear();
                for ch in input.chars() {
                    let _ = engine.feed(ch);
                }
                black_box(engine.output().len())
            })
        });

        // uvie-rs via feed loop (no batch API)
        group.bench_with_input(BenchmarkId::new("uvie", name), input, |b, input| {
            let mut engine = uvie::UltraFastViEngine::new();
            engine.set_input_method(uvie::InputMethod::Telex);
            b.iter(|| {
                engine.clear();
                for ch in input.chars() {
                    let _ = engine.feed(ch);
                }
                // Flush last syllable with space
                let out = engine.feed(' ');
                black_box(out.len())
            })
        });
    }
    group.finish();
}

// =============================================================================
// Incremental: character-by-character engine
// =============================================================================

fn bench_incremental(c: &mut Criterion) {
    let mut group = c.benchmark_group("vs_uvie/incremental");

    for (name, input) in [
        ("simple", SIMPLE_WORD),
        ("medium_word", MEDIUM_WORD),
        ("short_sentence", SHORT_SENTENCE),
        ("medium_sentence", MEDIUM_SENTENCE),
        ("long_sentence", LONG_SENTENCE),
    ] {
        // Vigo SyllableEngine
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

        // Vigo SmartEngine
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

        // Vigo FastEngine (zero-allocation)
        group.bench_with_input(
            BenchmarkId::new("vigo_fast", name),
            input,
            |b, input| {
                let mut engine = vigo::FastEngine::telex();
                b.iter(|| {
                    engine.clear();
                    for ch in input.chars() {
                        let _ = engine.feed(ch);
                    }
                    black_box(engine.output().len())
                })
            },
        );

        // uvie-rs UltraFastViEngine
        group.bench_with_input(
            BenchmarkId::new("uvie", name),
            input,
            |b, input| {
                let mut engine = uvie::UltraFastViEngine::new();
                engine.set_input_method(uvie::InputMethod::Telex);
                b.iter(|| {
                    engine.clear();
                    for ch in input.chars() {
                        let _ = engine.feed(ch);
                    }
                    let out = engine.feed(' ');
                    black_box(out.len())
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
    let mut group = c.benchmark_group("vs_uvie/throughput");

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

        group.bench_with_input(BenchmarkId::new("vigo_fast", name), input, |b, input| {
            let mut engine = vigo::FastEngine::telex();
            b.iter(|| {
                engine.clear();
                for ch in input.chars() {
                    let _ = engine.feed(ch);
                }
                black_box(engine.output().len())
            })
        });

        group.bench_with_input(BenchmarkId::new("uvie", name), input, |b, input| {
            let mut engine = uvie::UltraFastViEngine::new();
            engine.set_input_method(uvie::InputMethod::Telex);
            b.iter(|| {
                engine.clear();
                for ch in input.chars() {
                    let _ = engine.feed(ch);
                }
                let out = engine.feed(' ');
                black_box(out.len())
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
