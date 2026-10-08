#!/usr/bin/env python3
"""Reproducible numerical basis for the quadrf investigation documents.

Every number quoted in investigations/, specs/, lab/ and backlog.md that is
marked [D] (derived) is produced here. Run:  python3 analysis/linkbudget.py
No third-party dependencies. Tables are numbered T1…T19 and cited by number.

Inputs marked [S] in comments are sourced values (vendor documents, datasheets
in resources/datasheets/, CFR text); inputs marked [C] are placeholders that a
datasheet or a measurement will replace.
"""
import math

C = 299_792_458.0
K_dBm_Hz = -174.0            # kT at 290 K, dBm/Hz

# --- LoRa demodulator SNR thresholds (Semtech SX126x datasheet, CR 4/5, BER 1e-3 class)
SNR_MIN = {7: -7.5, 8: -10.0, 9: -12.5, 10: -15.0, 11: -17.5, 12: -20.0}

# --- Meshtastic presets (bandwidth kHz, SF, CR)
PRESETS = {
    "SHORT_TURBO":   (500, 7,  "4/5"),
    "SHORT_FAST":    (250, 7,  "4/5"),
    "MEDIUM_FAST":   (250, 9,  "4/5"),
    "LONG_TURBO":    (500, 11, "4/8"),
    "LONG_FAST":     (250, 11, "4/5"),
    "LONG_MODERATE": (125, 11, "4/8"),
    "LONG_SLOW":     (125, 12, "4/8"),
}

def sens(bw_hz, sf, nf_db):
    return K_dBm_Hz + 10*math.log10(bw_hz) + nf_db + SNR_MIN[sf]

def fspl(d_km, f_mhz):
    return 32.45 + 20*math.log10(d_km) + 20*math.log10(f_mhz)

def weissberger(d_m, f_ghz):
    """Modified exponential decay foliage model, 230 MHz-95 GHz, valid 14..400 m."""
    if d_m <= 14:
        return 0.45 * f_ghz**0.284 * d_m
    return 1.33 * f_ghz**0.284 * d_m**0.588

def knife_edge_J(v):
    """ITU-R P.526 single knife-edge approximation, valid v > -0.78."""
    return 6.9 + 20*math.log10(math.sqrt((v-0.1)**2 + 1) + v - 0.1)

def fresnel_r1(f_mhz, d1_km, d2_km):
    lam = C / (f_mhz*1e6)
    d1, d2 = d1_km*1e3, d2_km*1e3
    return math.sqrt(lam*d1*d2/(d1+d2))

def radio_horizon_km(h1_m, h2_m):
    return 4.12*(math.sqrt(h1_m) + math.sqrt(h2_m))   # 4/3 effective earth

def mpe_distance_m(eirp_w, s_limit_w_cm2):
    return math.sqrt(eirp_w / (4*math.pi*s_limit_w_cm2)) / 100.0

def hpbw_2el_deg(d_over_lambda):
    """Half-power beamwidth of a 2-element broadside pair, isotropic elements."""
    # AF = |cos(pi d/lambda sin th)| ; find th where AF^2 = 1/2
    x = math.acos(1/math.sqrt(2)) / (math.pi*d_over_lambda)
    return 2*math.degrees(math.asin(min(1.0, x)))

def grating_steer_limit_deg(d_over_lambda):
    """Max steer angle before a grating lobe enters visible space."""
    s = 1/d_over_lambda - 1
    return 90.0 if s >= 1 else math.degrees(math.asin(max(0.0, s)))

def unambiguous_doa_deg(d_over_lambda):
    s = 1/(2*d_over_lambda)
    return 90.0 if s >= 1 else math.degrees(math.asin(s))

def uca_radius_m(s, lam_m, n):
    """KrakenSDR-style uniform circular array: inter-element spacing s*lambda."""
    return s*lam_m / math.sqrt(2*(1 - math.cos(2*math.pi/n)))

def db(x): return 10*math.log10(x)
def lin(x_db): return 10**(x_db/10)

def line(): print("-"*78)

