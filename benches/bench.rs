#![feature(portable_simd)]
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rayon::prelude::*;
use std::simd::prelude::*;


#[derive(Copy, Clone)]
struct Point { x: f64, y: f64, z: f64 }

#[derive(Copy, Clone)]
struct Line(Point, Point);

struct Lines {
    ox: Vec<f64>, oy: Vec<f64>, oz: Vec<f64>,
    dx: Vec<f64>, dy: Vec<f64>, dz: Vec<f64>,
}


#[inline(always)]
fn simd_subs(origin: &[f64], direction: &[f64], t: f64, out: &mut [f64]) {
    let t_v = f64x4::splat(t);
    let (o_prefix, o_simd, o_suffix) = origin.as_simd::<4>();
    let (d_prefix, d_simd, d_suffix) = direction.as_simd::<4>();
    let (out_prefix, out_simd, out_suffix) = out.as_simd_mut::<4>();

    for i in 0..o_prefix.len() {
        out_prefix[i] = o_prefix[i] + d_prefix[i] * t;
    }
    for (i, (o_v, d_v)) in o_simd.iter().zip(d_simd.iter()).enumerate() {
        out_simd[i] = *o_v + *d_v * t_v;
    }
    for i in 0..o_suffix.len() {
        out_suffix[i] = o_suffix[i] + d_suffix[i] * t;
    }
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
    out.par_iter_mut().enumerate().for_each(|(i, p)| {
        let line = &lines[i];
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
    rx.par_iter_mut().zip(ry.par_iter_mut()).enumerate().for_each(|(i, (out_x, out_y))| {
        let line = &lines[i];
        *out_x = line.0.x + (line.1.x - line.0.x) * t;
        *out_y = line.0.y + (line.1.y - line.0.y) * t;
    });
}

fn serial_prep_subs_all_axis(lines: &Lines, t: f64, rx: &mut [f64], ry: &mut [f64], rz: &mut [f64]) {
    simd_subs(&lines.ox, &lines.dx, t, rx);
    simd_subs(&lines.oy, &lines.dy, t, ry);
    simd_subs(&lines.oz, &lines.dz, t, rz);
}

fn parallel_prep_subs_all_axis(lines: &Lines, t: f64, rx: &mut [f64], ry: &mut [f64], rz: &mut [f64]) {
    rayon::scope(|s| {
        s.spawn(|_| simd_subs(&lines.ox, &lines.dx, t, rx));
        s.spawn(|_| simd_subs(&lines.oy, &lines.dy, t, ry));
        simd_subs(&lines.oz, &lines.dz, t, rz);
    });
}

fn serial_prep_subs_xy(ox: &[f64], dx: &[f64], oy: &[f64], dy: &[f64], t: f64, rx: &mut [f64], ry: &mut [f64]) {
    simd_subs(ox, dx, t, rx);
    simd_subs(oy, dy, t, ry);
}

fn parallel_prep_subs_xy(ox: &[f64], dx: &[f64], oy: &[f64], dy: &[f64], t: f64, rx: &mut [f64], ry: &mut [f64]) {
    rayon::join(
        || simd_subs(ox, dx, t, rx),
                || simd_subs(oy, dy, t, ry)
    );
}


fn subs_bench(c: &mut Criterion) {
    let size = 20_000_000;
    let t = 0.42;

    let mut aos_lines = Vec::with_capacity(size);
    let mut soa_lines = Lines {
        ox: Vec::with_capacity(size), oy: Vec::with_capacity(size), oz: Vec::with_capacity(size),
        dx: Vec::with_capacity(size), dy: Vec::with_capacity(size), dz: Vec::with_capacity(size),
    };

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
        soa_lines.ox.push(p0.x); soa_lines.oy.push(p0.y); soa_lines.oz.push(p0.z);
        soa_lines.dx.push(p1.x - p0.x); soa_lines.dy.push(p1.y - p0.y); soa_lines.dz.push(p1.z - p0.z);
    }

    let mut aos_out = vec![Point { x: 0.0, y: 0.0, z: 0.0 }; size];
    let mut rx = vec![0.0; size];
    let mut ry = vec![0.0; size];
    let mut rz = vec![0.0; size];

    let mut group = c.benchmark_group("subs_bench_20M");
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
    serial_prep_subs_xy(black_box(&soa_lines.ox), black_box(&soa_lines.dx), black_box(&soa_lines.oy), black_box(&soa_lines.dy), black_box(t), black_box(&mut rx), black_box(&mut ry))
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
    parallel_prep_subs_xy(black_box(&soa_lines.ox), black_box(&soa_lines.dx), black_box(&soa_lines.oy), black_box(&soa_lines.dy), black_box(t), black_box(&mut rx), black_box(&mut ry))
    ));

    group.finish();
}

criterion_group!(benches, subs_bench);
criterion_main!(benches);
