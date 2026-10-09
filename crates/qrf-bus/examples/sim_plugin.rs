//! A simulated sensor plug-in (backlog R-02; feeds R-09 and R-P1): publishes
//! every qrf.v1 type at its declared S-009-10 rate on an ipc endpoint.
//!
//!     cargo run -p qrf-bus --example sim_plugin -- [endpoint] [seconds]
//!
//! Defaults: `ipc:///tmp/qrf-sim0.pub`, run forever.
use std::time::{Duration, Instant};

use qrf_bus::{Bearing, Calibration, Complex, Health, Limits, LoraFrame, Occupancy, Point, Publisher, Scatter, Slot, Spectrum, Vec3};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let endpoint = args.next().unwrap_or_else(|| "ipc:///tmp/qrf-sim0.pub".to_owned());
    let seconds: Option<f64> = args.next().map(|s| s.parse()).transpose()?;
    let mut p = Publisher::bind("sim0", &[&endpoint], Limits::default()).await?;
    eprintln!("sim0 publishing on {endpoint}");
    p.publish(Calibration {
        element_pos_m: vec![Vec3 { x: 0.0, y: 0.0, z: 0.0 }, Vec3 { x: 0.164, y: 0.0, z: 0.0 }, Vec3 { x: 0.0, y: 0.164, z: 0.0 }, Vec3 { x: 0.164, y: 0.164, z: 0.0 }],
        element_gain: vec![Complex { re: 1.0, im: 0.0 }; 4],
        f_hz: 915e6,
        ..Default::default()
    })
    .await?;
    let start = Instant::now();
    let mut tick = tokio::time::interval(Duration::from_millis(33));
    let mut n: u64 = 0;
    loop {
        tick.tick().await;
        if seconds.is_some_and(|s| start.elapsed().as_secs_f64() >= s) {
            break;
        }
        n += 1;
        let t = start.elapsed().as_secs_f64();
        // Scatter at every tick (30 Hz), the others when their period has elapsed; the limiter drops the surplus.
        p.publish(Scatter { points: (0..512).map(|i| Point { az_deg: (i as f32 * 0.7 + t as f32 * 10.0) % 360.0, el_deg: 5.0, power_dbm: -80.0 + (i % 7) as f32, f_hz: 5.8e9 }).collect(), ..Default::default() }).await?;
        p.publish(Spectrum { f_start_hz: 902e6, f_step_hz: 203_125.0, n_avg: 8, bins_dbm: (0..128).map(|i| -105.0 + 20.0 * (((i as f64 - 64.0 + 30.0 * t.sin()) / 4.0).powi(2) * -0.5).exp() as f32).collect(), ..Default::default() }).await?;
        p.publish(Occupancy { plan: "us-915-104".into(), slots: (0..104).map(|i| Slot { power_dbm: if i == 20 { -75.0 } else { -118.0 }, duty: if i == 20 { 0.3 } else { 0.0 }, az_deg: 123.4, az_sigma_deg: 2.0 }).collect(), ..Default::default() }).await?;
        if n % 15 == 0 {
            p.publish(Bearing { f_hz: 906.875e6, az_deg: 123.4 + (t.sin() as f32), el_deg: 1.0, sigma_deg: 1.5, snr_db: 15.0, burst_id: n, ..Default::default() }).await?;
        }
        if n % 30 == 0 {
            p.publish(LoraFrame { f_hz: 906.875e6, sf: 11, bw_hz: 250_000, cr: 5, snr_db: -4.0, rssi_dbm: -115.0, cfo_hz: 500.0, crc_ok: true, payload: n.to_be_bytes().to_vec(), ..Default::default() }).await?;
            p.publish(Health { cpu_pct: 3.0, temp_c: 45.0, loss_events: 0, msgs_per_s: 46.0, device_present: true, ..Default::default() }).await?;
        }
    }
    eprintln!("sent {} messages, throttled {}", p.seq(), p.limiter().throttled_total());
    Ok(())
}
