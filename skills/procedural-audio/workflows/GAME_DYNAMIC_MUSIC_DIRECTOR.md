# WORKFLOW: DYNAMIC ADAPTIVE GAME MUSIC DIRECTOR

**Execution Trigger**: `"create adaptive music"`, `"dynamic combat soundtrack"`, `"interactive music system"`, `"game score generator"`  
**Target Systems**: Godot 4.x, Unity C#, Web Audio API, C++17 Engines  
**Primary Goal**: Construct a multi-tier, real-time reactive musical soundtrack driven by in-game telemetry ($DTI$), Neo-Riemannian Tonnetz harmonic modulations, and metric Tihais.

---

## 1. DYNAMIC TENSION INDEX ($DTI$) INTEGRATION

The music director continuously samples in-game state to compute $DTI \in [0.0, 1.0]$:

$$DTI(t) = \text{clamp}\left( 0.35 \cdot \left(1 - \frac{HP}{HP_{\text{max}}}\right) + 0.30 \cdot \left(\frac{N_{\text{threats}}}{N_{\text{max}}}\right) + 0.20 \cdot \left(1 - \frac{d_{\text{target}}}{d_{\text{start}}}\right) + 0.15 \cdot \left(1 - \frac{t_{\text{rem}}}{t_{\text{tot}}}\right), 0.0, 1.0 \right)$$

### Dynamic Stem Activation Matrix:

```
+──────────────────────────────────────────────────────────────────────────────────────────────────────────+
|                                    DYNAMIC STEM ACTIVATION BY DTI TIER                                   |
+──────────────────────────────────────────────────────────────────────────────────────────────────────────+
| DTI RANGE   | INTENSITY LEVEL   | ACTIVE STEM LAYERS                                                     |
| [0.00-0.25] | Calm Exploration  | Layer 1 (Sub-Drone & Pad)                                              |
| [0.25-0.50] | Heightened Alert  | Layer 1 + Layer 2 (Arpeggiated Harps & Acoustic Pulses)                |
| [0.50-0.75] | Active Combat     | Layer 1 + Layer 2 + Layer 3 (Euclidean Percussion & Bass Riffs)        |
| [0.75-1.00] | Climax / Boss     | Layer 1 + Layer 2 + Layer 3 + Layer 4 (Distorted Leads & Metric Climax) |
+──────────────────────────────────────────────────────────────────────────────────────────────────────────+
```

---

## 2. HARMONIC MODULATION & TONNETZ GEODESIC NAVIGATION

When transitioning between game zones or tension states, compute the shortest path across the 24-triad torus using `scripts/neo_riemannian_tonnetz.py`:

```python
from scripts.neo_riemannian_tonnetz import TonnetzEngine, Triad

# Smooth harmonic modulation from Calm Exploration (C Major) to Boss Combat (Ab Minor)
start_chord = Triad(0, True)   # C Major
boss_chord = Triad(8, False)   # Ab Minor (Hexatonic Pole)

path = TonnetzEngine.find_shortest_path(start_chord, boss_chord)
# Returns: START(C) -> H(Abm) (Zero common tones, maximum shock)
```

---

## 3. COMBAT CADENCES VIA INDIAN TIHAI MATHEMATICAL SOLVER

During boss phase transitions or combat victories, resolve battle percussion using `scripts/indian_tala_engine.py`:

$$3P + 2D = N \cdot L + 1$$

```python
from scripts.indian_tala_engine import TalaEngine

engine = TalaEngine("tintal") # 16-beat cycle
tihai = engine.generate_tihai(target_cycle=1, preferred_dum=1.0)
# Returns phrase length = 5 beats, dum = 1 beat, landing exactly on Beat 1 (Sam)
```

---

## 4. CODE EMISSION TEMPLATE: DYNAMIC MUSIC STATE MACHINE

```gdscript
# AdaptiveMusicDirector.gd
extends Node

@export var tension_index: float = 0.0 # DTI [0.0 - 1.0]
@export var current_bpm: float = 128.0

var stem_drone_gain: float = 1.0
var stem_arp_gain: float = 0.0
var stem_drums_gain: float = 0.0
var stem_lead_gain: float = 0.0

func _process(delta: float) -> void:
    # Smooth dynamic stem crossfading
    stem_arp_gain = move_toward(stem_arp_gain, 1.0 if tension_index > 0.25 else 0.0, delta * 1.5)
    stem_drums_gain = move_toward(stem_drums_gain, 1.0 if tension_index > 0.50 else 0.0, delta * 2.0)
    stem_lead_gain = move_toward(stem_lead_gain, 1.0 if tension_index > 0.75 else 0.0, delta * 3.0)
```
