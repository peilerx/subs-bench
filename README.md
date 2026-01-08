# subs-bench 🚀

Performance study of **DOD (Structure of Arrays)** vs **OOP (Array of Structures)** using Rust's `portable_simd`. This project demonstrates the "Memory Wall" effect and how data layout impacts CPU cache efficiency and RAM bandwidth saturation.

## Full Benchmark Results (20,000,000 Lines)

| Configuration | Serial (ms) | Parallel (ms) | Speedup (vs Base) |
| :--- | :--- | :--- | :--- |
| **AoS XYZ** | 92.6 | 82.3 | 1.00x (Baseline) |
| **AoS XY** | 75.0 | 68.1 | 1.23x |
| **SoA XYZ** (SIMD) | 81.6 | 83.8 | 1.13x |
| **SoA XY** (SIMD) | **54.2** | **54.1** | **1.71x** |

---

## Hardware Environment
* **CPU:** Intel Core i5-7xxxU (2 Cores / 4 Threads)
* **RAM:** 8GB DDR4 @ 2400 MHz (Dual Channel)
* **OS:** Linux
* **Toolchain:** Rust 1.94.0-nightly (required for `portable_simd`)

---

## How to Run

To reproduce these benchmarks, ensure no background processes are heavy on memory and run:

```bash
RUSTFLAGS="-C target-cpu=native -C opt-level=3" cargo +nightly bench
