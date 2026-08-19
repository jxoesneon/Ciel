# WORKFLOW: ONE-SHOT SCENE PROCEDURAL AUDIOBLUEPRINT

**Execution Trigger**: `"create the audio for this screen"`, `"generate audio for this scene"`, `"add procedural sound to this level"`  
**Target Runtimes**: Godot 4.x (GDScript), Web Audio API (Vanilla JS / AudioWorklet / React), Unity (C#)  
**Primary Goal**: Autonomously inspect visual aesthetics, scene graph hierarchies, collision physics, and narrative mood to synthesize a zero-dependency, production-grade, 4-layer procedural audio system.

---

## 1. WORKFLOW PHASES & DECISION TREE

```
                                [MINIMALIST USER PROMPT: "create the audio for this screen"]
                                                             │
                                                             ▼
                                                [PHASE 1: AST INGESTION]
                                                - Scan Node Tree / DOM / Unity Scene
                                                - Extract Colliders, UI Controls, Particles
                                                             │
                                                             ▼
                                             [PHASE 2: SYNESTHESIA MAPPING]
                                             - CIELAB Dominant Hue -> Modal Scale
                                             - Lux Illumination -> Filter Cutoff (fc)
                                             - Contrast / Saturation -> FM Index & Saturation
                                                             │
                                                             ▼
                                              [PHASE 3: DTI CALCULATION]
                                              - Derive Dynamic Tension Index (0.0 to 1.0)
                                              - Map Health, Threat Proximity, Objectives
                                                             │
                                                             ▼
                                             [PHASE 4: 4-LAYER SYNTHESIS EMISSION]
                                             - Layer 1: Ambient Soundscape Drone
                                             - Layer 2: Adaptive Dynamic Music
                                             - Layer 3: Interactive Kinetic SFX & UI Clicks
                                             - Layer 4: Spatial Acoustics & Occlusion
                                                             │
                                                             ▼
                                              [PHASE 5: AUTO-WIRING & VERIFICATION]
                                              - Connect button signals & body_entered
                                              - Run Post-Synthesize Audio QC (LUFS, Peaks)
```

---

## 2. STEP-BY-STEP EXECUTION RUNBOOK

### Step 1: Automated AST Inspection & Entity Classification
Execute `scripts/scene_context_analyzer.py` or inspect target file:
- **UI Elements**: Locate all `BaseButton`, `<button>`, `<a>`, `<input>`. Target for Layer 3 UI micro-chirp synthesis.
- **Physics RigidBodies**: Locate `RigidBody2D/3D`, colliders, mass $m$, dynamic friction $\mu$, and bounce $e$. Target for Modal kinetic impact synthesis.
- **Particle Emitters**: Locate `GPUParticles2D/3D`, Canvas particles, Three.js `Points`. Target for granular weather/fluid synthesis.
- **Environment & Lighting**: Locate `WorldEnvironment`, `DirectionalLight3D`, CSS background colors.

### Step 2: Continuous Synesthesia & Affective Vector Derivation
Calculate mathematical transfer values:
1. **Hue Angle $\theta_H \to$ Harmonic Mode**:
   $$\text{Mode Index} = \text{round}\left(\frac{\theta_H}{360^\circ} \times 7\right) \pmod 7$$
2. **Scene Illumination (Lux) $\to$ Filter Cutoff Frequency $f_c$**:
   $$f_c(E_v) = 350 \cdot \left(\frac{18500}{350}\right)^{\text{clamp}\left(\frac{\log_{10}(E_v) - \log_{10}(0.1)}{4.0}, 0.0, 1.0\right)} \quad (\text{Hz})$$
3. **Room Dimensions $\to$ Eyring Reverberation Time $T_{60}$**:
   $$T_{60} = \frac{0.161 \cdot V}{-S_{\text{tot}} \ln(1 - \bar{\alpha}) + 4 m V} \quad (\text{seconds})$$

### Step 3: Emit Platform-Specific Procedural Engine Code
- For **Godot 4.x**: Emit a self-contained `Node` extending `AudioStreamGeneratorPlayback` utilizing `push_buffer(PackedVector2Array)` and pre-allocated voice pools.
- For **Web Applications**: Emit an `AudioWorkletNode` loading `ProceduralWorkletProcessor` with lock-free parameter message porting.
- For **Standalone Assets**: Bake 24-bit/32-bit float WAVs via `scripts/procedural_audio_generator.py` or `scripts/aaa_audio_generator.py`.

### Step 4: Automatic Signal Wiring & Hot Reload
Connect interactive events in code:
```gdscript
# Godot 4.x Signal Wiring
func _wire_scene(node: Node) -> void:
    if node is BaseButton:
        node.pressed.connect(func(): play_ui_click())
    elif node is RigidBody2D:
        node.contact_monitor = true
        node.max_contacts_reported = 4
        node.body_entered.connect(func(_b): play_modal_impact(node.linear_velocity.length(), node.mass))
    for child in node.get_children():
        _wire_scene(child)
```

### Step 5: Post-Synthesize Audio Verification (QC)
Run `hooks/post_synthesize_hook.py` to verify:
- Integrated Loudness meets game standards ($-16\text{ LUFS} \pm 1.0\text{ LUFS}$).
- True Peak $< -1.0\text{ dBFS}$ (zero digital clipping).
- DC offset $< 0.001$.
- Zero NaN/Infinity float values.

---

## 3. VERIFICATION & SUCCESS CRITERIA
- [ ] The scene produces a rich, evolving ambient soundscape upon loading with zero user interaction.
- [ ] Interacting with UI controls triggers crisp, responsive, micro-spectral varied clicks.
- [ ] Physics collisions trigger mass-proportional resonant impacts without machine-gun repetition.
- [ ] CPU overhead remains $< 2.5\%$ on a single core with zero subnormal denormal spikes.
