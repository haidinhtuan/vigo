//! Benchmarks for Vigo Vietnamese input engine.
//!
//! Run with: cargo bench
//!
//! Benchmark categories:
//! - transform_*: One-shot string transformation
//! - engine_*: Character-by-character Engine (legacy)
//! - syllable_*: Character-by-character SyllableEngine (new)
//! - smart_*: SmartEngine with all features
//! - validation_*: Spell checking and validation

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId, Throughput};
use vigo::{Engine, transform_buffer, SyllableEngine, SmartEngine};
use vigo::action::InputMethod;
use vigo::validation::{is_valid_vietnamese, suggest_corrections};

// Test inputs of varying complexity
const SIMPLE_WORD: &str = "vieetj";           // việt
const MEDIUM_WORD: &str = "thuwowngf";        // thường  
const COMPLEX_WORD: &str = "nguwowifaf";      // người à (with extra)
const SHORT_SENTENCE: &str = "xin chaof";
const MEDIUM_SENTENCE: &str = "xin chaof cacs banj tooi laf nguwowif Vieetj Nam";
const LONG_SENTENCE: &str = "Hoom nay tooi ddi hocj tieengs Vieetj taji truwowngf ddaij hocj Basch khoa Haf Nooji vaf tooi raats vui vif dduowcj hocj cuungf vowis cacs banj";

// =============================================================================
// Transform benchmarks (one-shot string transformation)
// =============================================================================

fn bench_transform_simple(c: &mut Criterion) {
    c.bench_function("transform/simple_word", |b| {
        b.iter(|| transform_buffer(black_box(SIMPLE_WORD)))
    });
}

fn bench_transform_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("transform/throughput");
    
    for (name, input) in [
        ("simple", SIMPLE_WORD),
        ("medium", MEDIUM_WORD),
        ("complex", COMPLEX_WORD),
        ("short_sentence", SHORT_SENTENCE),
        ("medium_sentence", MEDIUM_SENTENCE),
        ("long_sentence", LONG_SENTENCE),
    ] {
        group.throughput(Throughput::Bytes(input.len() as u64));
        group.bench_with_input(BenchmarkId::new("transform", name), input, |b, input| {
            b.iter(|| transform_buffer(black_box(input)))
        });
    }
    group.finish();
}

// =============================================================================
// Legacy Engine benchmarks
// =============================================================================

fn bench_engine_simple(c: &mut Criterion) {
    c.bench_function("engine/simple_word", |b| {
        let mut engine = Engine::telex();
        b.iter(|| {
            engine.clear();
            for ch in SIMPLE_WORD.chars() {
                black_box(engine.feed(ch));
            }
            let _ = black_box(engine.output().len());
        })
    });
}

fn bench_engine_sentence(c: &mut Criterion) {
    c.bench_function("engine/medium_sentence", |b| {
        let mut engine = Engine::telex();
        b.iter(|| {
            engine.clear();
            for ch in MEDIUM_SENTENCE.chars() {
                black_box(engine.feed(ch));
            }
            let _ = black_box(engine.output().len());
        })
    });
}

fn bench_engine_vni(c: &mut Criterion) {
    c.bench_function("engine/vni_simple", |b| {
        let mut engine = Engine::vni();
        b.iter(|| {
            engine.clear();
            for ch in "vie6t5".chars() {
                black_box(engine.feed(ch));
            }
            let _ = black_box(engine.output().len());
        })
    });
}

// =============================================================================
// SyllableEngine benchmarks (new architecture)
// =============================================================================

fn bench_syllable_engine_simple(c: &mut Criterion) {
    c.bench_function("syllable_engine/simple_word", |b| {
        let mut engine = SyllableEngine::new(InputMethod::Telex);
        b.iter(|| {
            engine.clear();
            for ch in SIMPLE_WORD.chars() {
                black_box(engine.feed(ch));
            }
            black_box(engine.output())
        })
    });
}

fn bench_syllable_engine_sentence(c: &mut Criterion) {
    c.bench_function("syllable_engine/medium_sentence", |b| {
        let mut engine = SyllableEngine::new(InputMethod::Telex);
        b.iter(|| {
            engine.clear();
            for ch in MEDIUM_SENTENCE.chars() {
                black_box(engine.feed(ch));
            }
            black_box(engine.output())
        })
    });
}

fn bench_syllable_engine_long(c: &mut Criterion) {
    c.bench_function("syllable_engine/long_sentence", |b| {
        let mut engine = SyllableEngine::new(InputMethod::Telex);
        b.iter(|| {
            engine.clear();
            for ch in LONG_SENTENCE.chars() {
                black_box(engine.feed(ch));
            }
            black_box(engine.output())
        })
    });
}

