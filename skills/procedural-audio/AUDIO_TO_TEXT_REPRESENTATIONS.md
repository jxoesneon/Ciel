# AUDIO-TO-TEXT & MULTIMODAL AUDIO ANALYSIS FOR LLMS
**Theoretical Foundations, Diagnostic Audiograms for VLMs, and the 3-Tier `AudioStructuralManifest` Specification for Text-Only Models**

---

## 1. Executive Problem Formulation: Bridging the Modality Gap

Large Language Models (LLMs) operate over discrete token vocabularies. Unlike native multimodal speech models (e.g. Gemini 2.0/3.7, GPT-4o-Audio), the majority of state-of-the-art coding and reasoning LLMs (e.g., Claude 3.5 Sonnet, DeepSeek R1/V3, Llama 3.3, Qwen-Coder) are either **purely text-based** or **vision-language (text + image) models**.

To enable these models to analyze, critique, diagnose, compose, and synthesize audio with professional precision, we establish a **dual-channel bridge**:

```
                                  ┌───────────────────────────────┐
                                  │      RAW AUDIO WAVEFORM       │
                                  │   (.wav, .mp3, PCM buffers)   │
                                  └──────────────┬────────────────┘
                                                 │
                   ┌─────────────────────────────┴─────────────────────────────┐
                   ▼                                                           ▼
    ┌──────────────────────────────┐                            ┌──────────────────────────────┐
    │     FOR VISION-LANGUAGE      │                            │      FOR PURELY TEXT-ONLY    │
    │     MODELS (TEXT + IMAGE)    │                            │      MODELS (TEXT TOKENS)    │
    └──────────────┬───────────────┘                            └──────────────┬───────────────┘
                   │                                                           │
                   ▼                                                           ▼
    ┌──────────────────────────────┐                            ┌──────────────────────────────┐
    │  COMPOSITE VLM AUDIOGRAM     │                            │ 3-TIER HIERARCHICAL MANIFEST │
    │  - Waveform + RMS + Crest    │                            │  AudioStructuralManifest     │
    │  - Log-Mel Spectrogram (dB)  │                            │ - Tier 1: Semantic Context   │
    │  - 12-Bin Pitch Chromagram   │                            │ - Tier 2: Physical MIR DSP   │
    │  - Stereo Lissajous Goniom.  │                            │ - Tier 3: Symbolic ABC/MIDI  │
    │  - Structural SSM Matrix     │                            │ (< 1,200 Tokens JSON)        │
    └──────────────────────────────┘                            └──────────────────────────────┘
```

---

## 2. Channel A: VLM Diagnostic Audiograms (For Vision+Text Models)

Vision-Language Models process images through Vision Transformer (ViT) patch tokenization ($14\times14$ or $16\times16$ pixel grids). An optimized Audiogram must align with these patch boundaries to provide maximum diagnostic utility.

### 2.1 Multi-Panel Diagnostic Audiogram Architecture (300 DPI, Dark Mode `#0E1117`)

```
+──────────────────────────────────────────────────────────────────────────────────────────────────+
|                                    COMPOSITE DIAGNOSTIC AUDIOGRAM                                |
+──────────────────────────────────────────────────────────────────────────────────────────────────+
| PANEL 1: DUAL WAVEFORM ENVELOPE & RMS ENERGY                                                     |
| • True Peak Envelope (L/R) | 50ms Running RMS (dBFS) | Red Hazard Dots on Digital Clipping        |
+──────────────────────────────────────────────────────────────────────────────────────────────────+
| PANEL 2: LOG-FREQUENCY MEL-SPECTROGRAM [-80 dBFS to 0 dBFS] (Magma Dynamic Colormap)             |
| • N_FFT=2048, Hop=512, 128 Mel Bins | High-Contrast Grid Ticks (Hz to kHz)                        |
| • Labeled Acoustic Problem Zones: Mud (200-400Hz), Harshness (2-5kHz), Sibilance (6-8.5kHz)      |
+──────────────────────────────────────────────────────────────────────────────────────────────────+
| PANEL 3: 12-BIN PITCH CLASS CHROMAGRAM (Constant-Q Transform Approximation)                      |
| • Pitches [C, C#, D, D#, E, F, F#, G, G#, A, A#, B] over time for Chord Progressions & Dissonance|
+──────────────────────────────────────────────────────────────────────────────────────────────────+
| PANEL 4A: STEREO GONIOMETER (Mid vs Side)     │ PANEL 4B: SELF-SIMILARITY MATRIX (SSM)           |
| • Lissajous Phase Space & Phase Correlation   │ • Cosine Distance Recurrence Matrix for Macro    |
|   Coefficient r in [-1.0, +1.0] (Mono Check)  │   Song Structure (Intro, Verse, Chorus, Outro)   |
+──────────────────────────────────────────────────────────────────────────────────────────────────+
```

### 2.2 VLM Inspection Checklist:
1. **Frequency Masking & Buildup**: Inspect the blue-shaded $200\text{--}400\text{ Hz}$ band for continuous low-mid mud.
2. **Resonant Peaks & Harshness**: Inspect the pink-shaded $2\text{--}5\text{ kHz}$ and $6\text{--}8.5\text{ kHz}$ zones for piercing horizontal lines.
3. **Dynamics & Compression**: Inspect Crest Factor ($Peak - RMS$). A gap $<6\text{ dB}$ signals over-compression ("sausage master"); $>14\text{ dB}$ signals dynamic transient punch.
4. **Stereo Phase Integrity**: Inspect the Goniometer scatter. Horizontal spread or $r < 0$ flags destructive mono phase cancellation.