if __name__ == "__main__":
    print("T1  LoRa receiver sensitivity (dBm) = -174 + 10log10(BW) + NF + SNRmin")
    print(f"{'preset':14}{'BW kHz':>8}{'SF':>4}{'CR':>5}{'SX1262 NF6':>12}{'QuadRF NF4.2':>14}")
    for name,(bw,sf,cr) in PRESETS.items():
        print(f"{name:14}{bw:8}{sf:4}{cr:>5}{sens(bw*1e3,sf,6.0):12.1f}{sens(bw*1e3,sf,4.2):14.1f}")
    line()
    print("T2  Free-space path loss (dB)")
    print(f"{'d km':>6}{'915 MHz':>10}{'5800 MHz':>10}{'delta':>8}")
    for d in (5,10,25,40,60,80,120):
        a,b = fspl(d,915), fspl(d,5800)
        print(f"{d:6}{a:10.1f}{b:10.1f}{b-a:8.1f}")
    line()
    print("T3  Foliage excess loss, Weissberger (dB)")
    print(f"{'depth m':>8}{'915 MHz':>10}{'5800 MHz':>10}")
    for dm in (10,20,50,100,200,400):
        print(f"{dm:8}{weissberger(dm,0.915):10.1f}{weissberger(dm,5.8):10.1f}")
    line()
    print("T4  Single knife-edge diffraction loss J(v) (dB), ITU-R P.526")
    for v in (-0.5,0,0.5,1,2,3,5):
        print(f"  v={v:5.1f}  J={knife_edge_J(v):5.1f}")
    line()
    print("T5  First Fresnel radius at midpoint (m)")
    for d in (10,25,40,80):
        print(f"  d={d:3} km  915: {fresnel_r1(915,d/2,d/2):5.1f}   5800: {fresnel_r1(5800,d/2,d/2):5.1f}")
    line()
    print("T6  Radio horizon, 4/3 earth (km)")
    for h1,h2 in ((2,300),(2,1200),(10,1200),(30,2000)):
        print(f"  h_user={h1:3} m  h_summit={h2:5} m  ->  {radio_horizon_km(h1,h2):6.1f} km")
    line()
    print("T7  Regulatory EIRP ceilings and worked link margins, 40 km LOS, Rx 6 dBi omni")
    cases = [
        ("Part15 902-928 any antenna (36 dBm cap)", 36.0, 915, 6.0),
        ("Part15 5725-5850 P2P, 1 W + 12 dBi tile", 42.0, 5800, 12.0),
        ("Part15 5725-5850 P2P, 1 W + 72-el 24.6 dBi", 54.6, 5800, 24.6),
        ("Part97 33cm SS 10 W PEP + 12 dBi yagi", 52.0, 915, 12.0),
        ("Part97 33cm SS 10 W PEP + 24.6 dBi array", 64.6, 915, 24.6),
    ]
    for name,eirp,f,grx in cases:
        pl = fspl(40,f)
        prx = eirp - pl + grx
        s_lf = sens(250e3,11,6.0) if f<2000 else sens(250e3,11,4.2)
        s_st = sens(500e3,7,6.0) if f<2000 else sens(500e3,7,4.2)
        print(f"  {name:46} EIRP {eirp:5.1f} dBm  Prx {prx:7.1f} dBm  margin LF {prx-s_lf:5.1f}  ST {prx-s_st:5.1f}")
    line()
    print("T8  Part 15.247(b)(4) conducted-power reduction, 902-928 MHz: Pc_max = 30 - max(0, G-6)")
    for g in (2,6,9,12,15,24):
        print(f"  G={g:2} dBi  Pc_max={30-max(0,g-6):5.1f} dBm  EIRP={min(30,30-max(0,g-6))+g:5.1f} dBm")
    line()
    print("T9  Array geometry, QuadRF tile pitch 45.5 mm")
    for f in (4900,5500,5800,6000):
        lam = C/(f*1e6); dl = 0.0455/lam
        print(f"  f={f} MHz  lambda={lam*1000:5.1f} mm  d/lambda={dl:5.3f}  HPBW2el={hpbw_2el_deg(dl):5.1f} deg"
              f"  steer-limit(no GL)={grating_steer_limit_deg(dl):5.1f} deg  DoA-unambiguous=+-{unambiguous_doa_deg(dl):5.1f} deg")
    lam915 = C/915e6
    print(f"  915 MHz lambda={lam915*1000:5.1f} mm; pitch for identical d/lambda as 5800: {0.0455*5800/915*1000:5.0f} mm;"
          f" half-wave pitch: {lam915/2*1000:5.0f} mm")
    print(f"  Array gain over element, N=4: {10*math.log10(4):4.1f} dB; N=72: {10*math.log10(72):4.1f} dB; N=240: {10*math.log10(240):4.1f} dB")
    line()
    print("T10 Narrowband (phase-steer) validity: aperture transit time vs 1/BW")
    for ap_m,bw in ((0.1,40e6),(0.3,500e3),(0.6,500e3),(1.0,125e3)):
        tau = ap_m/C
        print(f"  aperture {ap_m:3.1f} m  tau={tau*1e9:5.2f} ns  1/BW={1/bw*1e6:8.2f} us  ratio={tau*bw:9.2e}")
    line()
    print("T11 RF exposure (47 CFR 1.1310 Table 1, general population MPE): 300-1500 MHz -> f/1500 = 0.61 mW/cm2 at 915; 1500-100000 MHz -> 1.0 mW/cm2")
    for name,eirp_dbm,s in (("36 dBm @915",36,0.61e-3),("52 dBm @915",52,0.61e-3),("64.6 dBm @915",64.6,0.61e-3),("42 dBm @5800",42,1.0e-3),("54.6 dBm @5800",54.6,1.0e-3)):
        print(f"  {name:16} compliance distance (far-field formula) = {mpe_distance_m(10**(eirp_dbm/10)/1000, s):5.2f} m")
    line()
    print("T12 LoRa symbol/chip timing vs measured QuadRF-pair LO wander (800 Hz RMS)")
    for name,(bw,sf,cr) in PRESETS.items():
        ts = (2**sf)/(bw*1e3); binw = bw*1e3/(2**sf)
        print(f"  {name:14} Tsym={ts*1e3:7.2f} ms  bin={binw:7.1f} Hz  800Hz/bin={800/binw:6.2f}")
    line()
    print("T13 8-bit ADC quantisation vs thermal floor: with noise rms = k LSB, added noise power (dB)")
    for k in (0.5,1,2,4):
        q = (1/math.sqrt(12))**2; n=k**2
        print(f"  k={k:3}  10log10(1+q/n) = {10*math.log10(1+q/n):5.2f} dB")
    line()
    # ------------------------------------------------------------------
    # Second-pass tables (2026-10-08): data paths, 915 MHz apertures, LO
    # quality, cascade NF from the MAX2851 datasheet, CPU budget, USB power.
    # ------------------------------------------------------------------
    print("T14 Data-path budgets (bit/s), Pi 5 + QuadRF tile")
    lane_mbps = 700.0                     # [S] fpga-csi.dts: 350 MHz DDR -> 700 Mbps per lane
    csi_gbps = 4*lane_mbps/1e3           # 4 lanes
    print(f"  CSI-2 4 lanes x {lane_mbps:.0f} Mbit/s = {csi_gbps:.2f} Gbit/s raw; DSI same -> {2*csi_gbps:.1f} Gbit/s aggregate (vendor: 5.6)")
    print(f"  RP1 MIPI aggregate (datasheet): 8 Gbit/s; PCIe 2.0 x4 to BCM2712: 4 x 5 GT/s x 8/10 = {4*5*0.8:.0f} Gbit/s; USB 3.0 per port 5 GT/s x 8/10 = 4 Gbit/s")
    for label,ch,msps in (("1 ch, 40 MSPS",1,40),("4 ch interleaved, 26 MSPS",4,26),("4 ch interleaved, 40 MSPS",4,40),("PhaseGaze 1 ch, 38 MSPS",1,38)):
        gbps = ch*msps*1e6*16/1e9
        print(f"  {label:28} CS8: {gbps:5.2f} Gbit/s = {gbps/8*1000:6.0f} MB/s  ({100*gbps/csi_gbps:4.0f} % of CSI raw)")
    frame_bytes = 1024*128                # [S] fpga-csi.dts geometry, RAW8
    rate_Bps = 4*26e6*2
    frame_us = frame_bytes/rate_Bps*1e6
    print(f"  CSI frame = 1024 B x 128 lines = {frame_bytes} B; at 4 ch x 26 MSPS one frame every {frame_us:5.0f} us;"
          f" DMA_BUF_COUNT=16 -> {16*frame_us/1000:4.1f} ms of kernel-side buffering before loss")
    kr = 5*2.4e6*2*8/1e6
    print(f"  KrakenSDR 5 ch x 2.4 MSPS x 2 B = {kr:5.0f} Mbit/s ({100*kr/480:3.0f} % of one USB 2.0 link)")
    line()
    print("T15 915 MHz receive apertures: 4-element square (ours) and 5-element UCA (KrakenSDR rule s<=0.5, typical 0.33)")
    for pitch_mm in (164, 288):
        dl = pitch_mm/1000/lam915
        print(f"  square pitch {pitch_mm} mm  d/lambda={dl:5.3f}  HPBW2el={hpbw_2el_deg(dl):5.1f} deg"
              f"  steer-limit={grating_steer_limit_deg(dl):5.1f} deg  unambiguous=+-{unambiguous_doa_deg(dl):5.1f} deg  diagonal={pitch_mm*math.sqrt(2):4.0f} mm")
    for s in (0.33, 0.5):
        r = uca_radius_m(s, lam915, 5)
        D = 2*r
        ray = math.degrees(1.22*lam915/D) if 1.22*lam915/D < 1 else math.degrees(math.asin(min(1.0,1.22*lam915/D)))
        print(f"  UCA n=5 s={s:4.2f}: spacing {s*lam915*1000:5.1f} mm  radius {r*1000:5.1f} mm  aperture {D*1000:5.1f} mm  Rayleigh 1.22*lambda/D = {math.degrees(1.22*lam915/D):5.1f} deg (MUSIC ~10x finer per vendor)")
    line()
    print("T16 Bearing precision (CRLB, two-element phase difference) vs calibration: sigma_theta = 1/(sqrt(N*SNR) * k d cos(theta))")
    kd = 2*math.pi*0.5                    # half-wave pitch
    for snr_db, n in ((0,1024),(10,1024),(20,1024),(20,16384)):
        sig = 1/math.sqrt(n*lin(snr_db))/kd
        print(f"  SNR {snr_db:3} dB, N={n:5} samples (broadside): sigma_theta = {math.degrees(sig):6.3f} deg")
    print("  -> thermal noise is not what limits the G05 5-degree criterion; static phase calibration and multipath are.")
    line()
    print("T17 MAX2851 LO quality (datasheet 19-5121 Rev 1) vs 8-bit converter floor")
    ipn_dbc = -35.0                       # [S] integrated phase noise, 1 kHz-10 MHz, loop BW 200 kHz
    rms_rad = math.sqrt(2*lin(ipn_dbc))
    print(f"  integrated phase noise {ipn_dbc:.0f} dBc -> rms phase {rms_rad:.4f} rad = {math.degrees(rms_rad):.2f} deg")
    sfdr8 = 6.02*8 + 1.76
    print(f"  fractional spur level (0-19 MHz offset) -42 dBc typ [S]; 8-bit single-tone SFDR ~ {sfdr8:.1f} dB -> spur is above the quantiser floor for a near-full-scale signal")
    for p_in in (-30.0, -60.0):
        print(f"  strong in-band emitter at {p_in:4.0f} dBm -> LO-spur replica at {p_in-42:5.0f} dBm, i.e. {p_in-42-(-130):3.0f} dB above a -130 dBm LoRa signal in the replica's slot")
    wander = 800.0
    print(f"  measured pair wander 800 Hz rms at 5800 MHz = {wander/5.8e9*1e6:.3f} ppm; PLL step 40e6/2^19 = {40e6/2**19:.3f} Hz [S]")
    line()
    print("T18 Receive cascade NF: SKY65404-31 LNA (datasheet 201512K: NF 0.8/1.0/1.5 dB min/typ/max, gain 11/13/16 dB) ahead of the MAX2851 (4.5 dB DSB) [S]")
    for label, lna_nf, lna_g, pre_loss in (("typ, no loss ahead", 1.0, 13.0, 0.0), ("typ, 0.5 dB switch ahead", 1.0, 13.0, 0.5),
                                            ("best corner, no loss", 0.8, 16.0, 0.0), ("worst corner, 0.5 dB", 1.5, 11.0, 0.5)):
        F = lin(pre_loss) * (lin(lna_nf) + (lin(4.5) - 1)/lin(lna_g))
        print(f"  {label:26} LNA NF {lna_nf:3.1f} dB G {lna_g:4.1f} dB -> system NF {db(F):4.2f} dB")
    print("  vendor states ~1.2 dB [S]; the datasheet-typical chain gives 1.30 dB (1.80 dB with 0.5 dB ahead); 1.2 dB is reached only near the best-case corner")
    for tile_nf in (1.3, 1.8):
        F = lin(1.0) + (lin(7.0)-1)/lin(20.0) + (lin(tile_nf)-1)/(lin(20.0)*lin(-7.0))
        print(f"  FTFE (915 MHz): LNA 1.0 dB/20 dB, mixer+filter loss 7 dB, tile {tile_nf:3.1f} dB -> NF {db(F):4.2f} dB (no pre-filter); + 2 dB SAW ahead -> {db(F)+2:4.2f} dB")
    line()
    print("T19 CPU budget for a whole-band channeliser on the Pi 5 (4x Cortex-A76, 2.4 GHz)")
    M, P = 128, 8                          # 128 bins over 26 MHz -> 203 kHz bins (>= one 250 kHz slot per 1.23 bins); P taps per phase
    flops_per_sample = 4*P + 5*math.log2(M)   # polyphase filter (complex MAC ~4 flop) + FFT (5 N log2 N / N)
    per_ch = 26e6*flops_per_sample
    peak = 2.4e9*16                        # 2 FP pipes x 4-lane FMA x 2 flop
    print(f"  M={M} bins, P={P} taps: {flops_per_sample:.0f} flop/sample -> {per_ch/1e9:4.2f} GFLOP/s per channel, {4*per_ch/1e9:4.1f} GFLOP/s for 4 channels")
    print(f"  A76 NEON peak {peak/1e9:4.0f} GFLOP/s per core; at 25 % efficiency {0.25*peak/1e9:4.0f} GFLOP/s -> 4-ch channeliser = {4*per_ch/(0.25*peak):4.2f} cores")
    print(f"  quadrf-mesh measured PHY cost 0.35 core at 8 MSPS single channel [S]; decoding <=4 selected 250 kHz slots from channeliser outputs is < 0.1 core each [C]")
    line()
    print("T20 USB power on the Pi 5: peripherals budget 1.6 A at 5 V with a 5 A PD supply (0.6 A otherwise) [S Pi documentation]")
    for name, a in (("KrakenSDR (needs own supply)", 2.2), ("RTL-SDR v4", 0.3), ("HackRF One", 0.5), ("USB LoRa stick (CH341+SX1262, 22 dBm)", 0.2)):
        print(f"  {name:40} {a:3.1f} A -> {'exceeds' if a > 1.6 else 'within'} the 1.6 A budget")
    line()
    print("T21 Antenna-referred compression of the tile receive chain, and FTFE level plan")
    lna_g = 13.0                                   # [S] SKY65404-31 typ
    print(f"  LNA input P1dB -4 dBm [S]; MAX2851 input P1dB -34 dBm at max gain, -18 at max-16 dB, -1 at max-32 dB [S]")
    for setting, p1 in (("max gain", -34.0), ("max - 16 dB", -18.0), ("max - 32 dB", -1.0)):
        ant = min(p1 - lna_g, -4.0)
        print(f"  tile RF gain {setting:12}: chain compresses at {ant:6.1f} dBm referred to the element port (IC-limited unless the LNA's -4 dBm is lower)")
    for p_node_dbm, d_m in ((22.0, 10.0), (22.0, 100.0), (30.0, 10.0)):
        p_ant = p_node_dbm + 2.15 - fspl(d_m/1000, 915) + 2.15   # dipole-class antennas both ends
        p_port = p_ant + 10.0                       # FTFE net gain +10 dB (SPEC-007 S-007-5)
        print(f"  co-sited node {p_node_dbm:4.1f} dBm at {d_m:5.1f} m -> {p_ant:6.1f} dBm at the 915 MHz antenna, {p_port:6.1f} dBm at the element port after +10 dB FTFE gain"
              f" -> needs tile RF gain <= max-{'32' if p_port > -31 else ('16' if p_port > -47 else '0')} dB or the 30 dB FTFE pad")
    line()
    print("T22 FTFE LO candidate: MAX2871 (datasheet 19-7106 Rev 4) at the translation LO; in-band floor = -230 + 20log10(N) + 10log10(fPFD) [S]+[D]")
    for label, f_lo, f_pfd, mode in (("frac-N, 40 MHz PFD, 4585 MHz", 4585e6, 40e6, "fractional"), ("int-N, 20 MHz PFD, 4580 MHz", 4580e6, 20e6, "integer"),
                                      ("int-N, 5 MHz PFD, 4585 MHz", 4585e6, 5e6, "integer"), ("int-N, 40 MHz PFD, 4600 MHz", 4600e6, 40e6, "integer")):
        N = f_lo/f_pfd
        floor = -230 + 20*math.log10(N) + 10*math.log10(f_pfd)
        onef_10k = -122 + 20*math.log10(f_lo/1e9)   # 1/f term at 10 kHz offset (Note 7)
        total_10k = db(lin(floor) + lin(onef_10k))
        print(f"  {label:32} N={N:8.3f} ({mode:10}) in-band floor {floor:7.1f} dBc/Hz; 1/f at 10 kHz {onef_10k:7.1f}; total at 10 kHz {total_10k:7.1f} dBc/Hz; band 915 MHz -> {(f_lo+902e6)/1e6:.0f}-{(f_lo+928e6)/1e6:.0f} MHz")
    # rough integrated phase noise 1 kHz-10 MHz with a 150 kHz loop and the 4500 MHz VCO curve (-106 dBc/Hz at 100 kHz, -20 dB/dec)
    floor = -230 + 20*math.log10(4585e6/40e6) + 10*math.log10(40e6)
    inband = lin(floor) * (150e3 - 1e3)
    vco_100k = -106.0
    outband = lin(vco_100k) * (100e3**2) * (1/150e3 - 1/10e6)
    ipn = db(inband + outband)
    print(f"  integrated 1 kHz-10 MHz (150 kHz loop, VCO -106 dBc/Hz at 100 kHz): ~{ipn:5.1f} dBc -> rms {math.degrees(math.sqrt(2*lin(ipn))):5.2f} deg, vs tile LO -35 dBc = 1.44 deg (T17)")
    print("  PFD spurs -88 dBc at 50 kHz loop [S] vs tile LO fractional spurs -42 dBc [S]: the translator's own spurs are 46 dB below the tile's")
    print("  REF_IN 10-210 MHz accepts the tile's 40 MHz reference if it can be exported (U-001-2) [S]; supply 3.3 V, <= 200 mA both outputs [S]")
    line()
    print("T23 SX1261/2 datasheet (DS.SX1261-2.W.APP Rev 1.1, Table 3-8, Rx boosted gain) vs the T1 model; implied NF = P_sens + 174 - 10log10(BW) - SNRmin [S]+[D]")
    DS = {(125e3, 7): -124.0, (125e3, 12): -137.0, (250e3, 7): -121.0, (250e3, 12): -134.0, (500e3, 7): -117.0, (500e3, 12): -129.0, (10.4e3, 12): -148.0, (10.4e3, 7): -134.0}
    for (bw, sf), ps in sorted(DS.items()):
        nf_impl = ps + 174 - 10*math.log10(bw) - SNR_MIN[sf]
        print(f"  BW {bw/1e3:6.1f} kHz SF{sf:2}: datasheet {ps:7.1f} dBm; T1 model NF 6 dB {sens(bw, sf, 6.0):7.1f} dBm; implied NF+impl. loss {nf_impl:4.1f} dB")
    print("  -> the SX126x sensitivity model should use NF ~ 7 dB (6.5-8.0 implied) rather than 6 dB; T1/T7 margins for SX1262 receivers are 0.5-2 dB optimistic")
    print(f"  Meshtastic presets by SF-interpolation of the datasheet rows (2.6 dB per SF step at 250 kHz): LONG_FAST (250k/SF11) ~ {-121 + (-134+121)/5*4:6.1f} dBm; SHORT_TURBO (500k/SF7) {-117:6.1f} dBm; LONG_TURBO (500k/SF11) ~ {-117 + (-129+117)/5*4:6.1f} dBm")
    print("  Tolerated Tx-Rx frequency offset: +/-25 % of BW (all SF); tighter ppm limits SF12 +/-50, SF11 +/-100, SF10 +/-200 ppm [S]:")
    for name, (bw, sf, cr) in PRESETS.items():
        lim_bw = 0.25*bw*1e3
        lim_ppm = {12: 50, 11: 100, 10: 200}.get(sf)
        lim = min(lim_bw, lim_ppm*915) if lim_ppm else lim_bw
        print(f"    {name:14} limit {lim/1e3:6.2f} kHz ({'ppm rule' if lim_ppm and lim_ppm*915 < lim_bw else '25 % BW rule'}); a 1 ppm free-running translator LO at 4585 MHz (4.6 kHz error, G05 F.05.2) uses {100*4585/lim:4.1f} % of it")
    print("  SX126x synthesiser phase noise at 868/915 MHz: -75/-95/-100/-120/-135 dBc/Hz at 1k/10k/100k/1M/10M [S]; step 0.95 Hz; LDRO recommended for Tsym >= 16.38 ms [S]")
