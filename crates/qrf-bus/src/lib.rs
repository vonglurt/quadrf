//! qrf.v1 feed-bus schema and ZeroMQ transport
//!
//! Implements SPEC-009 S-009-4 and S-009-5 (the schema, compiled at build time
//! from `proto/qrf/v1/qrf.proto` into [`v1`]), S-009-9 (endpoints and topic
//! strings), S-009-10 (rate and size limits) and SPEC-008 S-008-7 (ZeroMQ
//! PUB/SUB carrying protobuf). Raw I/Q never travels here (S-009-9: that is
//! the shared-memory ring of `qrf-tiled`). Backlog R-02; the check is
//! `tests/r02_check.rs`.
#![forbid(unsafe_code)]

use std::fmt;
use std::path::Path;
use std::time::{Duration, Instant};

use bytes::Bytes;
use prost::Message;
use qrf_core::time::Stamp;
use zeromq::{PubSocket, Socket, SocketRecv, SocketSend, SubSocket, ZmqMessage};

/// The crate's name, for the `Health` message and logs (SPEC-009 S-009-5).
pub const NAME: &str = "qrf-bus";

/// The protobuf package (S-009-5). It is append-only; a breaking change is `qrf.v2`.
pub const PACKAGE: &str = "qrf.v1";

/// Generated from `proto/qrf/v1/qrf.proto` (S-009-4, S-009-5).
pub mod v1 {
    include!(concat!(env!("OUT_DIR"), "/qrf.v1.rs"));
}
pub use v1::{
    Bearing, Calibration, ClockQuality, Complex, Header, Health, LoraFrame, Occupancy, Point,
    PowerRef, Scatter, Slot, Spectrum, Vec3,
};

/// The seven message types of S-009-5, in the order the statement lists them.
/// Every one of them carries a [`Header`] as its first field (S-009-4).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Kind {
    Spectrum,
    Occupancy,
    Bearing,
    LoraFrame,
    Scatter,
    Calibration,
    Health,
}

impl Kind {
    pub const ALL: [Kind; 7] = [
        Kind::Spectrum,
        Kind::Occupancy,
        Kind::Bearing,
        Kind::LoraFrame,
        Kind::Scatter,
        Kind::Calibration,
        Kind::Health,
    ];

    /// The type's name as it appears in the topic string (S-009-9).
    pub fn name(self) -> &'static str {
        match self {
            Kind::Spectrum => "Spectrum",
            Kind::Occupancy => "Occupancy",
            Kind::Bearing => "Bearing",
            Kind::LoraFrame => "LoraFrame",
            Kind::Scatter => "Scatter",
            Kind::Calibration => "Calibration",
            Kind::Health => "Health",
        }
    }

    pub fn from_name(name: &str) -> Option<Kind> {
        Kind::ALL.into_iter().find(|k| k.name() == name)
    }
}

impl fmt::Display for Kind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

/// Subscription prefix that selects everything on the bus (S-009-9).
pub const ALL_TOPICS: &str = "qrf.v1.";

/// Topic string `qrf.v1.<Type>/<sensor_id>` (S-009-9).
pub fn topic(kind: Kind, sensor_id: &str) -> String {
    format!("{PACKAGE}.{}/{sensor_id}", kind.name())
}

/// Subscription prefix that selects one type from every sensor (S-009-9).
pub fn topic_prefix(kind: Kind) -> String {
    format!("{PACKAGE}.{}/", kind.name())
}

/// Splits a topic string into its type and sensor id (S-009-9).
pub fn parse_topic(topic: &str) -> Option<(Kind, &str)> {
    let rest = topic.strip_prefix(ALL_TOPICS)?;
    let (name, sensor_id) = rest.split_once('/')?;
    Some((Kind::from_name(name)?, sensor_id))
}

/// Where a sensor's publisher binds by default (S-009-9): `ipc:///run/qrf/<sensor_id>.pub`.
pub const DEFAULT_RUN_DIR: &str = "/run/qrf";

/// The ipc endpoint of one sensor under a run directory (S-009-9).
pub fn ipc_endpoint(run_dir: &Path, sensor_id: &str) -> String {
    format!("ipc://{}/{sensor_id}.pub", run_dir.display())
}

