#!/usr/bin/env python3
"""Reproducible numerical basis for the quadrf investigation documents.

Every number quoted in investigations/ and specs/ that is marked [D] (derived)
is produced here. Run:  python3 analysis/linkbudget.py
No third-party dependencies.
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
    print("T11 RF exposure (47 CFR 1.1310 general population MPE): 902-928 -> f/1500 = 0.61 mW/cm2; 5.8 GHz -> 1.0 mW/cm2")
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
