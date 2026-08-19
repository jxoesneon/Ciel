#!/usr/bin/env python3
"""
procedural_audio_generator.py
Comprehensive Zero-Dependency Procedural Audio Synthesizer & WAV Baker.
Supports 24-bit Linear PCM and 32-bit IEEE Float exports.
Author: Ciel / Antigravity Procedural Audio Intelligence
"""

import sys
import os
import math
import struct
import random
import json
import argparse
from typing import List, Tuple, Dict, Any, Optional

# --- WAV EXPORT ENGINES ---

def write_wav_32bit_float(filename: str, samples: List[Tuple[float, float]], sample_rate: int = 48000):
    """Encodes stereo float samples (-1.0 to 1.0) into IEEE 32-bit float WAV."""
    num_channels = 2
    bits_per_sample = 32
    byte_rate = sample_rate * num_channels * (bits_per_sample // 8)
    block_align = num_channels * (bits_per_sample // 8)
    data_size = len(samples) * block_align
    
    os.makedirs(os.path.dirname(os.path.abspath(filename)), exist_ok=True)
    with open(filename, 'wb') as f:
        # RIFF Header
        f.write(b'RIFF')
        f.write(struct.pack('<I', 36 + data_size))
        f.write(b'WAVE')
        
        # fmt chunk (AudioFormat = 3 for IEEE Float)
        f.write(b'fmt ')
        f.write(struct.pack('<I', 16))
        f.write(struct.pack('<H', 3))
        f.write(struct.pack('<H', num_channels))
        f.write(struct.pack('<I', sample_rate))
        f.write(struct.pack('<I', byte_rate))
        f.write(struct.pack('<H', block_align))
        f.write(struct.pack('<H', bits_per_sample))
        
        # data chunk
        f.write(b'data')
        f.write(struct.pack('<I', data_size))
        for left, right in samples:
            cl_l = max(-1.0, min(1.0, float(left)))
            cl_r = max(-1.0, min(1.0, float(right)))
            f.write(struct.pack('<ff', cl_l, cl_r))

def write_wav_24bit_pcm(filename: str, samples: List[Tuple[float, float]], sample_rate: int = 48000):
    """Encodes stereo float samples (-1.0 to 1.0) into 24-bit Linear PCM WAV."""
    num_channels = 2
    bits_per_sample = 24
    bytes_per_sample = 3
    block_align = num_channels * bytes_per_sample
    data_size = len(samples) * block_align
    
    os.makedirs(os.path.dirname(os.path.abspath(filename)), exist_ok=True)
    with open(filename, 'wb') as f:
        f.write(b'RIFF')
        f.write(struct.pack('<I', 36 + data_size))
        f.write(b'WAVE')
        
        f.write(b'fmt ')
        f.write(struct.pack('<I', 16))
        f.write(struct.pack('<H', 1)) # PCM = 1
        f.write(struct.pack('<H', num_channels))
        f.write(struct.pack('<I', sample_rate))
        f.write(struct.pack('<I', sample_rate * block_align))
        f.write(struct.pack('<H', block_align))
        f.write(struct.pack('<H', bits_per_sample))
        
        f.write(b'data')
        f.write(struct.pack('<I', data_size))
        max_int24 = 8388607 # 2^23 - 1
        for left, right in samples:
            val_l = int(max(-1.0, min(1.0, float(left))) * max_int24)
            val_r = int(max(-1.0, min(1.0, float(right))) * max_int24)
            f.write(val_l.to_bytes(3, byteorder='little', signed=True))
            f.write(val_r.to_bytes(3, byteorder='little', signed=True))

# --- DSP MATHEMATICAL PRIMITIVES ---

class PaulKelletPinkNoise:
    def __init__(self):
        self.b0 = self.b1 = self.b2 = self.b3 = self.b4 = self.b5 = self.b6 = 0.0

    def next_sample(self) -> float:
        white = (random.random() * 2.0) - 1.0
        self.b0 = 0.99886 * self.b0 + white * 0.0555179
        self.b1 = 0.99332 * self.b1 + white * 0.0750759
        self.b2 = 0.96900 * self.b2 + white * 0.1538520
        self.b3 = 0.86650 * self.b3 + white * 0.3104856
        self.b4 = 0.55000 * self.b4 + white * 0.5329522
        self.b5 = -0.7616 * self.b5 - white * 0.0168980
        pink = (self.b0 + self.b1 + self.b2 + self.b3 + self.b4 + self.b5 + self.b6 + white * 0.5362) * 0.11
        self.b6 = white * 0.115926
        return pink

class StateVariableFilter:
    def __init__(self, sample_rate: float = 48000.0):
        self.fs = sample_rate
        self.s1 = 0.0
        self.s2 = 0.0

    def process(self, x: float, cutoff_hz: float, q: float = 0.707) -> Tuple[float, float, float]:
        """Returns (lowpass, bandpass, highpass)"""
        cutoff_hz = max(10.0, min(0.48 * self.fs, cutoff_hz))
        q = max(0.1, q)
        g = math.tan(math.pi * cutoff_hz / self.fs)
        k = 1.0 / q
        hp = (x - (2.0 * k + g) * self.s1 - self.s2) / (1.0 + 2.0 * k * g + g * g)
        bp = g * hp + self.s1
        self.s1 = g * hp + bp
        lp = g * bp + self.s2
        self.s2 = g * bp + lp
        return (lp, bp, hp)

# --- PROCEDURAL SYNTHESIZERS ---

class ProceduralSoundSynthesizer:
    @staticmethod
    def synthesize_ui_click(sample_rate: int = 48000) -> List[Tuple[float, float]]:
        duration = 0.035
        num_samples = int(duration * sample_rate)
        out = []
        p1 = p2 = 0.0
        for i in range(num_samples):
            t = i / sample_rate
            env = (1.0 - (i / num_samples)) ** 1.5
            p1 += 2.0 * math.pi * 2400.0 / sample_rate
            p2 += 2.0 * math.pi * 4800.0 / sample_rate
            sample = (math.sin(p1) * 0.65 + math.sin(p2) * 0.35) * env * 0.4
            out.append((sample, sample))
        return out

    @staticmethod
    def synthesize_ui_whoosh(sample_rate: int = 48000) -> List[Tuple[float, float]]:
        duration = 0.22
        num_samples = int(duration * sample_rate)
        out = []
        svf = StateVariableFilter(sample_rate)
        pink = PaulKelletPinkNoise()
        for i in range(num_samples):
            t = i / sample_rate
            norm_t = t / duration
            # Bell curve cutoff sweep 300Hz -> 1900Hz -> 400Hz
            cutoff = 300.0 + 1600.0 * (math.sin(math.pi * norm_t) ** 2)
            noise = pink.next_sample()
            _, bp, _ = svf.process(noise, cutoff, q=3.2)
            env = math.sin(math.pi * norm_t) ** 2
            # Stereo pan left to right
            pan_angle = norm_t * (math.pi / 2.0)
            left = bp * env * math.cos(pan_angle) * 0.7
            right = bp * env * math.sin(pan_angle) * 0.7
            out.append((left, right))
        return out

    @staticmethod
    def synthesize_metal_impact(sample_rate: int = 48000, f0: float = 440.0) -> List[Tuple[float, float]]:
        duration = 2.2
        num_samples = int(duration * sample_rate)
        out = []
        mode_ratios = [1.0, 1.414, 2.142, 2.761, 3.824, 5.123]
        mode_gains  = [1.0, 0.75,  0.55,  0.40,  0.25,  0.15]
        mode_t60    = [2.0, 1.6,   1.2,   0.8,   0.45,  0.2]
        
        phases = [0.0] * len(mode_ratios)
        for i in range(num_samples):
            t = i / sample_rate
            sample = 0.0
            for k in range(len(mode_ratios)):
                freq = f0 * mode_ratios[k]
                if freq < 0.48 * sample_rate:
                    decay = math.exp(-3.0 * t / mode_t60[k])
                    phases[k] += 2.0 * math.pi * freq / sample_rate
                    sample += mode_gains[k] * decay * math.sin(phases[k])
            # Initial strike click
            if i < int(0.003 * sample_rate):
                sample += (random.random() * 2.0 - 1.0) * 0.6
            sat = math.tanh(sample * 1.5) * 0.75
            out.append((sat, sat))
        return out

    @staticmethod
    def synthesize_wood_thud(sample_rate: int = 48000, base_pitch: float = 110.0) -> List[Tuple[float, float]]:
        duration = 0.65
        num_samples = int(duration * sample_rate)
        out = []
        body_phase = 0.0
        mode_ratios = [1.0, 1.84, 2.72, 3.91]
        mode_gains  = [0.8, 0.5,  0.3,  0.15]
        mode_decay  = [0.08, 0.05, 0.025, 0.012]
        phases = [0.0] * len(mode_ratios)

        for i in range(num_samples):
            t = i / sample_rate
            pitch = 55.0 + (base_pitch * 2.0) * math.exp(-t / 0.012)
            body_phase += 2.0 * math.pi * pitch / sample_rate
            sample = math.sin(body_phase) * math.exp(-t / 0.045)
            for k in range(len(mode_ratios)):
                phases[k] += 2.0 * math.pi * (base_pitch * mode_ratios[k]) / sample_rate
                sample += mode_gains[k] * math.sin(phases[k]) * math.exp(-t / mode_decay[k])
            if i < int(0.004 * sample_rate):
                sample += (random.random() * 2.0 - 1.0) * 0.4
            sat = math.tanh(sample * 1.8) * 0.8
            out.append((sat, sat))
        return out

    @staticmethod
    def synthesize_laser_blaster(sample_rate: int = 48000) -> List[Tuple[float, float]]:
        duration = 0.35
        num_samples = int(duration * sample_rate)
        out = []
        phase = 0.0
        for i in range(num_samples):
            t = i / sample_rate
            env = math.exp(-8.0 * t)
            freq = 80.0 + 3120.0 * math.exp(-16.0 * t)
            phase += 2.0 * math.pi * freq / sample_rate
            # PolyBLEP Saw
            norm_phase = (phase / (2.0 * math.pi)) % 1.0
            saw = 2.0 * norm_phase - 1.0
            sample = math.tanh(saw * env * 3.0) * 0.75
            out.append((sample, sample * 0.95))
        return out

    @staticmethod
    def synthesize_explosion(sample_rate: int = 48000, duration: float = 2.8) -> List[Tuple[float, float]]:
        num_samples = int(duration * sample_rate)
        out = []
        sub_phase = 0.0
        svf = StateVariableFilter(sample_rate)
        for i in range(num_samples):
            t = i / sample_rate
            noise_env = math.exp(-3.2 * t)
            sub_env = math.exp(-1.5 * t) * (1.0 - math.exp(-18.0 * t))
            sub_freq = 30.0 + 45.0 * math.exp(-3.5 * t)
            sub_phase += 2.0 * math.pi * sub_freq / sample_rate
            sub = math.sin(sub_phase) * sub_env * 0.85
            white = (random.random() * 2.0) - 1.0
            cutoff = 150.0 + 3800.0 * math.exp(-5.5 * t)
            lp, _, _ = svf.process(white, cutoff, q=0.8)
            mixed = (lp * noise_env * 1.3) + sub
            sat = math.tanh(mixed * 1.6) * 0.85
            out.append((sat, sat))
        return out

    @staticmethod
    def synthesize_ambient_drone(sample_rate: int = 48000, duration: float = 8.0, root_hz: float = 55.0) -> List[Tuple[float, float]]:
        num_samples = int(duration * sample_rate)
        out = []
        p1 = p2 = p3 = p_lfo = 0.0
        pink = PaulKelletPinkNoise()
        svf = StateVariableFilter(sample_rate)
        for i in range(num_samples):
            t = i / sample_rate
            p_lfo += 2.0 * math.pi * 0.12 / sample_rate
            lfo = math.sin(p_lfo)
            f1 = root_hz + (lfo * 1.5)
            f2 = (root_hz * 1.5) + (math.cos(p_lfo * 0.7) * 0.8)
            f3 = (root_hz * 2.0)
            p1 += 2.0 * math.pi * f1 / sample_rate
            p2 += 2.0 * math.pi * f2 / sample_rate
            p3 += 2.0 * math.pi * f3 / sample_rate
            drone = (math.sin(p1) * 0.4 + math.sin(p2) * 0.25 + math.sin(p3) * 0.15)
            # Gentle room noise
            noise = pink.next_sample()
            lp_noise, _, _ = svf.process(noise, 450.0, q=0.707)
            # Subtle stereo warmth
            left = (drone + lp_noise * 0.08) * 0.7
            right = (drone + lp_noise * 0.075) * 0.7
            out.append((left, right))
        return out

    @staticmethod
    def synthesize_procedural_impulse_response(sample_rate: int = 48000, duration: float = 2.5, decay: float = 2.0) -> List[Tuple[float, float]]:
        num_samples = int(duration * sample_rate)
        out = []
        early_taps = [
            (int(sample_rate * 0.012), 0.7),
            (int(sample_rate * 0.024), 0.5),
            (int(sample_rate * 0.038), 0.35),
            (int(sample_rate * 0.055), 0.25)
        ]
        left_raw = [0.0] * num_samples
        right_raw = [0.0] * num_samples

        for i in range(num_samples):
            t = i / sample_rate
            env = math.exp(-decay * t)
            damp = math.exp(-4.5 * t)
            # Gaussian distributed noise via Box-Muller
            u1 = max(1e-12, random.random())
            u2 = max(1e-12, random.random())
            n_l = math.sqrt(-2.0 * math.log(u1)) * math.cos(2.0 * math.pi * u2)
            n_r = math.sqrt(-2.0 * math.log(u1)) * math.sin(2.0 * math.pi * u2)
            left_raw[i] = n_l * env * damp * 0.5
            right_raw[i] = n_r * env * damp * 0.5

        for delay_idx, gain in early_taps:
            if delay_idx < num_samples:
                left_raw[delay_idx] += gain * 0.4
            if delay_idx + 64 < num_samples:
                right_raw[delay_idx + 64] += gain * 0.38

        # Normalize
        max_val = max(1e-6, max(max(abs(x) for x in left_raw), max(abs(x) for x in right_raw)))
        for i in range(num_samples):
            out.append((left_raw[i] / max_val * 0.95, right_raw[i] / max_val * 0.95))
        return out

# --- CLI DISPATCHER ---

def main():
    parser = argparse.ArgumentParser(description="Procedural Audio Generator CLI")
    parser.add_argument("--sfx", choices=["click", "whoosh", "metal", "wood", "laser", "explosion", "drone", "ir"], default="click", help="Sound effect type to synthesize")
    parser.add_argument("--format", choices=["float32", "pcm24"], default="float32", help="WAV encoding format")
    parser.add_argument("--out", type=str, default="output.wav", help="Output file path")
    parser.add_argument("--sr", type=int, default=48000, help="Sample rate in Hz")
    parser.add_argument("--duration", type=float, default=2.0, help="Duration in seconds (for drone/explosion/ir)")
    args = parser.parse_args()

    print(f"[ProceduralAudioGenerator] Synthesizing '{args.sfx}' at {args.sr}Hz ({args.format})...")
    synth = ProceduralSoundSynthesizer()

    if args.sfx == "click":
        samples = synth.synthesize_ui_click(args.sr)
    elif args.sfx == "whoosh":
        samples = synth.synthesize_ui_whoosh(args.sr)
    elif args.sfx == "metal":
        samples = synth.synthesize_metal_impact(args.sr)
    elif args.sfx == "wood":
        samples = synth.synthesize_wood_thud(args.sr)
    elif args.sfx == "laser":
        samples = synth.synthesize_laser_blaster(args.sr)
    elif args.sfx == "explosion":
        samples = synth.synthesize_explosion(args.sr, duration=args.duration)
    elif args.sfx == "drone":
        samples = synth.synthesize_ambient_drone(args.sr, duration=args.duration)
    elif args.sfx == "ir":
        samples = synth.synthesize_procedural_impulse_response(args.sr, duration=args.duration)
    else:
        samples = synth.synthesize_ui_click(args.sr)

    if args.format == "float32":
        write_wav_32bit_float(args.out, samples, args.sr)
    else:
        write_wav_24bit_pcm(args.out, samples, args.sr)

    print(f"[ProceduralAudioGenerator] Successfully baked {len(samples)} frames to {args.out}")

if __name__ == "__main__":
    main()
