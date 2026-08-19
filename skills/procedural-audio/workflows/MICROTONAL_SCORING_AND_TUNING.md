# WORKFLOW: GLOBAL MICROTONAL SCORING & TUNING

**Execution Trigger**: `"compose in microtonal"`, `"arabic maqam"`, `"indian raga 22 shrutis"`, `"gamelan slendro"`, `"31-EDO"`, `"53-EDO"`  
**Primary Goal**: Author authentic non-12-TET melodies, continuous microtonal gamakas/meend, and non-Western harmonic soundscapes.

---

## 1. SUPPORTED MICROTONAL SYSTEMS

1. **Indian Classical 22 Shrutis**: 5-limit Just Intonation ratios with continuous cubic Hermite *Meend* portamento.
2. **Arabic 24-EDO Maqamat**: Quarter-tone *Ajnas* building blocks (Rast, Bayati, Hijaz, Sikah, Saba) following traditional *Sayr* trajectory paths.
3. **Indonesian Gamelan**: Inharmonic bronze metallophone stretch scales ($1215\text{ cents/octave}$) with paired *Ombak* acoustic beating ($4.5\text{--}7.5\text{ Hz}$).
4. **31-EDO & 53-EDO Systems**: Pure meantone thirds and Turkish classical tuning.

---

## 2. PYTHON WORKFLOW EXAMPLE

```python
from scripts.microtonal_pitch_engine import MicrotonalPitchEngine

engine = MicrotonalPitchEngine(concert_a=440.0)

# 1. Indian Raga Bhoopali with 22-Shruti Ratios (Sa, Re, Ga, Pa, Dha)
raga_notes = [
    engine.get_shruti_freq(261.63, 0),  # Sa (1/1)
    engine.get_shruti_freq(261.63, 4),  # Shuddha Re (9/8 = 203.9c)
    engine.get_shruti_freq(261.63, 7),  # Shuddha Ga (5/4 = 386.3c)
    engine.get_shruti_freq(261.63, 13), # Pa (3/2 = 702.0c)
    engine.get_shruti_freq(261.63, 16)  # Tivra Dha (5/3 = 884.4c)
]

# 2. Continuous Meend Glide between Re and Ga over 100ms
time_steps = 100
glide_freqs = [engine.continuous_meend_interpolation(raga_notes[1], raga_notes[2], t / 100.0) for t in range(time_steps)]
```

---

## 3. REAL-TIME RETUNING ENGINE (HERMODE TUNING)
For orchestral and synthesizer pads, deploy `scripts/advanced_humanizer.py` to dynamically adjust chord thirds by $-14\text{ cents}$ and fifths by $+2\text{ cents}$ in real time, eliminating all 12-TET sensory roughness.
