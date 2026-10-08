# Skyworks Solutions, Inc. — SKY65404-31 and SE5004L

| Field | Value |
| --- | --- |
| Copyright holder | Skyworks Solutions, Inc. ("Skyworks Proprietary Information" on every page) |
| Licence | Proprietary data sheets; not redistributable |
| Documents | `resources/datasheets/SKY65404-31.pdf`: "SKY65404-31: 5 GHz Low-Noise Amplifier", document 201512K, 2015-11-06, 9 pages (the PDF carries an "Alldatasheet" title tag, i.e. it was downloaded from a mirror; content is the Skyworks data sheet). `resources/datasheets/SE5004L.pdf`: "SE5004L-EK1: 802.11a/n 26dBm WLAN RF Power Amplifier Evaluation Kit", document 202643A, 2012-12-11, 5 pages (file arrived as `se5004l-ek1_202643a.pdf`). Both supplied by the user through the UTM share on 2026-10-08. The SE5004L *device* data sheet (DST-00316) is still missing. |
| Product pages | https://www.skyworksinc.com/ (SKY65404-31; SE5004L is a legacy SiGe Semiconductor part) |

## What we use

LNA noise figure, gain, linearity and current for SPEC-001 S-001-38 and the
cascade in `analysis/linkbudget.py` T18/T21; the PA's linear power and gain
for S-001-39. Restated in `vendor/summary/skyworks.md`. No figure, table or
text is reproduced.
