---
name: procedural-audio
version: 1.0.0
format: skill/1.0
description: CIEL's framework for procedural audio synthesis, spatial acoustics, and generative soundscapes from scene graphs. Zero static sample dependencies — pure DSP code emission.
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: MIT
tags: ["ciel", "harmonized", "domain:ai", "audio", "dsp", "synthesis", "procedural", "psychoacoustics", "spatial-audio", "game-audio"]
triggers:
  - pattern: "(procedural audio|soundscape|create audio for this screen|sound effect|sfx|synthesizer|dsp|generative music|adaptive audio)"
    confidence: 0.9
  - pattern: "(godot audio|web audio|audiostreamgenerator|audio worklet|psychoacoustics|spatial audio|ambisonics|microtonal)"
    confidence: 0.85
source: { tier: 0, origin: seed }
side_effects: ["fs"]
dependencies: { skills: [], mcp: [], system: ["python3"] }
---

# CIEL ADAPTATION: Procedural Audio & Autonomous Audiography

Synthesizes studio-grade sound effects, adaptive music, and spatial acoustic environments purely from code and mathematical DSP. Given a minimalist prompt like `"create the audio for this screen"`, autonomously extracts the scene's visual palette, node hierarchy, physics properties, and narrative context to emit production-ready, zero-dependency procedural audio. Writes audio files to disk (declared `fs` side effect).

## 4-Layer Sound Architecture

- **Layer 1 — Ambient Soundscape**: Sub-bass fundamentals (35–110 Hz), continuous room tones/wind, multi-band colored noise.
- **Layer 2 — Generative Music**: Neo-Riemannian Tonnetz & modal progressions, 5-tier adaptive stems modulated by Dynamic Tension Index (DTI), 1/f Voss-McCartney pink-noise melody + Euclidean grooves.
- **Layer 3 — Interactive Foley & SFX**: Kinetic physics impacts (E=½mv²), dual-sine/chirp UI clicks, physical modeling (Karplus-Strong).
- **Layer 4 — Spatial Acoustics**: ITD/ILD/pinna-notch binaural spatializer, distance attenuation (1/r) + ISO 9613-1 air absorption, raycast occlusion filter (20 kHz → 350 Hz).

## One-Shot Audiographer Protocol

1. **Scene ingestion**: Scan `.tscn`/DOM/Unity YAML for `BaseButton` (UI), `RigidBody2D/3D` + `CollisionShape` (impacts), `TileMapLayer` (footsteps), `GPUParticles` (weather), `Camera`/`AudioListener3D`.
2. **Synesthetic mapping**: Compute Valence-Arousal-Dominance (VAD) vector; extract dominant hex codes → map to mode/key, synthesis model, filter cutoff/resonance, spatial effects.
3. **DTI derivation**: Real-time tension `DTI ∈ [0,1]` from HP ratio, threat count, target distance, time remaining.
4. **Code emission**: Emit complete zero-dependency engine script (Godot `AudioStreamGenerator`, Web Audio `AudioWorklet`, or Python baker).
5. **Signal binding**: Connect `pressed`/`mouse_entered`/`body_entered` directly to synthesis functions; hot reload.

## Core DSP Synthesis Patterns

- **Karplus-Strong**: Delay `D = f_s/f_0 − 0.5`; loop filter with decay `10^(−3/(f_0·T_60))`.
- **Modal synthesis**: Per-mode damped sinusoid `y_k[n] = 2e^(−αT)cos(ω_d T)y_k[n−1] − e^(−2αT)y_k[n−2] + a_k sin(ω_d T)x[n]`. Material mode ratios: cast iron `[1,1.414,2.14,2.76,3.82,5.12]`, hardwood `[1,1.84,2.72,3.91]`, glass `[1,2.32,4.15,6.47,9.28]`, granite `[1,1.29,1.74,2.21,2.94]`.
- **PolyBLEP anti-aliased oscillators**: Apply residual correction `R(t,dt)` at phase discontinuities for saw/square.
- **Moog Ladder 4-pole TPT**: `g = tan(πf_c/f_s)`, `G = g/(1+g)`, tanh feedback for self-oscillation.
- **Minnaert bubble**: `f_0 ≈ 3.0/R` (meters); decaying chirp for raindrops/liquid.

## Humanization Laws (prevent robotic sterility)

- **1/f micro-timing jitter**: Voss-McCartney Gaussian offsets (σ ≈ 2.5–6.0 ms); never flat random.
- **Velocity-dependent swing**: `S(v) = 0.54 + 0.14·(v/127)`.
- **Velocity-to-filter tracking**: `f_cutoff(v) = f_base · 2^((v/127)·N_octaves)`.
- **Anti-machine-gun**: Never repeat exact params consecutively — gain trim ΔG ~ N(0,0.35 dB), cutoff jitter, attack micro-warp.
- **Thermal VCO drift**: ±3 cents @ 0.02 Hz; wow ±1.5 cents @ 0.4 Hz; delayed sigmoidal vibrato.
- **Triode warmth**: Asymmetric soft-clip `y = (x + 0.25x²)/(1 + 0.4|x|)` for 2f/3f harmonics.
- **ISO 226 loudness compensation**: Dynamic low-shelf boost <120 Hz at low master volume.

## Verification Checklist

- No static repetition on consecutive triggers (micro-spectral + timing variance).
- Dynamic loudness balance (bass compensated at low levels).
- Acoustic fusion (partials share synchronous modulation/pitch drift).
- Headroom/limiting (soft-clipper prevents hard wrap-around).
- Zero buffer underruns (non-blocking/allocation-free feed loops).
- Spatial consistency (occluded sounds low-pass smoothly behind geometry).

## Anti-Patterns

- **Flat random timing**: White noise for event scheduling — sounds mechanical. Use 1/f pink noise.
- **Exact param repetition**: Same gain/cutoff/attack on consecutive triggers (machine-gun effect). Perturb every event.
- **CPU readback in audio thread**: `buffer_get_data` or allocations in the fill loop → underruns. Pre-compute, lock-free SPSC.
- **Static samples**: Loading pre-recorded WAV for SFX defeats the zero-dependency contract. Synthesize from DSP.
- **Linear damping**: `speed *= (1 − drag)` is frame-dependent. Use exponential form.