---

## 3. Channel B: The 3-Tier `AudioStructuralManifest` (For Text-Only Models)

To allow a text-only model to reason about audio, the physical signal is serialized into an ultra-compact (< 1,200 tokens), standardized JSON schema covering high-level semantics down to exact millisecond DSP moments.

```
+──────────────────────────────────────────────────────────────────────────────────────────────────+
|                             3-TIER AUDIO STRUCTURAL MANIFEST ARCHITECTURE                        |
+──────────────────────────────────────────────────────────────────────────────────────────────────+
| TIER 1: SEMANTIC CONTEXT & ENVIRONMENTAL ACOUSTICS (High-Level Perceptual Layer)                 |
| • Dense Descriptive Caption (AudioCaps / Clotho style)                                           |
| • AudioSet Ontological Classification Tags with confidence scores                                |
| • Room Acoustics: Estimated RT60 (reverberation time in seconds), DRR (dB), & Space Character    |
+──────────────────────────────────────────────────────────────────────────────────────────────────+
| TIER 2: PHYSICAL MIR SPECTRAL & DSP VECTORS (Low-Level Engineering Precision)                    |
| • Spectral Moments: Centroid (Brightness in Hz), Spread, Flatness (SFM), Rolloff 85%, HNR (dB)   |
| • Dynamic Loudness: ITU-R BS.1770-4 EBU R128 Integrated LUFS, Loudness Range (LU), True Peak     |
| • Temporal Dynamics: Unicode ASCII Sparklines [ ▃▅▇██▇▆▅▄▃ ], Attack Time (ms), Crest Factor dB  |
| • Harmonicity & Pitch: Dominant F0 (Hz & Note+Cents e.g. D4+08c), Vibrato Rate/Depth, Drift Slope|
| • Rhythmic Grid: BPM, Meter (4/4), Swing Ratio % (59.5%), Groove Microtiming Jitter (±4.8ms)     |
+──────────────────────────────────────────────────────────────────────────────────────────────────+
| TIER 3: SYMBOLIC MUSICAL TRANSCRIPTION (Compositional & Structural Layer)                        |
| • Detected Key & Mode                                                                            |
| • Compact ABC Notation (1.2 tokens/note, multi-voice headers [V:1], [V:2])                       |
| • Quantized MIDI Event Matrix (Onsets, MIDI Pitches, Velocities, Durations)                      |
| • Microtonal Accidentals (_1/2E, ^1/2F, exact cents offset tags)                                 |
+──────────────────────────────────────────────────────────────────────────────────────────────────+
```

---

## 4. Extraction Engines & Production Tools

| Tool / Script | Supported Platforms | Purpose |
| :--- | :--- | :--- |
| [`scripts/audiogram_generator.py`](file:///Users/meilynlopezcubero/.ciel/skills/procedural-audio/scripts/audiogram_generator.py) | Python (NumPy, SciPy, Matplotlib) | Generates high-contrast 300 DPI composite Audiogram PNGs for multimodal Vision LLMs. |
| [`scripts/audio_manifest_extractor.py`](file:///Users/meilynlopezcubero/.ciel/skills/procedural-audio/scripts/audio_manifest_extractor.py) | Pure Python Standard Library | Zero-dependency extractor converting any `.wav` into the full 3-Tier JSON manifest for text LLMs. |

---

## 5. End-to-End LLM Prompting Protocol for Text-Only Models

When passing audio analysis data to a text-only model (e.g. Claude 3.5, GPT-4, Llama 3), use the following system prompt envelope:

```markdown
[SYSTEM CONTEXT: DETERMINISTIC ACOUSTIC REASONING]
You are analyzing an audio clip via its calibrated 3-Tier AudioStructuralManifest JSON:

```json
{
  "audio_metadata": { "duration_seconds": 1.15, "sample_rate": 44100, "channels": 1 },
  "tier_1_semantic_context": {
    "dense_caption": "A resonant mechanical bullet crack with an explosive Mach cone shockwave decaying into a dry room.",
    "audioset_ontology_tags": [{"label": "Gunshot / Explosion", "confidence": 0.95}],
    "environmental_acoustics": { "estimated_rt60_seconds": 0.35, "acoustic_space": "Dry Outdoor Open Field" }
  },
  "tier_2_mir_physical_dsp": {
    "loudness_and_dynamics": { "integrated_lufs": -18.2, "true_peak_dbtp": -1.0, "crest_factor_db": 19.8, "loudness_contour_sparkline_lufs": " █▇▅▃▂      " },
    "spectral_timbre": { "spectral_centroid_hz": 3450.0, "spectral_flatness": 0.18, "attack_time_ms": 2.5, "semantic_tags": ["Bright / Crisp", "Impulsive Transient Attack"] },
    "pitch_profile": { "modality": "percussive_transient", "pitch_f0_median_hz": 0.0, "note_range": "Aperiodic" },
    "rhythmic_profile": { "estimated_bpm": 120.0, "meter": "4/4" }
  },
  "tier_3_symbolic_music": { "detected_key": "N/A", "abc_notation": "X:1\nT:Crack\n[V:1] z4 |]" }
}
```

[REASONING OBJECTIVE]:
1. Evaluate the physical materials and collision mechanics based on Tier 2 Centroid (3450 Hz) and Attack Time (2.5 ms).
2. Assess dynamic headroom and clipping safety from True Peak (-1.0 dBTP) and Crest Factor (19.8 dB).
3. Diagnose whether EQ adjustment, saturation, or reverb modification is required for mix integration.
```