fn bench_syllable_engine_vni(c: &mut Criterion) {
    c.bench_function("syllable_engine/vni_simple", |b| {
        let mut engine = SyllableEngine::new(InputMethod::Vni);
        b.iter(|| {
            engine.clear();
            for ch in "vie6t5".chars() {
                black_box(engine.feed(ch));
            }
            black_box(engine.output())
        })
    });
}

// =============================================================================
// SmartEngine benchmarks (with all features)
// =============================================================================

fn bench_smart_engine_simple(c: &mut Criterion) {
    c.bench_function("smart_engine/simple_word", |b| {
        let mut engine = SmartEngine::telex();
        b.iter(|| {
            engine.clear();
            for ch in SIMPLE_WORD.chars() {
                black_box(engine.feed(ch));
            }
            black_box(engine.output())
        })
    });
}

fn bench_smart_engine_sentence(c: &mut Criterion) {
    c.bench_function("smart_engine/medium_sentence", |b| {
        let mut engine = SmartEngine::telex();
        b.iter(|| {
            engine.clear();
            for ch in MEDIUM_SENTENCE.chars() {
                black_box(engine.feed(ch));
            }
            black_box(engine.output())
        })
    });
}

fn bench_smart_engine_with_commit(c: &mut Criterion) {
    c.bench_function("smart_engine/sentence_with_commits", |b| {
        let mut engine = SmartEngine::telex();
        b.iter(|| {
            engine.clear();
            for ch in MEDIUM_SENTENCE.chars() {
                if ch == ' ' {
                    black_box(engine.commit());
                } else {
                    black_box(engine.feed(ch));
                }
            }
            black_box(engine.commit())
        })
    });
}

// =============================================================================
// Validation benchmarks
// =============================================================================

fn bench_validation_valid(c: &mut Criterion) {
    c.bench_function("validation/is_valid_vietnamese", |b| {
        b.iter(|| {
            black_box(is_valid_vietnamese(black_box("việt")));
            black_box(is_valid_vietnamese(black_box("thường")));
            black_box(is_valid_vietnamese(black_box("người")));
        })
    });
}

fn bench_validation_invalid(c: &mut Criterion) {
    c.bench_function("validation/is_invalid_vietnamese", |b| {
        b.iter(|| {
            black_box(is_valid_vietnamese(black_box("tôy")));
            black_box(is_valid_vietnamese(black_box("xyz")));
            black_box(is_valid_vietnamese(black_box("email")));
        })
    });
}

fn bench_suggest_corrections(c: &mut Criterion) {
    c.bench_function("validation/suggest_corrections", |b| {
        b.iter(|| {
            black_box(suggest_corrections(black_box("tôy"), 5))
        })
    });
}

// =============================================================================
// Comparison benchmarks
// =============================================================================

fn bench_compare_engines(c: &mut Criterion) {
    let mut group = c.benchmark_group("compare/engines");
    
    group.bench_function("legacy_engine", |b| {
        let mut engine = Engine::telex();
        b.iter(|| {
            engine.clear();
            for ch in MEDIUM_SENTENCE.chars() {
                black_box(engine.feed(ch));
            }
            let _ = black_box(engine.output().len());
        })
    });
    
    group.bench_function("syllable_engine", |b| {
        let mut engine = SyllableEngine::new(InputMethod::Telex);
        b.iter(|| {
            engine.clear();
            for ch in MEDIUM_SENTENCE.chars() {
                black_box(engine.feed(ch));
            }
            black_box(engine.output())
        })
    });
    
    group.bench_function("smart_engine", |b| {
        let mut engine = SmartEngine::telex();
        b.iter(|| {
            engine.clear();
            for ch in MEDIUM_SENTENCE.chars() {
                black_box(engine.feed(ch));
            }
            black_box(engine.output())
        })
    });
    
    group.finish();
}

// =============================================================================
// Criterion groups
// =============================================================================

criterion_group!(
    transform_benches,
    bench_transform_simple,
    bench_transform_throughput,
);

criterion_group!(
    engine_benches,
    bench_engine_simple,
    bench_engine_sentence,
    bench_engine_vni,
);

criterion_group!(
    syllable_engine_benches,
    bench_syllable_engine_simple,
    bench_syllable_engine_sentence,
    bench_syllable_engine_long,
    bench_syllable_engine_vni,
);

criterion_group!(
    smart_engine_benches,
    bench_smart_engine_simple,
    bench_smart_engine_sentence,
    bench_smart_engine_with_commit,
);

criterion_group!(
    validation_benches,
    bench_validation_valid,
    bench_validation_invalid,
    bench_suggest_corrections,
);

criterion_group!(
    comparison_benches,
    bench_compare_engines,
);

criterion_main!(
    transform_benches,
    engine_benches,
    syllable_engine_benches,
    smart_engine_benches,
    validation_benches,
    comparison_benches,
);
