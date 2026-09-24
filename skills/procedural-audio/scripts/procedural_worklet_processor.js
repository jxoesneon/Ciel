/**
 * ProceduralWorkletProcessor
 * Runs on the dedicated high-priority Real-Time Audio Rendering Thread.
 * Zero dynamic object allocation during process() calls.
 */
class ProceduralWorkletProcessor extends AudioWorkletProcessor {
  constructor() {
    super();
    this.sampleRate = 48000;
    
    // Voice Allocation Pool (8 PolyBLEP Synth Voices)
    this.MAX_VOICES = 8;
    this.voices = [];
    for (let i = 0; i < this.MAX_VOICES; i++) {
      this.voices.push({
        active: false,
        phase: 0.0,
        phaseInc: 0.0,
        freq: 440.0,
        env: 0.0,
        decay: 0.9995,
        s1: 0.0,
        s2: 0.0,
        cutoff: 2400.0,
        resonance: 0.707
      });
    }

    // Ambient Drone Generators
    this.dronePhase1 = 0.0;
    this.dronePhase2 = 0.0;
    this.tension = 0.0;

    // Message Port Handler (Lock-free command intake)
    this.port.onmessage = (e) => {
      const data = e.data;
      if (data.type === 'NOTE_ON') {
        this.triggerVoice(data.freq, data.decay, data.cutoff);
      } else if (data.type === 'SET_TENSION') {
        this.tension = Math.max(0.0, Math.min(1.0, data.value));
      }
    };
  }

  triggerVoice(freq, decaySec, cutoffHz) {
    let voice = this.voices.find(v => !v.active);
    if (!voice) voice = this.voices[0]; // Simple voice steal

    voice.active = true;
    voice.freq = freq;
    voice.phaseInc = freq / this.sampleRate;
    voice.env = 1.0;
    voice.decay = Math.exp(-1.0 / (this.sampleRate * (decaySec || 0.35)));
    voice.cutoff = cutoffHz || 2200.0;
    voice.s1 = 0.0;
    voice.s2 = 0.0;
  }

  // Pure mathematical Anti-Aliased PolyBLEP Residual
  polyblep(t, dt) {
    if (t < dt) {
      const r = t / dt;
      return r + r - r * r - 1.0;
    } else if (t > 1.0 - dt) {
      const r = (t - 1.0) / dt;
      return r * r + r + r + 1.0;
    }
    return 0.0;
  }

  process(inputs, outputs, parameters) {
    const output = outputs[0];
    const left = output[0];
    const right = output[1];
    const blockSize = left.length; // Always 128 in Web Audio API

    for (let i = 0; i < blockSize; i++) {
      // 1. Ambient Sub-Bass Drone
      this.dronePhase1 += (6.283185307 * 55.0) / this.sampleRate;
      if (this.dronePhase1 > 6.283185307) this.dronePhase1 -= 6.283185307;

      const f2 = 110.0 + this.tension * 25.0;
      this.dronePhase2 += (6.283185307 * f2) / this.sampleRate;
      if (this.dronePhase2 > 6.283185307) this.dronePhase2 -= 6.283185307;

      let droneSample = (Math.sin(this.dronePhase1) * 0.2 + Math.sin(this.dronePhase2) * (0.05 + this.tension * 0.1));

      // 2. Synthesize Active Arp Voices with TPT SVF Filter
      let voiceSum = 0.0;
      for (let v = 0; v < this.MAX_VOICES; v++) {
        const voice = this.voices[v];
        if (!voice.active) continue;

        // PolyBLEP Saw
        voice.phase += voice.phaseInc;
        if (voice.phase >= 1.0) voice.phase -= 1.0;
        const rawSaw = 2.0 * voice.phase - 1.0;
        const saw = rawSaw - this.polyblep(voice.phase, voice.phaseInc);

        // TPT SVF Low-Pass
        const g = Math.tan((Math.PI * voice.cutoff) / this.sampleRate);
        const k = 1.0 / voice.resonance;
        const hp = (saw * voice.env - (2.0 * k + g) * voice.s1 - voice.s2) / (1.0 + 2.0 * k * g + g * g);
        const bp = g * hp + voice.s1;
        voice.s1 = g * hp + bp;
        const lp = g * bp + voice.s2;
        voice.s2 = g * bp + lp;

        voiceSum += lp * 0.25;

        // Decay Envelope & Anti-Denormal clamp
        voice.env *= voice.decay;
        if (voice.env < 1e-4) {
          voice.active = false;
          voice.env = 0.0;
        }
      }

      // Master Soft Clipper Saturation
      const mix = droneSample + voiceSum;
      const saturated = mix / (1.0 + 0.3 * Math.abs(mix));

      left[i] = saturated;
      right[i] = saturated;
    }

    return true; // Keep processor alive
  }
}

registerProcessor('procedural-worklet-processor', ProceduralWorkletProcessor);
