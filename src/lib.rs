#![feature(portable_simd)]
use rayon::prelude::*;
use std::simd::prelude::*;
use std::simd::StdFloat;

#[derive(Copy, Clone)]
pub struct Point { pub x: f64, pub y: f64, pub z: f64 }

#[derive(Copy, Clone)]
pub struct Line(pub Point, pub Point);

pub struct Lines {
    pub tx: Vec<f64>, pub ty: Vec<f64>, pub tz: Vec<f64>,
}

#[inline(always)]
pub fn simd_subs(target: &[f64], t: f64, out: &mut [f64]) {
    let len = target.len();
    assert_eq!(len, out.len());

    let t_v = f64x4::splat(t);
    let mut i = 0;

    while i + 3 < len {
        let p0_v = f64x4::from_slice(&out[i..i+4]);
        let p1_v = f64x4::from_slice(&target[i..i+4]);

        let res_v = (p1_v - p0_v).mul_add(t_v, p0_v);

        res_v.copy_to_slice(&mut out[i..i+4]);

        i += 4;
    }

    while i < len {
        let p0 = out[i];
        out[i] = p0 + (target[i] - p0) * t;
        i += 1;
    }
}

pub fn serial_subs_all_axis(lines: &[Line], t: f64, out: &mut [Point]) {
    for i in 0..lines.len() {
        let line = &lines[i];
        out[i].x = line.0.x + (line.1.x - line.0.x) * t;
        out[i].y = line.0.y + (line.1.y - line.0.y) * t;
        out[i].z = line.0.z + (line.1.z - line.0.z) * t;
    }
}

pub fn parallel_subs_all_axis(lines: &[Line], t: f64, out: &mut [Point]) {
    lines.par_iter().zip(out.par_iter_mut()).for_each(|(line, p)| {
        p.x = line.0.x + (line.1.x - line.0.x) * t;
        p.y = line.0.y + (line.1.y - line.0.y) * t;
        p.z = line.0.z + (line.1.z - line.0.z) * t;
    });
}

pub fn serial_subs_xy(lines: &[Line], t: f64, rx: &mut [f64], ry: &mut [f64]) {
    for i in 0..lines.len() {
        let line = &lines[i];
        rx[i] = line.0.x + (line.1.x - line.0.x) * t;
        ry[i] = line.0.y + (line.1.y - line.0.y) * t;
    }
}

pub fn parallel_subs_xy(lines: &[Line], t: f64, rx: &mut [f64], ry: &mut [f64]) {
    lines.par_iter().zip(rx.par_iter_mut()).zip(ry.par_iter_mut()).for_each(|((line, ox), oy)| {
        *ox = line.0.x + (line.1.x - line.0.x) * t;
        *oy = line.0.y + (line.1.y - line.0.y) * t;
    });
}

pub fn serial_prep_subs_all_axis(lines: &Lines, t: f64, rx: &mut [f64], ry: &mut [f64], rz: &mut [f64]) {
    simd_subs(&lines.tx, t, rx);
    simd_subs(&lines.ty, t, ry);
    simd_subs(&lines.tz, t, rz);
}

pub fn parallel_prep_subs_all_axis(lines: &Lines, t: f64, rx: &mut [f64], ry: &mut [f64], rz: &mut [f64]) {
    rayon::scope(|s| {
        s.spawn(|_| simd_subs(&lines.tx, t, rx));
        s.spawn(|_| simd_subs(&lines.ty, t, ry));
        simd_subs(&lines.tz, t, rz);
    });
}

pub fn serial_prep_subs_xy(tx: &[f64], ty: &[f64], t: f64, rx: &mut [f64], ry: &mut [f64]) {
    simd_subs(tx, t, rx);
    simd_subs(ty, t, ry);
}

pub fn parallel_prep_subs_xy(tx: &[f64], ty: &[f64], t: f64, rx: &mut [f64], ry: &mut [f64]) {
    rayon::join(|| simd_subs(tx, t, rx), || simd_subs(ty, t, ry));
}

pub const BATCH_SIZE: usize = 30_000;

pub fn serial_stream_subs_all_axis(lines: &Lines, t: f64, rx: &mut [f64], ry: &mut [f64], rz: &mut [f64]) {
    for i in (0..lines.tx.len()).step_by(BATCH_SIZE) {
        let end = (i + BATCH_SIZE).min(lines.tx.len());
        simd_subs(&lines.tx[i..end], t, &mut rx[i..end]);
        simd_subs(&lines.ty[i..end], t, &mut ry[i..end]);
        simd_subs(&lines.tz[i..end], t, &mut rz[i..end]);
    }
}

pub fn serial_stream_subs_xy(lines: &Lines, t: f64, rx: &mut [f64], ry: &mut [f64]) {
    for i in (0..lines.tx.len()).step_by(BATCH_SIZE) {
        let end = (i + BATCH_SIZE).min(lines.tx.len());
        simd_subs(&lines.tx[i..end], t, &mut rx[i..end]);
        simd_subs(&lines.ty[i..end], t, &mut ry[i..end]);
    }
}