/// A message type that travels on the bus: it carries a [`Header`] and knows its [`Kind`].
pub trait Feed: Message + Default + Sized {
    const KIND: Kind;
    fn header(&self) -> Option<&Header>;
    fn header_mut(&mut self) -> &mut Option<Header>;
    /// Element count judged against the S-009-10 size limit, for the types that have one.
    fn size(&self) -> Option<usize> {
        None
    }
    /// Lets the publisher write its throttle count into `Health` (S-009-10).
    fn set_throttled(&mut self, _throttled: u64) {}
}

macro_rules! feed {
    ($ty:ident, $kind:expr $(, $extra:item)*) => {
        impl Feed for $ty {
            const KIND: Kind = $kind;
            fn header(&self) -> Option<&Header> {
                self.header.as_ref()
            }
            fn header_mut(&mut self) -> &mut Option<Header> {
                &mut self.header
            }
            $($extra)*
        }
    };
}

feed!(Spectrum, Kind::Spectrum,
    fn size(&self) -> Option<usize> { Some(self.bins_dbm.len()) }
);
feed!(Occupancy, Kind::Occupancy);
feed!(Bearing, Kind::Bearing);
feed!(LoraFrame, Kind::LoraFrame);
feed!(Scatter, Kind::Scatter,
    fn size(&self) -> Option<usize> { Some(self.points.len()) }
);
feed!(Calibration, Kind::Calibration);
feed!(Health, Kind::Health,
    fn set_throttled(&mut self, throttled: u64) { self.throttled = throttled; }
);

/// Declared rate and size limits; the defaults are S-009-10. A rate of 0 means unlimited.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Limits {
    pub health_hz: f64,
    pub occupancy_hz: f64,
    pub spectrum_hz: f64,
    pub scatter_hz: f64,
    pub spectrum_bins: usize,
    pub scatter_points: usize,
}

impl Default for Limits {
    /// S-009-10: Health 1 Hz; Occupancy ≤ 10 Hz; Spectrum ≤ 5 Hz at ≤ 4096 bins;
    /// Scatter ≤ 30 Hz at ≤ 4096 points; Bearing and LoraFrame per event.
    fn default() -> Self {
        Self {
            health_hz: 1.0,
            occupancy_hz: 10.0,
            spectrum_hz: 5.0,
            scatter_hz: 30.0,
            spectrum_bins: 4096,
            scatter_points: 4096,
        }
    }
}

impl Limits {
    /// The declared maximum rate of a kind; `None` for per-event types (S-009-10).
    pub fn rate_hz(&self, kind: Kind) -> Option<f64> {
        match kind {
            Kind::Health => Some(self.health_hz),
            Kind::Occupancy => Some(self.occupancy_hz),
            Kind::Spectrum => Some(self.spectrum_hz),
            Kind::Scatter => Some(self.scatter_hz),
            Kind::Bearing | Kind::LoraFrame | Kind::Calibration => None,
        }
    }

    /// The declared maximum element count of a kind (S-009-10).
    pub fn max_size(&self, kind: Kind) -> Option<usize> {
        match kind {
            Kind::Spectrum => Some(self.spectrum_bins),
            Kind::Scatter => Some(self.scatter_points),
            _ => None,
        }
    }
}

/// Enforces [`Limits`] (S-009-10): a message is admitted when at least one
/// period of its declared rate, less 1 % for timer jitter, has elapsed since
/// the last admitted message of its kind; otherwise it is dropped and counted.
#[derive(Debug)]
pub struct RateLimiter {
    limits: Limits,
    last: [Option<Instant>; 7],
    throttled: [u64; 7],
}

impl RateLimiter {
    pub fn new(limits: Limits) -> Self {
        Self { limits, last: [None; 7], throttled: [0; 7] }
    }

    pub fn limits(&self) -> &Limits {
        &self.limits
    }

    /// Decides one message at time `now`.
    pub fn admit(&mut self, kind: Kind, now: Instant) -> bool {
        let Some(hz) = self.limits.rate_hz(kind) else { return true };
        if hz <= 0.0 {
            return true;
        }
        let period = Duration::from_secs_f64(1.0 / hz).mul_f64(0.99);
        let i = kind as usize;
        match self.last[i] {
            Some(last) if now.duration_since(last) < period => {
                self.throttled[i] += 1;
                false
            }
            _ => {
                self.last[i] = Some(now);
                true
            }
        }
    }

