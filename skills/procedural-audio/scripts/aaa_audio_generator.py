#!/usr/bin/env python3
"""
aaa_audio_generator.py
Standalone AAA Procedural Audio Generator & WAV Baker (v3.0.0)
Synthesizes physical acoustics, vehicle engines, creature vocal tracts, 
and ballistic shockwaves purely from mathematical code.

Zero external dependencies (uses standard library: math, struct, wave, random, argparse).
"""

import argparse
import math
import random
import struct
import wave

SAMPLE_RATE = 44100
SOUND_SPEED = 343.0 # m/s

def write_wav(filename: str, samples: list[float], sample_rate: int = SAMPLE_RATE, bit_depth: int = 16):
    with wave.open(filename, 'wb') as wav:
        wav.setnchannels(1)
        wav.setsampwidth(bit_depth // 8)
        wav.setframerate(sample_rate)
        
        packed = bytearray()
        for s in samples:
            # Soft clip
            sat = (s + 0.25 * s * s) / (1.0 + 0.4 * abs(s)) if abs(s) > 0.001 else s
            if bit_depth == 16:
                int_val = max(-32767, min(32767, int(sat * 32760.0)))
                packed.extend(struct.pack('<h', int_val))
            elif bit_depth == 24:
                int_val = max(-8388607, min(8388607, int(sat * 8388600.0)))
                b = int_val.to_bytes(3, byteorder='little', signed=True)
                packed.extend(b)
        wav.writeframes(packed)
    print(f'[OK] Baked {filename} ({len(samples)/sample_rate:.2f}s, {bit_depth}-bit)')

# =============================================================================
# 1. DICE SUPERSONIC BULLET CRACK & MUZZLE BLAST
# =============================================================================
def bake_supersonic_bullet(distance_m: float = 80.0, miss_dist_m: float = 1.2, speed_mps: float = 850.0) -> list[float]:
    M = speed_mps / SOUND_SPEED
    theta_m = math.asin(1.0 / M)
    
    # Times of arrival
    x_travel = distance_m - miss_dist_m / math.tan(theta_m)
    t_crack = (x_travel / speed_mps) + (miss_dist_m / (math.cos(theta_m) * SOUND_SPEED))
    t_muzzle = distance_m / SOUND_SPEED
    
    total_dur = t_muzzle + 0.8
    num_samples = int(total_dur * SAMPLE_RATE)
    samples = [0.0] * num_samples
    
    # 1. Whitham N-Wave Crack
    n_crack_start = int(t_crack * SAMPLE_RATE)
    n_wave_len = int(0.00045 * SAMPLE_RATE) # 450 us
    crack_peak = 1.0 / math.pow(max(0.2, miss_dist_m), 0.75)
    
    for i in range(n_wave_len):
        idx = n_crack_start + i
        if idx < num_samples:
            t_norm = i / n_wave_len
            n_val = (1.0 - 2.0 * t_norm) * crack_peak
            win = math.sin(math.pi * t_norm)
            samples[idx] += n_val * win * 0.9
            
    # 2. Subsonic Muzzle Boom (Low-frequency thump with environmental echo tail)
    n_muzzle_start = int(t_muzzle * SAMPLE_RATE)
    muzzle_dur = int(0.6 * SAMPLE_RATE)
    for i in range(muzzle_dur):
        idx = n_muzzle_start + i
        if idx < num_samples:
            t = i / SAMPLE_RATE
            env = math.exp(-t * 12.0)
            boom = math.sin(2.0 * math.pi * 65.0 * t * math.exp(-t * 8.0)) * 0.7
            rumble = (random.random() * 2.0 - 1.0) * math.exp(-t * 6.0) * 0.3
            samples[idx] += (boom + rumble) * env * (1.0 / math.sqrt(distance_m * 0.1))
            
    return samples

# =============================================================================
# 2. FORZA / GT7 PHYSICAL V8 COMBUSTION ENGINE REV
# =============================================================================
def bake_v8_engine_rev(duration: float = 3.5) -> list[float]:
    num_samples = int(duration * SAMPLE_RATE)
    samples = [0.0] * num_samples
    
    # Crossplane V8 firing angles (radians)
    firing_offsets = [0.0, math.pi*0.5, math.pi*1.5, math.pi, math.pi*2.5, math.pi*2.0, math.pi*3.5, math.pi*3.0]
    crank_angle = 0.0
    lpf_state = 0.0
    
    for n in range(num_samples):
        t = n / SAMPLE_RATE
        # RPM trajectory: Idle 900 -> 6500 RPM rev -> 1200 RPM settle
        if t < 0.5:
            rpm = 900.0
            throttle = 0.05
        elif t < 2.0:
            rpm = 900.0 + (6500.0 - 900.0) * math.sin((t - 0.5) / 1.5 * (math.pi / 2.0))
            throttle = 0.95
        else:
            rpm = 1200.0 + (6500.0 - 1200.0) * math.exp(-(t - 2.0) * 2.5)
            throttle = 0.02
            
        rad_per_sec = (rpm / 60.0) * (2.0 * math.pi)
        crank_angle += rad_per_sec / SAMPLE_RATE
        if crank_angle >= 4.0 * math.pi:
            crank_angle -= 4.0 * math.pi
            
        combustion_sum = 0.0
        for phi_offset in firing_offsets:
            phi = (crank_angle + phi_offset) % (4.0 * math.pi)
            if phi < 1.85: # ~106 deg burn
                norm = phi / 1.85
                pulse = math.pow(math.sin(math.pi * norm), 2.2) * math.exp(-3.5 * norm)
                combustion_sum += pulse * (0.3 + 0.7 * throttle)
                
        # Exhaust wave steepening & dynamic lowpass
        non_linear = combustion_sum + 0.35 * (combustion_sum * abs(combustion_sum) - (combustion_sum**3) * 0.1)
        fc = 180.0 + (rpm / 7000.0) * 950.0 + throttle * 400.0
        alpha = math.exp(-2.0 * math.pi * fc / SAMPLE_RATE)
        lpf_state = (1.0 - alpha) * non_linear + alpha * lpf_state
        
        samples[n] = lpf_state * 1.2
    return samples

# =============================================================================
# 3. NO MAN'S SKY VOCALIEN CREATURE ROAR (WITH CHAOS BIFURCATION)
# =============================================================================
def bake_vocalien_creature_roar(duration: float = 2.5, mass_kg: float = 450.0, chaos: float = 0.65) -> list[float]:
    num_samples = int(duration * SAMPLE_RATE)
    samples = [0.0] * num_samples
    
    dt = 1.0 / SAMPLE_RATE
    x1 = 0.0001; x2 = 0.0001; v1 = 0.0; v2 = 0.0
    fwd = [0.0] * 10
    bwd = [0.0] * 10
    area = [1.0] * 10
    prev_lip = 0.0
    
    mass_scale = math.pow(80.0 / max(0.1, mass_kg), 0.38)
    m1 = 0.12 / mass_scale
    m2 = 0.03 / mass_scale
    k1 = 80.0 * mass_scale
    k2 = 20.0 * mass_scale
    kc = 25.0 * mass_scale
    b1 = 0.015 * math.sqrt(m1 * k1)
    b2 = 0.015 * math.sqrt(m2 * k2)
    
    for n in range(num_samples):
        t = n / SAMPLE_RATE
        # Envelope: Attack swell -> Sustained roar -> Release
        env = math.sin(math.pi * (t / duration)) ** 0.8
        Ps = 1800.0 * env
        if chaos > 0.01 and env > 0.3:
            Ps += chaos * 65.0 * math.sin(x1 * 1600.0)
            
        a1 = max(1e-6, 0.014 * (x1 + 0.0002))
        a2 = max(1e-6, 0.014 * (x2 + 0.0002))
        a_min = min(a1, a2)
        
        Ug = math.sqrt(max(0.0, (2.0 * Ps) / 1.14)) * a_min
        P1 = Ps * (1.0 - (a_min / a1)**2)
        
        fc1 = 3.0 * k1 * (x1 + 0.0002) if (x1 + 0.0002 < 0.0) else 0.0
        fc2 = 3.0 * k2 * (x2 + 0.0002) if (x2 + 0.0002 < 0.0) else 0.0
        
        acc1 = (P1 * 0.014 - k1 * x1 - kc * (x1 - x2) - b1 * v1 + fc1) / m1
        acc2 = (-k2 * x2 - kc * (x2 - x1) - b2 * v2 + fc2) / m2
        
        v1 += acc1 * dt; v2 += acc2 * dt
        x1 += v1 * dt;   x2 += v2 * dt
        
        # Area function (Vocal tract)
        mouth_open = 0.4 + 0.6 * env
        for i in range(10):
            frac = i / 9.0
            area[i] = (1.0 + 0.5 * math.sin(frac * math.pi)) * (0.2 + 1.8 * mouth_open if i == 9 else 1.0)
            
        fwd[0] = Ug + bwd[0] * 0.65
        for i in range(9):
            r = (area[i+1] - area[i]) / (area[i+1] + area[i])
            f = fwd[i]; b = bwd[i+1]
            fwd[i+1] = (1.0 + r) * f - r * b
            bwd[i]   = r * f + (1.0 - r) * b
            
        lip_out = fwd[9] - prev_lip
        prev_lip = fwd[9]
        samples[n] = math.tanh(lip_out * 4.5) * env
    return samples

# =============================================================================
# 4. RETURNAL MICRO-GRANULAR RAIN DOWNPOUR ON HELMET VISOR
# =============================================================================
def bake_rain_visor_downpour(duration: float = 3.0, drops_per_sec: float = 85.0) -> list[float]:
    num_samples = int(duration * SAMPLE_RATE)
    samples = [0.0] * num_samples
    
    # Generate Poisson onsets
    current_time = 0.0
    while current_time < duration:
        current_time += random.expovariate(drops_per_sec)
        if current_time >= duration:
            break
            
        # Modal polycarbonate impact (4.8 kHz ring)
        f0 = 4800.0 + (random.random() * 600.0 - 300.0)
        decay = 450.0
        amp = 0.3 + 0.7 * random.random()
        start_idx = int(current_time * SAMPLE_RATE)
        grain_len = int(0.015 * SAMPLE_RATE) # 15 ms
        
        for i in range(grain_len):
            idx = start_idx + i
            if idx < num_samples:
                t = i / SAMPLE_RATE
                val = math.sin(2.0 * math.pi * f0 * t) * math.exp(-t * decay) * amp
                samples[idx] += val * 0.4
    return samples

if __name__ == '__main__':
    parser = argparse.ArgumentParser(description='AAA Procedural Audio Generator')
    parser.add_argument('--sfx', choices=['bullet', 'v8', 'creature', 'rain'], default='bullet')
    parser.add_argument('--out', default='output.wav')
    parser.add_argument('--depth', type=int, choices=[16, 24], default=16)
    parser.add_argument('--duration', type=float, default=3.0)
    args = parser.parse_args()
    
    if args.sfx == 'bullet':
        data = bake_supersonic_bullet(distance_m=120.0, miss_dist_m=1.5, speed_mps=880.0)
    elif args.sfx == 'v8':
        data = bake_v8_engine_rev(duration=args.duration)
    elif args.sfx == 'creature':
        data = bake_vocalien_creature_roar(duration=args.duration, mass_kg=650.0, chaos=0.75)
    elif args.sfx == 'rain':
        data = bake_rain_visor_downpour(duration=args.duration, drops_per_sec=120.0)
        
    write_wav(args.out, data, sample_rate=SAMPLE_RATE, bit_depth=args.depth)
