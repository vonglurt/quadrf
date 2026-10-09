//! Backlog R-07, the standing short form of `make dsp-check`: the R-03 mock
//! at the T14 cadence through the de-interleave, the CS8 conversion, four
//! channelisers, the four-element power sum, the CA-CFAR detector and the
//! duty estimator for 3 s; the four tone slots are occupied at duty 1 and
//! the tones' bin powers equal the amplitude times the prototype response
//! at their offsets within 0.1 dB. The 60-s run with the CPU accounting is
//! `examples/dsp_soak.rs`; its result is recorded in `lab/LR-010`.

use std::time::Duration;

use num_complex::Complex;
use qrf_dsp::cfar::{Detector, DetectorConfig, Duty, SlotMap, SlotPlan};
use qrf_dsp::channeliser::{Channeliser, FS_HZ, M, P};
use qrf_dsp::convert::{CS8_SCALE, cs8_to_c32};
use qrf_dsp::filter::{KAISER_BETA_60DB, prototype, response_db};
use qrf_mipi::deinterleave::{buffers, deinterleave, views};
use qrf_mipi::frame::{ELEMENTS, SAMPLES_PER_FRAME};
use qrf_mipi::{MockConfig, MockDevice, Reader};

#[test]
fn mock_tile_through_the_dsp_chain_for_three_seconds() {
    let cfg = MockConfig { frames: Some(3 * 1_000_000 / 630), block_when_full: true, ..Default::default() };
    let mut dev = MockDevice::new(cfg);
    let tones = *dev.tones();
    let amplitude = dev.config().amplitude;
    dev.start();
    let mut r = Reader::new(dev).unwrap();
    let mut bufs = buffers(SAMPLES_PER_FRAME);
    let mut c32: Vec<Vec<Complex<f32>>> = (0..ELEMENTS).map(|_| vec![Complex::new(0.0, 0.0); SAMPLES_PER_FRAME]).collect();
    let mut bins = c32.clone();
    let mut chs: Vec<Channeliser> = (0..ELEMENTS).map(|_| Channeliser::new(M, P)).collect();
    let h = prototype(M, P, KAISER_BETA_60DB);
    let map = SlotMap::new(SlotPlan::US_104, 915e6, FS_HZ, M);
    let mut det = Detector::with_prototype(map.clone(), DetectorConfig { looks_per_block: ELEMENTS, ..Default::default() }, &h, M, P);
    let mut duty = Duty::new(&SlotPlan::US_104, 500);
    let tone_slots: Vec<usize> = tones.iter().map(|t| map.slot_for_offset(t.freq_hz(FS_HZ), 915e6).unwrap()).collect();
    let tone_bin: Vec<usize> = tones.iter().map(|t| chs[0].bin_for_offset(t.freq_hz(FS_HZ), FS_HZ)).collect();
    let mut tone_acc = vec![0.0f64; ELEMENTS];
    let mut tone_blocks = 0u64;
    let mut power = vec![0.0f32; M];
    let mut windows = 0usize;
    let mut frames = 0u64;
    while let Some(sp) = r.next(Duration::from_millis(500)).unwrap() {
        frames += 1;
        deinterleave(sp.data, &mut views(&mut bufs));
        for e in 0..ELEMENTS {
            cs8_to_c32(&bufs[e], CS8_SCALE, &mut c32[e]);
            chs[e].process(&c32[e], &mut bins[e]);
        }
        for b in 0..SAMPLES_PER_FRAME / M {
            power.iter_mut().for_each(|p| *p = 0.0);
            for e in 0..ELEMENTS {
                let row = &bins[e][b * M..(b + 1) * M];
                for (p, z) in power.iter_mut().zip(row) {
                    *p += z.norm_sqr();
                }
                if b >= P {
                    tone_acc[e] += row[tone_bin[e]].norm_sqr() as f64;
                }
            }
            if b >= P {
                tone_blocks += 1;
            }
            if let Some(look) = det.push_power(&power) {
                if let Some(occ) = duty.update(&look) {
                    windows += 1;
                    for &s in &tone_slots {
                        assert!(occ.slots[s].duty >= 1.0, "tone slot {s} duty {}", occ.slots[s].duty);
                    }
                }
            }
        }
    }
    assert!(frames >= 4000 && windows >= 4, "{frames} frames, {windows} windows");
    for (e, t) in tones.iter().enumerate() {
        let cycles_per_block = t.bin as f64 * M as f64 / SAMPLES_PER_FRAME as f64;
        let centre = if tone_bin[e] < M / 2 { tone_bin[e] as f64 } else { tone_bin[e] as f64 - M as f64 };
        let measured = 10.0 * (tone_acc[e] / tone_blocks as f64).log10();
        let expected = 20.0 * (amplitude / 128.0).log10() + response_db(&h, M, cycles_per_block - centre);
        assert!((measured - expected).abs() < 0.1, "element {e}: {measured} vs {expected} dBFS");
    }
}
