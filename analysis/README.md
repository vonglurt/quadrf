# analysis/

`linkbudget.py` is the single source for every `[D]` number in the documents:
LoRa sensitivities, free-space and foliage losses, knife-edge diffraction,
Fresnel radii, radio horizon, regulatory EIRP cases, array geometry (pitch,
beamwidth, grating-lobe steer limit, DoA ambiguity), narrowband validity,
RF-exposure distances, LO-wander-to-bin ratios, and ADC quantisation penalty.

```
python3 analysis/linkbudget.py
```

Tables are numbered T1…T13 and are referenced by that number from the
investigations. No third-party dependencies.
