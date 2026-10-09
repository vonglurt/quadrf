//! Backlog R-02 check (SPEC-009 S-009-4/5/9/10): a simulated plug-in publishes
//! all eight types (a Header inside each of the seven messages); a subscriber
//! decodes them; rate and size limits are enforced. Runs over an ipc endpoint
//! in the temp directory on the VM.
use std::collections::BTreeSet;
use std::time::{Duration, Instant};

use qrf_bus::{
    ALL_TOPICS, AnyFeed, Bearing, BusError, Calibration, ClockQuality, Complex, Header,
    Health, Kind, Limits, LoraFrame, Occupancy, Point, PowerRef, Publish, Publisher, Scatter,
    Slot, Spectrum, Subscriber, Vec3, topic_prefix,
};

fn endpoint(tag: &str) -> String {
    format!("ipc://{}/qrf-r02-{}-{tag}.pub", std::env::temp_dir().display(), std::process::id())
}

fn cleanup(endpoint: &str) {
    let _ = std::fs::remove_file(endpoint.trim_start_matches("ipc://"));
}

/// ZeroMQ's slow-joiner problem: a subscription takes a moment to reach the
/// publisher. Publish per-event probes until one comes back, then drain.
async fn join(p: &mut Publisher, s: &mut Subscriber) {
    let mut joined = false;
    for _ in 0..200 {
        p.publish(Calibration::default()).await.unwrap();
        if let Ok(Ok(env)) = tokio::time::timeout(Duration::from_millis(25), s.recv()).await {
            assert_eq!(env.kind, Kind::Calibration);
            joined = true;
            break;
        }
    }
    assert!(joined, "subscriber never received the join probe");
    while let Ok(Ok(_)) = tokio::time::timeout(Duration::from_millis(50), s.recv()).await {}
}

