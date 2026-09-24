# WORKFLOW: 3D SPATIAL ACOUSTICS & PORTAL PROPAGATION

**Execution Trigger**: `"setup spatial audio"`, `"higher order ambisonics"`, `"binaural hrtf"`, `"room acoustics occlusion"`  
**Target Systems**: C++17 GDExtension, Web Audio API, Unity Native Audio  
**Primary Goal**: Implement 3rd-order HOA, 3D VBAP, near-field DVTF parallax ($r < 1\text{ m}$), Delany-Bazley ground reflections, and portal diffraction.

---

## 1. SPATIAL ACOUSTIC PIPELINE

```
                                [3D POINT SOURCE (r, theta, phi)]
                                                │
                 ┌──────────────────────────────┼──────────────────────────────┐
                 ▼                              ▼                              ▼
    [HOA 3RD ORDER ENCODER]            [3D VBAP SPATIALIZER]         [NEAR-FIELD BINAURAL]
    - 16 Spherical Harmonics           - Delaunay Triangulation      - Ear Parallax Correction
    - Max-rE High-Freq Weighting       - L2 Energy Normalization     - DVTF Low-Shelf Bass Boost
                 │                              │                              │
                 ▼                              ▼                              ▼
    [WIGNER-D QUATERNION ROTATION]     [LOUDSPEAKER GAINS]           [FRACTIONAL ITD / ILD]
                 │
                 ▼
    [DELANY-BAZLEY GROUND DIP & METEOROLOGICAL REFRACTION RAY-TRACING]
```

---

## 2. C++17 INTEGRATION WORKFLOW

Include `scripts/advanced_spatial_acoustics.h` directly in your game audio pipeline:

```cpp
#include "scripts/advanced_spatial_acoustics.h"

using namespace SpatialAcoustics;

// 1. Initialize 3rd Order Ambisonic Encoder
HigherOrderAmbisonics3D hoa_encoder;
std::array<float, 16> b_format_frame;

// Encode mono source at Azimuth 45 deg, Elevation 10 deg
hoa_encoder.encode(audio_sample, 45.0f * PI / 180.0f, 10.0f * PI / 180.0f, b_format_frame);

// 2. Process Near-Field Binaural Sound (e.g. Ear-level flying insect at 0.25 meters)
NearFieldBinauralSpatializer near_field_spat(48000.0f);
float out_left, out_right;
near_field_spat.process(audio_sample, 45.0f, 0.0f, 0.25f, out_left, out_right);

// 3. Apply Outdoor Ground Impedance (Delany-Bazley Notch)
OutdoorGroundAtmosphericDSP ground_dsp(48000.0f);
float outdoor_sample = ground_dsp.process(audio_sample, 50.0f, 1.8f, 1.8f, 200.0f, -0.05f);
```