    /// Messages of one kind dropped since start.
    pub fn throttled(&self, kind: Kind) -> u64 {
        self.throttled[kind as usize]
    }

    /// Messages of every kind dropped since start; what `Health.throttled` reports.
    pub fn throttled_total(&self) -> u64 {
        self.throttled.iter().sum()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum BusError {
    #[error("{kind} has {len} elements, the declared limit is {max} (SPEC-009 S-009-10)")]
    TooLarge { kind: Kind, len: usize, max: usize },
    #[error("zeromq: {0}")]
    Zmq(#[from] zeromq::ZmqError),
    #[error("protobuf decode: {0}")]
    Decode(#[from] prost::DecodeError),
    #[error("topic {0:?} is not qrf.v1.<Type>/<sensor_id> (SPEC-009 S-009-9)")]
    BadTopic(String),
    #[error("message has {0} frames; the bus carries exactly two: topic, payload")]
    Frames(usize),
    #[error("envelope carries {found}, asked to decode {wanted}")]
    WrongKind { found: Kind, wanted: Kind },
}

/// What [`Publisher::publish`] did with a message.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Publish {
    Sent,
    /// Dropped by the rate limiter and counted (S-009-10).
    Throttled,
}

/// One sensor's publisher (S-009-9): a ZeroMQ PUB socket bound to the sensor's
/// endpoints that stamps every message's [`Header`] and enforces [`Limits`].
pub struct Publisher {
    sock: PubSocket,
    sensor_id: String,
    seq: u64,
    limiter: RateLimiter,
    clock_quality: ClockQuality,
    power_ref: PowerRef,
}

/// A stale ipc socket file from a previous run makes `bind` fail; remove it
/// first, but only if it is a socket.
fn unlink_stale_ipc(endpoint: &str) {
    if let Some(path) = endpoint.strip_prefix("ipc://") {
        use std::os::unix::fs::FileTypeExt;
        if std::fs::metadata(path).map(|m| m.file_type().is_socket()).unwrap_or(false) {
            let _ = std::fs::remove_file(path);
        }
    }
}

impl Publisher {
    /// Binds every endpoint (`ipc://…` and optional `tcp://…`, S-009-9).
    pub async fn bind(sensor_id: &str, endpoints: &[&str], limits: Limits) -> Result<Self, BusError> {
        let mut sock = PubSocket::new();
        for ep in endpoints {
            unlink_stale_ipc(ep);
            sock.bind(ep).await?;
        }
        Ok(Self {
            sock,
            sensor_id: sensor_id.to_owned(),
            seq: 0,
            limiter: RateLimiter::new(limits),
            clock_quality: ClockQuality::Free,
            power_ref: PowerRef::Dbfs,
        })
    }

    /// The clock quality written into headers that do not set their own (S-009-4, S-009-6).
    pub fn set_clock_quality(&mut self, q: ClockQuality) {
        self.clock_quality = q;
    }

    /// The power reference written into headers that do not set their own (S-009-5).
    pub fn set_power_ref(&mut self, p: PowerRef) {
        self.power_ref = p;
    }

    pub fn sensor_id(&self) -> &str {
        &self.sensor_id
    }

    /// Sequence number of the last message sent (0 before the first).
    pub fn seq(&self) -> u64 {
        self.seq
    }

    pub fn limiter(&self) -> &RateLimiter {
        &self.limiter
    }

    /// Publishes one message on topic `qrf.v1.<Type>/<sensor_id>` after the
    /// S-009-10 checks: an oversized message is an error, a too-frequent one
    /// is dropped and counted. The header is completed, never overwritten:
    /// `sensor_id` and `seq` are always the publisher's; the two clocks are
    /// stamped now only if the sensor left them at zero; `clock_quality` and
    /// `power_ref` take the publisher's values only if unspecified.
    pub async fn publish<M: Feed>(&mut self, mut msg: M) -> Result<Publish, BusError> {
        if let (Some(len), Some(max)) = (msg.size(), self.limiter.limits.max_size(M::KIND))
            && len > max
        {
            return Err(BusError::TooLarge { kind: M::KIND, len, max });
        }
        if !self.limiter.admit(M::KIND, Instant::now()) {
            return Ok(Publish::Throttled);
        }
        self.seq += 1;
        let h = msg.header_mut().get_or_insert_with(Header::default);
        h.sensor_id = self.sensor_id.clone();
        h.seq = self.seq;
        if h.t_tai_ns == 0 && h.t_mono_ns == 0 {
            let s = Stamp::now();
            h.t_tai_ns = s.tai_ns;
            h.t_mono_ns = s.mono_ns;
        }
        if h.clock_quality() == ClockQuality::Unspecified {
            h.set_clock_quality(self.clock_quality);
        }
        if h.power_ref() == PowerRef::Unspecified {
            h.set_power_ref(self.power_ref);
        }
        msg.set_throttled(self.limiter.throttled_total());
        let mut frames = ZmqMessage::from(topic(M::KIND, &self.sensor_id));
        frames.push_back(Bytes::from(msg.encode_to_vec()));
        self.sock.send(frames).await?;
        Ok(Publish::Sent)
    }
}

/// One received message before decoding: its type, its sensor and its bytes.
#[derive(Clone, Debug)]
pub struct Envelope {
    pub kind: Kind,
    pub sensor_id: String,
    pub payload: Bytes,
}

/// Every message type, decoded (S-009-5).
#[derive(Clone, Debug, PartialEq)]
pub enum AnyFeed {
    Spectrum(Spectrum),
    Occupancy(Occupancy),
    Bearing(Bearing),
    LoraFrame(LoraFrame),
    Scatter(Scatter),
    Calibration(Calibration),
    Health(Health),
}

impl AnyFeed {
    pub fn kind(&self) -> Kind {
        match self {
            AnyFeed::Spectrum(_) => Kind::Spectrum,
            AnyFeed::Occupancy(_) => Kind::Occupancy,
            AnyFeed::Bearing(_) => Kind::Bearing,
            AnyFeed::LoraFrame(_) => Kind::LoraFrame,
            AnyFeed::Scatter(_) => Kind::Scatter,
            AnyFeed::Calibration(_) => Kind::Calibration,
            AnyFeed::Health(_) => Kind::Health,
        }
    }

    pub fn header(&self) -> Option<&Header> {
        match self {
            AnyFeed::Spectrum(m) => m.header(),
            AnyFeed::Occupancy(m) => m.header(),
            AnyFeed::Bearing(m) => m.header(),
            AnyFeed::LoraFrame(m) => m.header(),
            AnyFeed::Scatter(m) => m.header(),
            AnyFeed::Calibration(m) => m.header(),
            AnyFeed::Health(m) => m.header(),
        }
    }
}

impl Envelope {
    /// Decodes as one known type; an error if the topic says another.
    pub fn decode<M: Feed>(&self) -> Result<M, BusError> {
        if self.kind != M::KIND {
            return Err(BusError::WrongKind { found: self.kind, wanted: M::KIND });
        }
        Ok(M::decode(self.payload.as_ref())?)
    }

    /// Decodes as whatever the topic says it is.
    pub fn decode_any(&self) -> Result<AnyFeed, BusError> {
        Ok(match self.kind {
            Kind::Spectrum => AnyFeed::Spectrum(Spectrum::decode(self.payload.as_ref())?),
            Kind::Occupancy => AnyFeed::Occupancy(Occupancy::decode(self.payload.as_ref())?),
            Kind::Bearing => AnyFeed::Bearing(Bearing::decode(self.payload.as_ref())?),
            Kind::LoraFrame => AnyFeed::LoraFrame(LoraFrame::decode(self.payload.as_ref())?),
            Kind::Scatter => AnyFeed::Scatter(Scatter::decode(self.payload.as_ref())?),
            Kind::Calibration => AnyFeed::Calibration(Calibration::decode(self.payload.as_ref())?),
            Kind::Health => AnyFeed::Health(Health::decode(self.payload.as_ref())?),
        })
    }
}

/// A ZeroMQ SUB socket connected to one or more publishers, filtering by topic prefix (S-009-9).
pub struct Subscriber {
    sock: SubSocket,
}

impl Subscriber {
    pub async fn connect(endpoints: &[&str]) -> Result<Self, BusError> {
        let mut sock = SubSocket::new();
        for ep in endpoints {
            sock.connect(ep).await?;
        }
        Ok(Self { sock })
    }

    /// Subscribes to a prefix: [`ALL_TOPICS`], [`topic_prefix`] of one kind, or [`topic`] of one sensor and kind.
    pub async fn subscribe(&mut self, prefix: &str) -> Result<(), BusError> {
        Ok(self.sock.subscribe(prefix).await?)
    }

    pub async fn unsubscribe(&mut self, prefix: &str) -> Result<(), BusError> {
        Ok(self.sock.unsubscribe(prefix).await?)
    }

    /// The next message: two frames, topic then payload.
    pub async fn recv(&mut self) -> Result<Envelope, BusError> {
        let frames = self.sock.recv().await?.into_vec();
        if frames.len() != 2 {
            return Err(BusError::Frames(frames.len()));
        }
        let topic = String::from_utf8_lossy(&frames[0]).into_owned();
        let (kind, sensor_id) = parse_topic(&topic).ok_or_else(|| BusError::BadTopic(topic.clone()))?;
        Ok(Envelope { kind, sensor_id: sensor_id.to_owned(), payload: frames[1].clone() })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_stable() {
        assert_eq!(NAME, "qrf-bus");
    }

    #[test]
    fn topics_follow_s_009_9() {
        assert_eq!(topic(Kind::Bearing, "tile0"), "qrf.v1.Bearing/tile0");
        assert_eq!(topic_prefix(Kind::LoraFrame), "qrf.v1.LoraFrame/");
        assert!(topic(Kind::Health, "x").starts_with(ALL_TOPICS));
        for k in Kind::ALL {
            assert_eq!(parse_topic(&topic(k, "sensor-7")), Some((k, "sensor-7")));
        }
        assert_eq!(parse_topic("qrf.v2.Bearing/tile0"), None);
        assert_eq!(parse_topic("qrf.v1.Nothing/tile0"), None);
        assert_eq!(parse_topic("qrf.v1.Bearing"), None);
    }

    #[test]
    fn limits_default_to_s_009_10() {
        let l = Limits::default();
        assert_eq!(l.rate_hz(Kind::Health), Some(1.0));
        assert_eq!(l.rate_hz(Kind::Occupancy), Some(10.0));
        assert_eq!(l.rate_hz(Kind::Spectrum), Some(5.0));
        assert_eq!(l.rate_hz(Kind::Scatter), Some(30.0));
        assert_eq!(l.rate_hz(Kind::Bearing), None);
        assert_eq!(l.rate_hz(Kind::LoraFrame), None);
        assert_eq!(l.max_size(Kind::Spectrum), Some(4096));
        assert_eq!(l.max_size(Kind::Scatter), Some(4096));
        assert_eq!(l.max_size(Kind::Occupancy), None);
    }

    #[test]
    fn limiter_admits_at_most_the_declared_rate() {
        let mut lim = RateLimiter::new(Limits::default());
        let t0 = Instant::now();
        // 1 000 attempts one millisecond apart at 10 Hz declared: the effective period is 99 ms
        // (100 ms less the 1 % jitter allowance), so t = 0, 99, 198, …, 990 ms are admitted: 11 of 1 000.
        let admitted = (0..1000).filter(|i| lim.admit(Kind::Occupancy, t0 + Duration::from_millis(*i))).count();
        assert_eq!(admitted, 11);
        assert_eq!(lim.throttled(Kind::Occupancy), 989);
        // Per-event kinds are never throttled.
        assert!((0..1000).all(|i| lim.admit(Kind::Bearing, t0 + Duration::from_micros(i))));
        assert_eq!(lim.throttled(Kind::Bearing), 0);
        assert_eq!(lim.throttled_total(), 989);
        // 10 ms after the last admission (990 ms) is refused; 99 ms after it is admitted.
        assert!(!lim.admit(Kind::Occupancy, t0 + Duration::from_millis(1000)));
        assert!(lim.admit(Kind::Occupancy, t0 + Duration::from_millis(1089)));
    }

    #[test]
    fn ipc_endpoint_follows_s_009_9() {
        assert_eq!(ipc_endpoint(Path::new(DEFAULT_RUN_DIR), "tile0"), "ipc:///run/qrf/tile0.pub");
    }
}