#[tokio::test]
async fn all_eight_types_round_trip() {
    let ep = endpoint("types");
    let mut p = Publisher::bind("sim0", &[&ep], Limits::default()).await.unwrap();
    p.set_clock_quality(ClockQuality::Ntp);
    let mut s = Subscriber::connect(&[&ep]).await.unwrap();
    s.subscribe(ALL_TOPICS).await.unwrap();
    join(&mut p, &mut s).await;
    let seq0 = p.seq();

    let spectrum = Spectrum { f_start_hz: 902e6, f_step_hz: 203_125.0, n_avg: 8, bins_dbm: (0..128).map(|i| -100.0 + i as f32 * 0.1).collect(), ..Default::default() };
    let occupancy = Occupancy { plan: "us-915-104".into(), slots: (0..104).map(|i| Slot { power_dbm: -110.0 + i as f32, duty: 0.01 * i as f32, az_deg: 3.0 * i as f32, az_sigma_deg: 2.5 }).collect(), ..Default::default() };
    let bearing = Bearing { f_hz: 906.875e6, az_deg: 123.4, el_deg: 2.0, sigma_deg: 0.8, snr_db: 17.5, burst_id: 42, ..Default::default() };
    let frame = LoraFrame { f_hz: 906.875e6, sf: 11, bw_hz: 250_000, cr: 5, snr_db: -3.5, rssi_dbm: -117.0, cfo_hz: 812.5, bearing: Some(bearing.clone()), crc_ok: true, payload: vec![0xff, 0xff, 0x00, 0x01, 0x2b], t_start_ns: 1_700_000_000_000_000_000, ..Default::default() };
    let scatter = Scatter { points: (0..1000).map(|i| Point { az_deg: (i % 360) as f32, el_deg: 10.0, power_dbm: -70.0, f_hz: 5.8e9 }).collect(), ..Default::default() };
    let calibration = Calibration { element_pos_m: vec![Vec3 { x: 0.0, y: 0.0, z: 0.0 }, Vec3 { x: 0.164, y: 0.0, z: 0.0 }, Vec3 { x: 0.0, y: 0.164, z: 0.0 }, Vec3 { x: 0.164, y: 0.164, z: 0.0 }], element_gain: vec![Complex { re: 1.0, im: 0.0 }; 4], f_hz: 915e6, ..Default::default() };
    // A sensor that stamps its own capture time keeps it (the publisher completes, never overwrites).
    let health = Health { header: Some(Header { t_tai_ns: 1_234, t_mono_ns: 5_678, ..Default::default() }), cpu_pct: 12.5, temp_c: 51.0, loss_events: 0, msgs_per_s: 47.0, device_present: true, throttled: 0 };

    assert_eq!(p.publish(spectrum.clone()).await.unwrap(), Publish::Sent);
    assert_eq!(p.publish(occupancy.clone()).await.unwrap(), Publish::Sent);
    assert_eq!(p.publish(bearing.clone()).await.unwrap(), Publish::Sent);
    assert_eq!(p.publish(frame.clone()).await.unwrap(), Publish::Sent);
    assert_eq!(p.publish(scatter.clone()).await.unwrap(), Publish::Sent);
    assert_eq!(p.publish(calibration.clone()).await.unwrap(), Publish::Sent);
    assert_eq!(p.publish(health.clone()).await.unwrap(), Publish::Sent);

    let mut got = Vec::new();
    while got.len() < 7 {
        let env = tokio::time::timeout(Duration::from_secs(5), s.recv()).await.expect("message within 5 s").unwrap();
        assert_eq!(env.sensor_id, "sim0");
        got.push(env.decode_any().unwrap());
    }
    assert_eq!(got.iter().map(AnyFeed::kind).collect::<BTreeSet<_>>().len(), 7, "all seven types arrived");
    assert_eq!(got.iter().map(|m| m.kind()).collect::<Vec<_>>(), Kind::ALL.to_vec(), "in publication order");

    // Every message carries a Header (the eighth type) completed by the publisher (S-009-4).
    let mut prev_seq = seq0;
    for m in &got {
        let h = m.header().expect("header present");
        assert_eq!(h.sensor_id, "sim0");
        assert_eq!(h.seq, prev_seq + 1, "seq increases by one per message");
        prev_seq = h.seq;
        assert!(h.t_tai_ns > 0 && h.t_mono_ns > 0);
        assert_eq!(h.clock_quality(), ClockQuality::Ntp);
        assert_eq!(h.power_ref(), PowerRef::Dbfs);
    }

    // Payloads round-trip field for field.
    let strip = |h: &mut Option<Header>| *h = None;
    for m in got {
        match m {
            AnyFeed::Spectrum(mut x) => { strip(&mut x.header); assert_eq!(x, spectrum) }
            AnyFeed::Occupancy(mut x) => { strip(&mut x.header); assert_eq!(x, occupancy) }
            AnyFeed::Bearing(mut x) => { strip(&mut x.header); assert_eq!(x, bearing) }
            AnyFeed::LoraFrame(mut x) => { strip(&mut x.header); assert_eq!(x, frame) }
            AnyFeed::Scatter(mut x) => { strip(&mut x.header); assert_eq!(x, scatter) }
            AnyFeed::Calibration(mut x) => { strip(&mut x.header); assert_eq!(x, calibration) }
            AnyFeed::Health(x) => {
                let h = x.header.as_ref().unwrap();
                assert_eq!((h.t_tai_ns, h.t_mono_ns), (1_234, 5_678), "the sensor's own stamps are kept");
                assert_eq!(x.throttled, 0);
                assert_eq!((x.cpu_pct, x.temp_c, x.msgs_per_s, x.device_present), (12.5, 51.0, 47.0, true));
            }
        }
    }
    cleanup(&ep);
}

