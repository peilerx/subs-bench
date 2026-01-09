use criterion::{black_box, criterion_group, criterion_main, Criterion};

use subs_bench::*;

pub fn subs_bench(c: &mut Criterion) {
    let size = 30_000;
    let size = 20_000_000;
    let t = 0.42;

    let mut aos_lines = Vec::with_capacity(size);
    let mut soa_lines = Lines {
        tx: Vec::with_capacity(size), ty: Vec::with_capacity(size), tz: Vec::with_capacity(size),
    };

    let mut rx_init = vec![0.0; size];
    let mut ry_init = vec![0.0; size];
    let mut rz_init = vec![0.0; size];

    use std::f64::consts::PI;
    let golden_ratio = (1.0 + 5.0_f64.sqrt()) / 2.0;
    for i in 0..size {
        let theta = 2.0 * PI * (i as f64) / golden_ratio;
        let phi = (1.0 - 2.0 * (i as f64 + 0.5) / size as f64).acos();
        let x = phi.sin() * theta.cos();
        let y = phi.sin() * theta.sin();
        let z = phi.cos();
        let p0 = Point { x, y, z };
        let p1 = Point { x: x * 1.5, y: y * 1.5, z: z * 1.5 };

        aos_lines.push(Line(p0, p1));
        soa_lines.tx.push(p1.x); soa_lines.ty.push(p1.y); soa_lines.tz.push(p1.z);
        rx_init[i] = p0.x; ry_init[i] = p0.y; rz_init[i] = p0.z;
    }

    let mut aos_out = vec![Point { x: 0.0, y: 0.0, z: 0.0 }; size];
    let mut rx = rx_init.clone();
    let mut ry = ry_init.clone();
    let mut rz = rz_init.clone();

    let mut group = c.benchmark_group("subs_bench_20M_DOD_Final");
    group.sample_size(10);

    // --- SECTION 1: SERIAL ---
    group.bench_function("Serial_AoS_XYZ", |b| b.iter(||
    serial_subs_all_axis(black_box(&aos_lines), black_box(t), black_box(&mut aos_out))
    ));
    group.bench_function("Serial_AoS_XY", |b| b.iter(||
    serial_subs_xy(black_box(&aos_lines), black_box(t), black_box(&mut rx), black_box(&mut ry))
    ));
    group.bench_function("Serial_SoA_XYZ", |b| b.iter(||
    serial_prep_subs_all_axis(black_box(&soa_lines), black_box(t), black_box(&mut rx), black_box(&mut ry), black_box(&mut rz))
    ));
    group.bench_function("Serial_SoA_XY", |b| b.iter(||
    serial_prep_subs_xy(black_box(&soa_lines.tx), black_box(&soa_lines.ty), black_box(t), black_box(&mut rx), black_box(&mut ry))
    ));

    // --- SECTION 2: PARALLEL (2 CORES) ---
    rayon::ThreadPoolBuilder::new().num_threads(2).build_global().unwrap_or(());

    group.bench_function("Parallel_AoS_XYZ", |b| b.iter(||
    parallel_subs_all_axis(black_box(&aos_lines), black_box(t), black_box(&mut aos_out))
    ));
    group.bench_function("Parallel_AoS_XY", |b| b.iter(||
    parallel_subs_xy(black_box(&aos_lines), black_box(t), black_box(&mut rx), black_box(&mut ry))
    ));
    group.bench_function("Parallel_SoA_XYZ", |b| b.iter(||
    parallel_prep_subs_all_axis(black_box(&soa_lines), black_box(t), black_box(&mut rx), black_box(&mut ry), black_box(&mut rz))
    ));
    group.bench_function("Parallel_SoA_XY", |b| b.iter(||
    parallel_prep_subs_xy(black_box(&soa_lines.tx), black_box(&soa_lines.ty), black_box(t), black_box(&mut rx), black_box(&mut ry))
    ));

    // --- SECTION 3: STREAMING ---
    group.bench_function("Stream_SoA_XYZ", |b| b.iter(||
    serial_stream_subs_all_axis(black_box(&soa_lines), black_box(t), black_box(&mut rx), black_box(&mut ry), black_box(&mut rz))
    ));
    group.bench_function("Stream_SoA_XY", |b| b.iter(||
    serial_stream_subs_xy(black_box(&soa_lines), black_box(t), black_box(&mut rx), black_box(&mut ry))
    ));

    group.finish();
}

criterion_group!(benches, subs_bench);
criterion_main!(benches);
