//! Prints one line per message from a qrf.v1 publisher (backlog R-02; a bench tool for F-03, R-09).
//!
//!     cargo run -p qrf-bus --example bus_dump -- [endpoint] [topic-prefix]
use qrf_bus::{ALL_TOPICS, AnyFeed, Subscriber};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let endpoint = args.next().unwrap_or_else(|| "ipc:///tmp/qrf-sim0.pub".to_owned());
    let prefix = args.next().unwrap_or_else(|| ALL_TOPICS.to_owned());
    let mut s = Subscriber::connect(&[&endpoint]).await?;
    s.subscribe(&prefix).await?;
    eprintln!("listening on {endpoint} for {prefix:?}");
    loop {
        let env = s.recv().await?;
        let m = env.decode_any()?;
        let h = m.header().cloned().unwrap_or_default();
        let detail = match &m {
            AnyFeed::Spectrum(x) => format!("{} bins from {:.3} MHz", x.bins_dbm.len(), x.f_start_hz / 1e6),
            AnyFeed::Occupancy(x) => format!("plan {} {} slots, busiest {:.2}", x.plan, x.slots.len(), x.slots.iter().map(|s| s.duty).fold(0.0, f32::max)),
            AnyFeed::Bearing(x) => format!("az {:.1} el {:.1} sigma {:.1} snr {:.1} dB", x.az_deg, x.el_deg, x.sigma_deg, x.snr_db),
            AnyFeed::LoraFrame(x) => format!("SF{} BW {} kHz snr {:.1} crc {} {} B", x.sf, x.bw_hz / 1000, x.snr_db, x.crc_ok, x.payload.len()),
            AnyFeed::Scatter(x) => format!("{} points", x.points.len()),
            AnyFeed::Calibration(x) => format!("{} elements at {:.3} MHz", x.element_pos_m.len(), x.f_hz / 1e6),
            AnyFeed::Health(x) => format!("cpu {:.1} % temp {:.1} C loss {} throttled {} present {}", x.cpu_pct, x.temp_c, x.loss_events, x.throttled, x.device_present),
        };
        println!("{:<11} {:<8} seq {:>6} tai {} {:?} {}", m.kind(), env.sensor_id, h.seq, h.t_tai_ns, h.clock_quality(), detail);
    }
}
