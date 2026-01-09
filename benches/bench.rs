#![feature(portable_simd)]
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rayon::prelude::*;
use std::simd::prelude::*;
use std::simd::StdFloat;

#[derive(Copy, Clone)]
struct Point { x: f64, y: f64, z: f64 }

#[derive(Copy, Clone)]
struct Line(Point, Point);

struct Lines {
    tx: Vec<f64>, ty: Vec<f64>, tz: Vec<f64>,
}

#[inline(always)]
pub fn simd_subs(target: &[f64], t: f64, out: &mut [f64]) {
    assert_eq!(target.len(), out.len());

    let t_v = f64x4::splat(t);
    let (t_prefix, t_simd, t_suffix) = target.as_simd::<4>();
    let (out_prefix, out_simd_mut, out_suffix) = out.as_simd_mut::<4>();

    out_prefix.iter_mut().zip(t_prefix.iter()).for_each(|(res, &trg)| {
        *res = *res + (trg - *res) * t;
    });

    let mut t_chunks = t_simd.chunks_exact(8);
    let mut out_chunks = out_simd_mut.chunks_exact_mut(8);

    for (t_c, out_c) in t_chunks.by_ref().zip(out_chunks.by_ref()) {
        out_c[0] = (t_c[0] - out_c[0]).mul_add(t_v, out_c[0]);
        out_c[1] = (t_c[1] - out_c[1]).mul_add(t_v, out_c[1]);
        out_c[2] = (t_c[2] - out_c[2]).mul_add(t_v, out_c[2]);
        out_c[3] = (t_c[3] - out_c[3]).mul_add(t_v, out_c[3]);
        out_c[4] = (t_c[4] - out_c[4]).mul_add(t_v, out_c[4]);
        out_c[5] = (t_c[5] - out_c[5]).mul_add(t_v, out_c[5]);
        out_c[6] = (t_c[6] - out_c[6]).mul_add(t_v, out_c[6]);
        out_c[7] = (t_c[7] - out_c[7]).mul_add(t_v, out_c[7]);
    }

    out_chunks.into_remainder().iter_mut().zip(t_chunks.remainder().iter()).for_each(|(outv, &tv)| {
        *outv = (tv - *outv).mul_add(t_v, *outv);
    });

    out_suffix.iter_mut().zip(t_suffix.iter()).for_each(|(res, &trg)| {
        *res = *res + (trg - *res) * t;
    });
}

fn serial_subs_all_axis(lines: &[Line], t: f64, out: &mut [Point]) {
    for i in 0..lines.len() {
        let line = &lines[i];
        out[i].x = line.0.x + (line.1.x - line.0.x) * t;
        out[i].y = line.0.y + (line.1.y - line.0.y) * t;
        out[i].z = line.0.z + (line.1.z - line.0.z) * t;
    }
}

fn parallel_subs_all_axis(lines: &[Line], t: f64, out: &mut [Point]) {
    lines.par_iter().zip(out.par_iter_mut()).for_each(|(line, p)| {
        p.x = line.0.x + (line.1.x - line.0.x) * t;
        p.y = line.0.y + (line.1.y - line.0.y) * t;
        p.z = line.0.z + (line.1.z - line.0.z) * t;
    });
}

fn serial_subs_xy(lines: &[Line], t: f64, rx: &mut [f64], ry: &mut [f64]) {
    for i in 0..lines.len() {
        let line = &lines[i];
        rx[i] = line.0.x + (line.1.x - line.0.x) * t;
        ry[i] = line.0.y + (line.1.y - line.0.y) * t;
    }
}

fn parallel_subs_xy(lines: &[Line], t: f64, rx: &mut [f64], ry: &mut [f64]) {
    lines.par_iter().zip(rx.par_iter_mut()).zip(ry.par_iter_mut()).for_each(|((line, ox), oy)| {
        *ox = line.0.x + (line.1.x - line.0.x) * t;
        *oy = line.0.y + (line.1.y - line.0.y) * t;
    });
}

fn serial_prep_subs_all_axis(lines: &Lines, t: f64, rx: &mut [f64], ry: &mut [f64], rz: &mut [f64]) {
    simd_subs(&lines.tx, t, rx);
    simd_subs(&lines.ty, t, ry);
    simd_subs(&lines.tz, t, rz);
}

fn parallel_prep_subs_all_axis(lines: &Lines, t: f64, rx: &mut [f64], ry: &mut [f64], rz: &mut [f64]) {
    rayon::scope(|s| {
        s.spawn(|_| simd_subs(&lines.tx, t, rx));
        s.spawn(|_| simd_subs(&lines.ty, t, ry));
        simd_subs(&lines.tz, t, rz);
    });
}

fn serial_prep_subs_xy(tx: &[f64], ty: &[f64], t: f64, rx: &mut [f64], ry: &mut [f64]) {
    simd_subs(tx, t, rx);
    simd_subs(ty, t, ry);
}

fn parallel_prep_subs_xy(tx: &[f64], ty: &[f64], t: f64, rx: &mut [f64], ry: &mut [f64]) {
    rayon::join(|| simd_subs(tx, t, rx), || simd_subs(ty, t, ry));
}

const BATCH_SIZE: usize = 30_000;

fn serial_stream_subs_all_axis(lines: &Lines, t: f64, rx: &mut [f64], ry: &mut [f64], rz: &mut [f64]) {
    for i in (0..lines.tx.len()).step_by(BATCH_SIZE) {
        let end = (i + BATCH_SIZE).min(lines.tx.len());
        simd_subs(&lines.tx[i..end], t, &mut rx[i..end]);
        simd_subs(&lines.ty[i..end], t, &mut ry[i..end]);
        simd_subs(&lines.tz[i..end], t, &mut rz[i..end]);
    }
}

fn serial_stream_subs_xy(lines: &Lines, t: f64, rx: &mut [f64], ry: &mut [f64]) {
    for i in (0..lines.tx.len()).step_by(BATCH_SIZE) {
        let end = (i + BATCH_SIZE).min(lines.tx.len());
        simd_subs(&lines.tx[i..end], t, &mut rx[i..end]);
        simd_subs(&lines.ty[i..end], t, &mut ry[i..end]);
    }
}

fn subs_bench(c: &mut Criterion) {
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