#[tokio::test]
async fn prefix_subscription_selects_one_type() {
    let ep = endpoint("prefix");
    let mut p = Publisher::bind("sim1", &[&ep], Limits::default()).await.unwrap();
    let mut s = Subscriber::connect(&[&ep]).await.unwrap();
    s.subscribe(&topic_prefix(Kind::Calibration)).await.unwrap();
    join(&mut p, &mut s).await;
    s.subscribe(&topic_prefix(Kind::Bearing)).await.unwrap();
    tokio::time::sleep(Duration::from_millis(100)).await;
    for i in 0..5 {
        p.publish(Scatter { points: vec![Point::default(); 10], ..Default::default() }).await.unwrap();
        p.publish(LoraFrame { sf: 7, ..Default::default() }).await.unwrap();
        p.publish(Bearing { burst_id: i, ..Default::default() }).await.unwrap();
    }
    for i in 0..5 {
        let env = tokio::time::timeout(Duration::from_secs(5), s.recv()).await.unwrap().unwrap();
        assert_eq!(env.kind, Kind::Bearing);
        assert_eq!(env.decode::<Bearing>().unwrap().burst_id, i);
        assert!(matches!(env.decode::<Scatter>(), Err(BusError::WrongKind { .. })));
    }
    assert!(tokio::time::timeout(Duration::from_millis(200), s.recv()).await.is_err(), "nothing but bearings arrives");
    cleanup(&ep);
}

#[tokio::test]
async fn rate_and_size_limits_are_enforced() {
    let ep = endpoint("limits");
    let mut p = Publisher::bind("sim2", &[&ep], Limits::default()).await.unwrap();
    let mut s = Subscriber::connect(&[&ep]).await.unwrap();
    s.subscribe(ALL_TOPICS).await.unwrap();
    join(&mut p, &mut s).await;

    // Size: 4 097 bins is refused, 4 096 pass (S-009-10).
    let too_big = Spectrum { bins_dbm: vec![0.0; 4097], ..Default::default() };
    assert!(matches!(p.publish(too_big).await, Err(BusError::TooLarge { kind: Kind::Spectrum, len: 4097, max: 4096 })));
    assert_eq!(p.publish(Spectrum { bins_dbm: vec![0.0; 4096], ..Default::default() }).await.unwrap(), Publish::Sent);
    assert!(matches!(p.publish(Scatter { points: vec![Point::default(); 4097], ..Default::default() }).await, Err(BusError::TooLarge { kind: Kind::Scatter, .. })));

    // Rate: 100 Occupancy messages in well under a second at 10 Hz declared.
    let t0 = Instant::now();
    let mut sent = 0;
    for _ in 0..100 {
        if p.publish(Occupancy::default()).await.unwrap() == Publish::Sent {
            sent += 1;
        }
    }
    let elapsed = t0.elapsed();
    let allowed = 1 + (elapsed.as_secs_f64() * 10.0).ceil() as usize;
    assert!(sent <= allowed, "{sent} occupancy messages sent in {elapsed:?}; at most {allowed} allowed");
    assert!(sent >= 1);
    let throttled_occupancy = p.limiter().throttled(Kind::Occupancy);
    assert_eq!(sent + throttled_occupancy as usize, 100);

    // Health is limited to 1 Hz and reports the throttle count (S-009-10).
    assert_eq!(p.publish(Health::default()).await.unwrap(), Publish::Sent);
    assert_eq!(p.publish(Health::default()).await.unwrap(), Publish::Throttled);
    let total = p.limiter().throttled_total();
    assert_eq!(total, throttled_occupancy + 1);

    // The subscriber sees exactly what was sent: 1 Spectrum, `sent` Occupancy, 1 Health, and the Health carries the count at its send time.
    let mut kinds = Vec::new();
    let mut health_throttled = None;
    loop {
        match tokio::time::timeout(Duration::from_millis(500), s.recv()).await {
            Ok(Ok(env)) => {
                if env.kind == Kind::Health {
                    health_throttled = Some(env.decode::<Health>().unwrap().throttled);
                }
                kinds.push(env.kind);
            }
            _ => break,
        }
    }
    assert_eq!(kinds.iter().filter(|k| **k == Kind::Spectrum).count(), 1);
    assert_eq!(kinds.iter().filter(|k| **k == Kind::Occupancy).count(), sent);
    assert_eq!(kinds.iter().filter(|k| **k == Kind::Health).count(), 1);
    assert_eq!(health_throttled, Some(throttled_occupancy), "Health.throttled is the count when it was sent");
    cleanup(&ep);
}
