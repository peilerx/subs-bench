use subs_bench::*;

#[cfg(test)]
mod tests {
    use super::*;

    const EPS: f64 = 1e-15;

    fn setup_test_data(size: usize) -> (Vec<Line>, Lines, Vec<f64>, Vec<f64>, Vec<f64>) {
        let mut aos_lines = Vec::with_capacity(size);
        let mut soa_lines = Lines {
            tx: Vec::with_capacity(size),
            ty: Vec::with_capacity(size),
            tz: Vec::with_capacity(size),
        };
        let mut rx = vec![0.0; size];
        let mut ry = vec![0.0; size];
        let mut rz = vec![0.0; size];

        for i in 0..size {
            let p0 = Point { x: i as f64, y: (i * 2) as f64, z: (i * 3) as f64 };
            let p1 = Point { x: (i + 10) as f64, y: (i * 2 + 10) as f64, z: (i * 3 + 10) as f64 };

            aos_lines.push(Line(p0, p1));
            soa_lines.tx.push(p1.x);
            soa_lines.ty.push(p1.y);
            soa_lines.tz.push(p1.z);

            rx[i] = p0.x;
            ry[i] = p0.y;
            rz[i] = p0.z;
        }
        (aos_lines, soa_lines, rx, ry, rz)
    }

    #[test]
    fn test_math_consistency_all_sizes() {
        let sizes = [1, 2, 3, 4, 5, 7, 8, 9, 15, 16, 31, 33, 127];
        let t = 0.42;

        for &size in &sizes {
            let (aos, soa, mut rx, mut ry, mut rz) = setup_test_data(size);
            let mut aos_out = vec![Point { x: 0.0, y: 0.0, z: 0.0 }; size];

            serial_subs_all_axis(&aos, t, &mut aos_out);

            serial_prep_subs_all_axis(&soa, t, &mut rx, &mut ry, &mut rz);

            for i in 0..size {
                assert!((aos_out[i].x - rx[i]).abs() < EPS, "X fail at size {}, idx {}", size, i);
                assert!((aos_out[i].y - ry[i]).abs() < EPS, "Y fail at size {}, idx {}", size, i);
                assert!((aos_out[i].z - rz[i]).abs() < EPS, "Z fail at size {}, idx {}", size, i);
            }
        }
    }

    #[test]
    fn test_fma_precision_drift() {
        let (aos, soa, mut rx, mut ry, mut rz) = setup_test_data(100);
        let t = 0.3333333333333333;
        let mut aos_out = vec![Point { x: 0.0, y: 0.0, z: 0.0 }; 100];

        serial_subs_all_axis(&aos, t, &mut aos_out);
        serial_prep_subs_all_axis(&soa, t, &mut rx, &mut ry, &mut rz);

        for i in 0..100 {
            let diff = (aos_out[i].x - rx[i]).abs();
            assert!(diff < 2e-15, "FMA drift too high at idx {}: {}", i, diff);
        }
    }

    #[test]
    fn test_parallel_correctness() {
        let size = 1000;
        let t = 0.5;
        let (_, soa, mut rx_ser, mut ry_ser, mut rz_ser) = setup_test_data(size);

        let (mut rx_par, mut ry_par, mut rz_par) = (rx_ser.clone(), ry_ser.clone(), rz_ser.clone());

        serial_prep_subs_all_axis(&soa, t, &mut rx_ser, &mut ry_ser, &mut rz_ser);
        parallel_prep_subs_all_axis(&soa, t, &mut rx_par, &mut ry_par, &mut rz_par);

        for i in 0..size {
            assert_eq!(rx_ser[i], rx_par[i], "Parallel X mismatch at idx {}", i);
            assert_eq!(ry_ser[i], ry_par[i], "Parallel Y mismatch at idx {}", i);
            assert_eq!(rz_ser[i], rz_par[i], "Parallel Z mismatch at idx {}", i);
        }
    }

    #[test]
    fn test_batch_boundary_consistency() {
        let size = 60_005;
        let t = 0.1;
        let (_, soa, mut rx_norm, mut ry_norm, mut rz_norm) = setup_test_data(size);

        let mut rx_stream = rx_norm.clone();
        let mut ry_stream = ry_norm.clone();
        let mut rz_stream = rz_norm.clone();

        serial_prep_subs_all_axis(&soa, t, &mut rx_norm, &mut ry_norm, &mut rz_norm);
        serial_stream_subs_all_axis(&soa, t, &mut rx_stream, &mut ry_stream, &mut rz_stream);

        for i in 0..size {
            assert_eq!(rx_norm[i], rx_stream[i], "Stream boundary error at idx {}", i);
        }
    }
}
