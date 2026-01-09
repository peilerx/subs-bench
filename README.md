# subs-bench 🚀

Performance study of **DOD (Structure of Arrays)** vs **OOP (Array of Structures)** using Rust's `portable_simd`. This project demonstrates the "Memory Wall" effect and how data layout impacts CPU cache efficiency and RAM bandwidth saturation.

## RAM-Locality Benchmark (20M Lines)

| Configuration | Serial (ms) | Parallel (ms) | 
| :--- | :---: | :---: | 
| **AoS XYZ** | 93.0 | 82.3 | 
| **AoS XY** | 75.9 | 68.9 | 
| **SoA XYZ** | 61.8 | 63.0 | 
| **SoA XY** | **41.1** | **41.3** |

## Cache-Locality Benchmark (30k Lines)

| Configuration | Serial (µs) | Parallel (µs) |
| :--- | :---: | :---: |
| **AoS XYZ** | 69.5 | 58.1 |
| **AoS XY** | 44.4 | 51.4 |
| **SoA XYZ** | 29.1 | 33.3 |
| **SoA XY** | **19.4** | **28.2** |

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
