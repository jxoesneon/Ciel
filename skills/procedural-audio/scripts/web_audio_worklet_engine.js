/**
 * web_audio_worklet_engine.js
 * Universal Autonomous Web Audio API & AudioWorklet Engine.
 * 100% Zero-Dependency Standalone Runtime for Web & Canvas games.
 */

export class WebProceduralAudioEngine {
  constructor() {
    this.ctx = null;
    this.masterGain = null;
    this.reverbConvolver = null;
    this.tensionIndex = 0.0;
    this.tempoBpm = 116.0;
    this.stepTimer = null;
    this.currentStep = 0;
    this.scaleIntervals = [0, 2, 3, 5, 7, 9, 10]; // Dorian
    this.rootMidi = 50; // D3
    this.currentChordRoot = 50;
    this.isRunning = false;
  }

  async init() {
    const AudioContextClass = window.AudioContext || window.webkitAudioContext;
    this.ctx = new AudioContextClass({ latencyHint: 'interactive', sampleRate: 48000 });

    // Master Compressor / Limiter
    const limiter = this.ctx.createDynamicsCompressor();
    limiter.threshold.setValueAtTime(-1.0, this.ctx.currentTime);
    limiter.ratio.setValueAtTime(20.0, this.ctx.currentTime);
    limiter.connect(this.ctx.destination);

    this.masterGain = this.ctx.createGain();
    this.masterGain.gain.setValueAtTime(0.85, this.ctx.currentTime);
    this.masterGain.connect(limiter);

    // Procedural Convolver Reverb
    this.reverbConvolver = this.ctx.createConvolver();
    this.reverbConvolver.buffer = this.generateImpulseResponse(2.2, 2.0);
    const reverbGain = this.ctx.createGain();
    reverbGain.gain.setValueAtTime(0.25, this.ctx.currentTime);
    this.reverbConvolver.connect(reverbGain);
    reverbGain.connect(this.masterGain);

    this.startAmbienceDrone();
    this.startMusicScheduler();
    this.autoWireDom();
    this.isRunning = true;
    console.log("[WebProceduralAudioEngine] Engine initialized and DOM wired.");
  }

  generateImpulseResponse(durationSec, decay) {
    const rate = this.ctx.sampleRate;
    const len = Math.floor(rate * durationSec);
    const buf = this.ctx.createBuffer(2, len, rate);
    const L = buf.getChannelData(0);
    const R = buf.getChannelData(1);

    for (let i = 0; i < len; i++) {
      const t = i / rate;
      const env = Math.exp(-decay * t) * Math.exp(-4.0 * t);
      L[i] = (Math.random() * 2 - 1) * env;
      R[i] = (Math.random() * 2 - 1) * env;
    }
    return buf;
  }

  startAmbienceDrone() {
    const osc1 = this.ctx.createOscillator();
    const osc2 = this.ctx.createOscillator();
    const filter = this.ctx.createBiquadFilter();
    const gain = this.ctx.createGain();

    osc1.type = 'sine';
    osc1.frequency.setValueAtTime(55.0, this.ctx.currentTime);

    osc2.type = 'triangle';
    osc2.frequency.setValueAtTime(110.0, this.ctx.currentTime);

    filter.type = 'lowpass';
    filter.frequency.setValueAtTime(450.0, this.ctx.currentTime);

    gain.gain.setValueAtTime(0.18, this.ctx.currentTime);

    osc1.connect(filter);
    osc2.connect(filter);
    filter.connect(gain);
    gain.connect(this.masterGain);

    osc1.start();
    osc2.start();
  }

  startMusicScheduler() {
    const stepDurationMs = (60.0 / (this.tempoBpm * 4.0)) * 1000.0;
    this.stepTimer = setInterval(() => {
      this.currentStep = (this.currentStep + 1) % 16;
      if (this.currentStep === 0) {
        const roots = [50, 53, 48, 45];
        this.currentChordRoot = roots[Math.floor(Math.random() * roots.length)];
      }

      if (this.currentStep % 2 === 0 || this.tensionIndex > 0.4) {
        const noteOffset = this.scaleIntervals[this.currentStep % this.scaleIntervals.length];
        const midi = this.currentChordRoot + noteOffset + (this.tensionIndex > 0.7 ? 12 : 0);
        this.triggerArpNote(midi);
      }
    }, stepDurationMs);
  }

  triggerArpNote(midiNote) {
    const freq = 440.0 * Math.pow(2.0, (midiNote - 69) / 12.0);
    const osc = this.ctx.createOscillator();
    const gain = this.ctx.createGain();
    const filter = this.ctx.createBiquadFilter();

    osc.type = 'sawtooth';
    osc.frequency.setValueAtTime(freq, this.ctx.currentTime);

    filter.type = 'lowpass';
    filter.frequency.setValueAtTime(2400.0, this.ctx.currentTime);
    filter.Q.setValueAtTime(3.0, this.ctx.currentTime);

    const now = this.ctx.currentTime;
    gain.gain.setValueAtTime(0.001, now);
    gain.gain.exponentialRampToValueAtTime(0.15, now + 0.005);
    gain.gain.exponentialRampToValueAtTime(0.0001, now + 0.35);

    osc.connect(filter);
    filter.connect(gain);
    gain.connect(this.masterGain);
    gain.connect(this.reverbConvolver);

    osc.start(now);
    osc.stop(now + 0.38);
  }

  playUiClick() {
    if (!this.ctx) return;
    const now = this.ctx.currentTime;
    const osc1 = this.ctx.createOscillator();
    const osc2 = this.ctx.createOscillator();
    const gain = this.ctx.createGain();

    osc1.frequency.setValueAtTime(2400.0, now);
    osc2.frequency.setValueAtTime(4800.0, now);

    gain.gain.setValueAtTime(0.2, now);
    gain.gain.exponentialRampToValueAtTime(0.001, now + 0.035);

    osc1.connect(gain);
    osc2.connect(gain);
    gain.connect(this.masterGain);

    osc1.start(now);
    osc2.start(now);
    osc1.stop(now + 0.04);
    osc2.stop(now + 0.04);
  }

  autoWireDom() {
    document.querySelectorAll('button, input[type="button"], a[role="button"]').forEach((el) => {
      el.addEventListener('click', () => this.playUiClick());
    });
  }
}
