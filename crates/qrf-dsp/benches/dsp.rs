//! Criterion benchmarks for backlog R-07: the channeliser per block and per
//! T14 frame (four elements), the CS8 conversion, the two bearing estimators.
//! `make dsp-bench` runs them; the VM figures are reference only (LR-007 §IV).

use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use num_complex::Complex;
use qrf_dsp::channeliser::{FS_HZ, M, P};
use qrf_dsp::convert::{CS8_SCALE, cs8_to_c32};
use qrf_dsp::rng::Rng;
use qrf_dsp::{Array, Channeliser, Kernel, Music, phase_difference};
use std::hint::black_box;

const FRAME_SAMPLES: usize = 16_384;

fn channeliser(c: &mut Criterion) {
    let mut r = Rng::new(1);
    let x = r.complex_gaussian_f32(FRAME_SAMPLES, 0.1);
    let mut out = vec![Complex::new(0.0f32, 0.0); FRAME_SAMPLES];
    let mut g = c.benchmark_group("channeliser");
    g.throughput(Throughput::Elements(M as u64));
    for k in [Kernel::Scalar, Kernel::fastest()] {
        let mut ch = Channeliser::new(M, P);
        ch.set_kernel(k);
        g.bench_function(format!("block_{}", k.name()), |b| b.iter(|| ch.push_block(black_box(&x[..M]), &mut out[..M])));
    }
    g.finish();
    let mut g = c.benchmark_group("channeliser_frame");
    g.throughput(Throughput::Elements(4 * FRAME_SAMPLES as u64));
    let mut chs: Vec<Channeliser> = (0..4).map(|_| Channeliser::new(M, P)).collect();
    g.bench_function("four_elements_16384", |b| {
        b.iter(|| {
            for ch in chs.iter_mut() {
                ch.process(black_box(&x), &mut out);
            }
        })
    });
    g.finish();
}

fn convert(c: &mut Criterion) {
    let mut r = Rng::new(2);
    let src: Vec<u8> = (0..2 * FRAME_SAMPLES).map(|_| r.next_u64() as u8).collect();
    let mut out = vec![Complex::new(0.0f32, 0.0); FRAME_SAMPLES];
    let mut g = c.benchmark_group("convert");
    g.throughput(Throughput::Elements(FRAME_SAMPLES as u64));
    g.bench_function("cs8_to_c32_16384", |b| b.iter(|| cs8_to_c32(black_box(&src), CS8_SCALE, &mut out)));
    g.finish();
}

fn bearing(c: &mut Criterion) {
    let a = Array::half_wave(915e6);
    let mut r = Rng::new(3);
    let x = a.plane_wave(915e6, 12.0, 5.0, 1024, 20.0, &mut r);
    let v = [&x[0][..], &x[1][..], &x[2][..], &x[3][..]];
    let m = Music::new(a, 915e6, 1);
    let mut g = c.benchmark_group("bearing_1024");
    g.bench_function("phase_difference", |b| b.iter(|| phase_difference(&a, 915e6, black_box(v))));
    g.bench_function("music_1_source", |b| b.iter(|| m.estimate(black_box(v))));
    g.finish();
    let _ = FS_HZ;
}

criterion_group!(benches, channeliser, convert, bearing);
criterion_main!(benches);
