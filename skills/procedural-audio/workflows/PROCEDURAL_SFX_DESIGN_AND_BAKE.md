# WORKFLOW: PROCEDURAL SFX DESIGN & ASSET BAKING

**Execution Trigger**: `"bake sound effect"`, `"synthesize sfx"`, `"procedural sound design"`, `"generate wav"`  
**Target Output**: 24-bit PCM / 32-bit Float Broadcast WAV files (`.wav`)  
**Primary Goal**: Design, synthesize, and export studio-grade physical sound effects directly from mathematical formulas and physical modeling without recorded sample dependencies.

---

## 1. SUPPORTED SYNTHESIS RECIPES

| Category | SFX Type | Physical Engine / Math Model | CLI Command |
| :--- | :--- | :--- | :--- |
| **Ballistics** | Supersonic Bullet Crack | Whitham $N$-Wave + Mach Cone Timing | `python3 scripts/aaa_audio_generator.py --sfx bullet --out crack.wav` |
| **Vehicles** | Crossplane V8 Rev | 4-Stroke ICE Kinematics + Wiebe Pulses | `python3 scripts/aaa_audio_generator.py --sfx v8 --out v8_rev.wav` |
| **Bio-Acoustics** | Creature Mega-Fauna Roar | Kelly-Lochbaum Waveguide + Chaos Glottis | `python3 scripts/aaa_audio_generator.py --sfx creature --out roar.wav` |
| **Granular Weather** | Micro-Granular Visor Rain | Asynchronous Grain Clouds + Modal Visor | `python3 scripts/aaa_audio_generator.py --sfx rain --out rain.wav` |
| **UI Micro-SFX** | Crisp Glass / Cyber Click | PolyBLEP Dual Chirp + Exponential Decay | `python3 scripts/procedural_audio_generator.py --sfx click --out click.wav` |
| **Kinetic Impacts** | Solid Metal Collision | Modal Resonator Bank ($[1.0, 1.414, 2.14]$) | `python3 scripts/procedural_audio_generator.py --sfx metal --out metal.wav` |
| **Acoustic Spaces** | Concert Hall Impulse Response | Schroeder FDN Diffusion Matrix | `python3 scripts/procedural_audio_generator.py --sfx ir --out hall_ir.wav` |

---

## 2. STEP-BY-STEP BAKING PROCESS

### Step 1: Parameter Definition
Configure physical parameters in CLI or code:
- Sample Rate: $48000\text{ Hz}$ or $96000\text{ Hz}$.
- Bit Depth: 24-bit PCM or 32-bit Float.
- Material Properties: Density $\rho$, Young's Modulus $E$, Internal Damping $\alpha_k$.

### Step 2: Render & Normalization
Execute synthesis script:
```bash
python3 ~/.ciel/skills/procedural-audio/scripts/aaa_audio_generator.py --sfx bullet --format float32 --out /path/to/game/assets/audio/bullet_crack.wav
```

### Step 3: Post-Synthesis Verification Hook
Run `hooks/post_synthesize_hook.py` to audit:
```bash
python3 ~/.ciel/skills/procedural-audio/hooks/post_synthesize_hook.py /path/to/game/assets/audio/bullet_crack.wav
```
Ensures:
- Peak amplitude is constrained to $-1.0\text{ dBFS}$.
- Integrated loudness matches intended category.
- Zero DC offset.
