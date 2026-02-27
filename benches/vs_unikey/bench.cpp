/*
 * Head-to-head benchmark: Vigo vs Unikey
 *
 * Measures raw engine throughput for Vietnamese input transformation.
 * Both engines process the same keystrokes using Telex input method.
 *
 * Build: see CMakeLists.txt in this directory
 * Run:   ./bench_vs_unikey
 */

#include <chrono>
#include <cstdio>
#include <cstring>
#include <string>
#include <vector>

// Unikey engine
#include "unikey.h"

// Vigo engine (C FFI)
extern "C" {
#include "vigo.h"
}

struct TestCase {
    const char *name;
    const char *input;
    const char *expected;
};

static const std::vector<TestCase> tests = {
    {"simple word",     "vieetj",                                          "việt"},
    {"two words",       "xin chaof",                                       "xin chào"},
    {"medium",          "cacs banj tooi laf Vieetj Nam",
                        "các bạn tôi là Việt Nam"},
    {"long sentence",   "Hoom nay tooi ddi hocj tieengs Vieetj tooi raats vui vaf thichs hocj",
                        "Hôm nay tôi đi học tiếng Việt tôi rất vui và thích học"},
    {"capitalized",     "Vieejt Nam",                                      "Việt Nam"},
    {"all caps",        "VIEEJT NAM",                                      "VIỆT NAM"},
};

static const int WARMUP_ITERS = 1000;
static const int BENCH_ITERS = 100000;

// ============================================================================
// Unikey benchmark
// ============================================================================

// Remove count UTF-8 codepoints from end of string
static void erase_chars(std::string &s, int count) {
    int i = s.length();
    while (i > 0 && count > 0) {
        unsigned char code = s[i - 1];
        if ((code >> 6) != 2) { // not a continuation byte
            count--;
        }
        i--;
    }
    s.erase(i);
}

static std::string unikey_process(const char *input) {
    UnikeyResetBuf();
    std::string preedit;

    for (const char *p = input; *p; ++p) {
        if (*p == ' ') {
            // Word break: commit preedit and reset
            UnikeyResetBuf();
            if (!preedit.empty()) {
                preedit += ' ';
            } else {
                preedit += ' ';
            }
            continue;
        }

        UnikeySetCapsState(0, 0);
        UnikeyFilter((unsigned int)*p);

        // Process backspaces (remove from preedit)
        if (UnikeyBackspaces > 0) {
            if (preedit.length() <= (size_t)UnikeyBackspaces) {
                preedit.clear();
            } else {
                erase_chars(preedit, UnikeyBackspaces);
            }
        }

        // Append new output from engine
        if (UnikeyBufChars > 0) {
            preedit.append((const char *)UnikeyBuf, UnikeyBufChars);
        } else {
            // Engine didn't process it — append raw character
            preedit += *p;
        }
    }

    return preedit;
}

static double bench_unikey(const char *input, int iters) {
    auto start = std::chrono::high_resolution_clock::now();
    for (int i = 0; i < iters; ++i) {
        auto result = unikey_process(input);
        asm volatile("" : : "r"(result.data()) : "memory");
    }
    auto end = std::chrono::high_resolution_clock::now();
    return std::chrono::duration<double, std::micro>(end - start).count() / iters;
}

// ============================================================================
// Vigo benchmark
// ============================================================================

static std::string vigo_process(const char *input) {
    vigo_engine_t *engine = vigo_new_telex();
    std::string result;

    for (const char *p = input; *p; ++p) {
        if (*p == ' ') {
            char *out = vigo_commit(engine);
            if (out) {
                result += out;
                vigo_free_string(out);
            }
            result += ' ';
            continue;
        }
        vigo_feed(engine, (uint32_t)*p);
    }

    // Final commit
    char *out = vigo_commit(engine);
    if (out) {
        result += out;
        vigo_free_string(out);
    }

    vigo_free(engine);
    return result;
}

static double bench_vigo(const char *input, int iters) {
    // Reuse engine across iterations for fair comparison
    vigo_engine_t *engine = vigo_new_telex();

    auto start = std::chrono::high_resolution_clock::now();
    for (int i = 0; i < iters; ++i) {
        vigo_clear(engine);
        for (const char *p = input; *p; ++p) {
            if (*p == ' ') {
                char *out = vigo_commit(engine);
                if (out) vigo_free_string(out);
                continue;
            }
            vigo_feed(engine, (uint32_t)*p);
        }
        char *out = vigo_commit(engine);
        if (out) {
            asm volatile("" : : "r"(out) : "memory");
            vigo_free_string(out);
        }
    }
    auto end = std::chrono::high_resolution_clock::now();

    vigo_free(engine);
    return std::chrono::duration<double, std::micro>(end - start).count() / iters;
}

// ============================================================================
// Main
// ============================================================================

int main() {
    // Initialize unikey
    UnikeySetup();
    UnikeySetInputMethod(UkTelex);
    // UnikeySetup() defaults to XUTF8 (charset 12) which is correct

    printf("=== Vigo vs Unikey Benchmark ===\n\n");

    // Correctness check
    printf("Correctness check:\n");
    printf("%-20s %-30s %-30s %-30s\n", "Test", "Expected", "Vigo", "Unikey");
    printf("%-20s %-30s %-30s %-30s\n", "----", "--------", "----", "------");

    for (auto &tc : tests) {
        auto vigo_out = vigo_process(tc.input);
        auto unikey_out = unikey_process(tc.input);

        const char *vigo_mark = (vigo_out == tc.expected) ? " OK" : " FAIL";
        const char *unikey_mark = (unikey_out == tc.expected) ? " OK" : " DIFF";

        printf("%-20s %-30s %-27s%-3s %-27s%-3s\n",
               tc.name, tc.expected,
               vigo_out.c_str(), vigo_mark,
               unikey_out.c_str(), unikey_mark);
    }

    printf("\nBenchmark (%d iterations each):\n", BENCH_ITERS);
    printf("%-20s %12s %12s %12s\n", "Test", "Vigo (µs)", "Unikey (µs)", "Ratio");
    printf("%-20s %12s %12s %12s\n", "----", "---------", "-----------", "-----");

    // Warmup
    for (auto &tc : tests) {
        bench_vigo(tc.input, WARMUP_ITERS);
        bench_unikey(tc.input, WARMUP_ITERS);
    }

    for (auto &tc : tests) {
        double vigo_us = bench_vigo(tc.input, BENCH_ITERS);
        double unikey_us = bench_unikey(tc.input, BENCH_ITERS);
        double ratio = vigo_us / unikey_us;

        printf("%-20s %10.2f µs %10.2f µs %10.2fx\n",
               tc.name, vigo_us, unikey_us, ratio);
    }

    printf("\nNote: Ratio < 1.0 means Vigo is faster, > 1.0 means Unikey is faster.\n");
    printf("Both engines are measured in microseconds per iteration.\n");
    printf("At 60 WPM, one keystroke arrives every ~100,000 µs.\n");

    UnikeyCleanup();
    return 0;
}
