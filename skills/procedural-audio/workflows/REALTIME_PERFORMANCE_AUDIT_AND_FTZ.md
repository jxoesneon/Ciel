# WORKFLOW: REAL-TIME DSP PERFORMANCE AUDIT & FTZ/DAZ ENFORCEMENT

**Execution Trigger**: `"real-time audio audit"`, `"prevent audio glitches"`, `"fix audio cpu spike"`, `"lock-free audio"`  
**Primary Goal**: Eliminate audio thread dropouts, lock contention, subnormal float denormal traps, and garbage collection pauses.

---

## 1. REAL-TIME SAFETY RULES (THE AUDIO THREAD CONTRACT)

Inside the audio callback rendering loop, the code **MUST NEVER**:
1. Call `malloc()`, `free()`, `new`, `delete`, or allocate dynamic memory.
2. Acquire OS mutexes (`std::mutex`, `pthread_mutex_lock`), which cause priority inversion.
3. Perform I/O operations (file writing, network calls, console logging).
4. Allow IIR filter states to decay into subnormal float ranges without hardware FTZ/DAZ guards.

---

## 2. STEP-BY-STEP AUDITING PROCEDURE

### Step 1: Wrap Block Render Loops with Scoped FTZ/DAZ
In C++ audio engines, include `scripts/bulletproof_procedural_dsp.h`:
```cpp
#include "scripts/bulletproof_procedural_dsp.h"

void audio_render_callback(float* buffer, size_t num_frames) {
    // RAII guard automatically sets FTZ and DAZ on CPU registers (x86_64 MXCSR / ARM64 FPCR)
    BulletproofDSP::ScopedNoDenormals no_denormals;

    for (size_t i = 0; i < num_frames; ++i) {
        buffer[i] = render_dsp_sample();
    }
}
```

### Step 2: Thread Communication via Lock-Free SPSC Queues
Pass commands from the game/UI thread to the audio thread via cache-line aligned SPSC ring buffers:
```cpp
BulletproofDSP::SPSCQueue<AudioCommand, 1024> command_queue;

// Producer (Game Thread):
command_queue.push(AudioCommand{NOTE_ON, 440.0f});

// Consumer (Real-Time Audio Thread):
AudioCommand cmd;
while (command_queue.pop(cmd)) {
    handle_command(cmd);
}
```

### Step 3: Web AudioWorklet Zero-GC Deployment
In browser environments, use `scripts/procedural_worklet_processor.js` registered as an `AudioWorkletProcessor` with pre-allocated voice pools. Never create dynamic Web Audio nodes on note triggers.
