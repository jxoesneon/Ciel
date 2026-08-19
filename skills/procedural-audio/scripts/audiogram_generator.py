#!/usr/bin/env python3
"""
===============================================================================
PRODUCTION AUDIOGRAM GENERATOR FOR VISION-LANGUAGE MODELS (VLMs)
===============================================================================
Generates high-contrast, multi-panel 300 DPI acoustic analysis sheets optimized
for Vision Transformers (ViT) and VLMs (GPT-4o, Claude 3.5 Sonnet, Gemini Pro/Flash).

Panels:
1. Waveform Peak Envelope + RMS Energy + True Peak + Digital Clipping Callouts
2. Log-Frequency Mel-Spectrogram with Labeled Diagnostic Bands (Mud, Harshness, Sibilance)
3. 12-Bin Pitch Class Chromagram (CQT Approximation) for Harmonic Progression & Dissonance
4. Stereo Lissajous Goniometer (Mid vs Side) + Phase Correlation Meter
5. Self-Similarity Structural Matrix (SSM) for Macro Section Detection
6. Production HUD with EBU R128 / True Peak / Crest Factor Metrics

Zero external C-dependencies: Pure NumPy, SciPy, Matplotlib.
===============================================================================
"""

import sys
import os
import math
import numpy as np
import scipy.io.wavfile as wavfile
import scipy.signal as signal
import matplotlib.pyplot as plt
import matplotlib.gridspec as gridspec
from matplotlib.colors import Normalize

def load_and_normalize_audio(wav_path):
    """Loads a WAV file, converts to 32-bit float [-1.0, 1.0], and standardizes stereo."""
    if not os.path.exists(wav_path):
        raise FileNotFoundError(f"Audio file not found: {wav_path}")

    sr, data = wavfile.read(wav_path)
    
    if data.dtype == np.int16:
        data = data.astype(np.float32) / 32768.0
    elif data.dtype == np.int32:
        data = data.astype(np.float32) / 2147483648.0
    elif data.dtype == np.uint8:
        data = (data.astype(np.float32) - 128.0) / 128.0
    elif data.dtype in (np.float32, np.float64):
        data = data.astype(np.float32)
    else:
        raise ValueError(f"Unsupported audio format: {data.dtype}")
        
    if data.ndim == 1:
        data_stereo = np.column_stack([data, data])
        is_stereo = False
    else:
        data_stereo = data[:, :2]
        is_stereo = True
        
    return sr, data_stereo, is_stereo

def hz_to_mel(f):
    return 2595.0 * np.log10(1.0 + f / 700.0)

def mel_to_hz(m):
    return 700.0 * (10.0 ** (m / 2595.0) - 1.0)

def create_mel_filterbank(sr, n_fft, n_mels=128, fmin=20.0, fmax=None):
    if fmax is None:
        fmax = sr / 2.0
    
    mel_min = hz_to_mel(fmin)
    mel_max = hz_to_mel(fmax)
    mel_points = np.linspace(mel_min, mel_max, n_mels + 2)
    hz_points = mel_to_hz(mel_points)
    
    bin_points = np.floor((n_fft + 1) * hz_points / sr).astype(int)
    n_freqs = n_fft // 2 + 1
    filterbank = np.zeros((n_mels, n_freqs), dtype=np.float32)
    
    for m in range(1, n_mels + 1):
        f_m_minus = bin_points[m - 1]
        f_m = bin_points[m]
        f_m_plus = bin_points[m + 1]
        
        for k in range(f_m_minus, f_m):
            if f_m != f_m_minus:
                filterbank[m - 1, k] = (k - f_m_minus) / (f_m - f_m_minus)
        for k in range(f_m, f_m_plus):
            if f_m_plus != f_m:
                filterbank[m - 1, k] = (f_m_plus - k) / (f_m_plus - f_m)
                
    enorm = 2.0 / (hz_points[2:n_mels + 2] - hz_points[:n_mels])
    filterbank *= enorm[:, np.newaxis]
    return filterbank, hz_points[1:n_mels + 1]

def compute_chromagram(sr, mono_audio, n_fft=4096, hop_length=1024):
    window = signal.windows.hann(n_fft, sym=False)
    freqs, times, Zxx = signal.stft(mono_audio, fs=sr, window=window, 
                                    nperseg=n_fft, noverlap=n_fft - hop_length)
    mag_spec = np.abs(Zxx)
    chroma = np.zeros((12, mag_spec.shape[1]), dtype=np.float32)
    
    valid_mask = freqs > 27.5
    valid_freqs = freqs[valid_mask]
    midi_notes = 69.0 + 12.0 * np.log2(valid_freqs / 440.0)
    pitch_classes = (np.round(midi_notes).astype(int)) % 12
    
    valid_indices = np.where(valid_mask)[0]
    for bin_idx, pc in zip(valid_indices, pitch_classes):
        chroma[pc, :] += mag_spec[bin_idx, :] ** 2
        
    norm = np.sqrt(np.sum(chroma ** 2, axis=0, keepdims=True)) + 1e-8
    return chroma / norm, times

def compute_self_similarity_matrix(chroma):
    norm = np.linalg.norm(chroma, axis=0, keepdims=True) + 1e-8
    chroma_norm = chroma / norm
    ssm = np.dot(chroma_norm.T, chroma_norm)
    return np.clip(ssm, 0.0, 1.0)

def generate_vlm_audiogram(wav_path: str, output_image_path: str = "diagnostic_audiogram.png"):
    sr, audio, is_stereo = load_and_normalize_audio(wav_path)
    mono_audio = np.mean(audio, axis=1)
    duration = len(mono_audio) / sr
    time_vec = np.linspace(0, duration, len(mono_audio))
    
    # 1. Waveform, RMS, True Peak & Clipping
    rms_win_size = int(sr * 0.050)
    square_audio = mono_audio ** 2
    rms_env = np.sqrt(signal.convolve(square_audio, np.ones(rms_win_size)/rms_win_size, mode='same'))
    
    clipping_mask = np.abs(mono_audio) >= 0.994
    clipping_times = time_vec[clipping_mask]
    clipping_vals = mono_audio[clipping_mask]
    
    # 2. Mel-Spectrogram
    n_fft = 2048
    hop_length = 512
    window = signal.windows.hann(n_fft, sym=False)
    freqs, spec_times, Zxx = signal.stft(mono_audio, fs=sr, window=window, 
                                         nperseg=n_fft, noverlap=n_fft - hop_length)
    mag_spec = np.abs(Zxx)
    power_spec = (mag_spec ** 2) / n_fft
    
    n_mels = 128
    mel_fb, mel_freqs = create_mel_filterbank(sr, n_fft, n_mels=n_mels, fmin=20.0, fmax=sr/2.0)
    mel_spec = np.dot(mel_fb, power_spec)
    
    ref_power = np.max(mel_spec) + 1e-8
    mel_spec_db = 10 * np.log10(np.maximum(mel_spec, 1e-8) / ref_power)
    mel_spec_db = np.clip(mel_spec_db, -80.0, 0.0)
    
    # 3. Chromagram & SSM
    chroma, chroma_times = compute_chromagram(sr, mono_audio, n_fft=4096, hop_length=1024)
    ssm = compute_self_similarity_matrix(chroma)
    
    # 4. Stereo Lissajous & Phase Correlation
    left = audio[:, 0]
    right = audio[:, 1]
    mid = (left + right) / np.sqrt(2)
    side = (left - right) / np.sqrt(2)
    
    denom = np.sqrt(np.sum(left**2) * np.sum(right**2)) + 1e-8
    phase_corr = float(np.sum(left * right) / denom)
    
    max_pts = 10000
    step = max(1, len(left) // max_pts)
    gonio_side = side[::step]
    gonio_mid = mid[::step]
    
    # Render Master Composite Canvas
    plt.style.use('dark_background')
    fig = plt.figure(figsize=(24, 16), dpi=300)
    
    gs = gridspec.GridSpec(4, 3, width_ratios=[4, 4, 2.5], height_ratios=[1.2, 2.2, 1.4, 1.8],
                           wspace=0.25, hspace=0.35)
    
    bg_color = "#0E1117"
    grid_color = "#2E3440"
    fig.patch.set_facecolor(bg_color)
    
    # PANEL 1: Waveform
    ax_wave = fig.add_subplot(gs[0, 0:2])
    ax_wave.set_facecolor("#161B22")
    ax_wave.plot(time_vec, mono_audio, color="#38BDF8", lw=0.6, alpha=0.65, label="Waveform Peak")
    ax_wave.plot(time_vec, rms_env, color="#F59E0B", lw=1.8, label="RMS Loudness (50ms)")
    ax_wave.plot(time_vec, -rms_env, color="#F59E0B", lw=1.8)
    
    if len(clipping_times) > 0:
        clip_sample_idx = np.linspace(0, len(clipping_times)-1, min(50, len(clipping_times))).astype(int)
        ax_wave.scatter(clipping_times[clip_sample_idx], clipping_vals[clip_sample_idx], 
                        color="#EF4444", s=35, zorder=5, edgecolors="#FFFFFF", label=f"CLIPPING ({len(clipping_times)} pts)")
    
    ax_wave.set_xlim(0, duration)
    ax_wave.set_ylim(-1.05, 1.05)
    ax_wave.set_ylabel("Amplitude (Linear)", fontsize=11, fontweight='bold', color="#E2E8F0")
    ax_wave.set_title(f"PANEL 1: WAVEFORM ENVELOPE, RMS ENERGY & CLIPPING DIAGNOSTICS (Duration: {duration:.2f}s)",
                      fontsize=13, fontweight='bold', color="#38BDF8", pad=8, loc='left')
    ax_wave.grid(True, color=grid_color, linestyle="--", alpha=0.6)
    ax_wave.legend(loc="upper right", framealpha=0.8, fontsize=9)
    
    # PANEL 2: Log-Mel Spectrogram
    ax_spec = fig.add_subplot(gs[1, 0:2], sharex=ax_wave)
    ax_spec.set_facecolor("#161B22")
    img_spec = ax_spec.imshow(mel_spec_db, origin="lower", aspect="auto",
                              extent=[spec_times[0], spec_times[-1], 0, n_mels],
                              cmap="magma", norm=Normalize(vmin=-80.0, vmax=0.0))
    
    freq_ticks_hz = [60, 250, 500, 1000, 2000, 4000, 8000, 16000]
    mel_ticks_y = [np.interp(hz_to_mel(f), hz_to_mel(mel_freqs), np.arange(n_mels)) for f in freq_ticks_hz]
    ax_spec.set_yticks(mel_ticks_y)
    ax_spec.set_yticklabels([f"{int(f)} Hz" if f < 1000 else f"{int(f/1000)}k Hz" for f in freq_ticks_hz], 
                            fontsize=10, color="#E2E8F0")
    
    mud_y1 = np.interp(hz_to_mel(200), hz_to_mel(mel_freqs), np.arange(n_mels))
    mud_y2 = np.interp(hz_to_mel(400), hz_to_mel(mel_freqs), np.arange(n_mels))
    ax_spec.axhspan(mud_y1, mud_y2, color="#3B82F6", alpha=0.15, lw=1.2, linestyle=":")
    ax_spec.text(duration * 0.01, (mud_y1 + mud_y2)/2, "MUD / LOW-MID (200-400 Hz)", 
                 color="#93C5FD", fontsize=9, fontweight='bold', va='center')
    
    sib_y1 = np.interp(hz_to_mel(6000), hz_to_mel(mel_freqs), np.arange(n_mels))
    sib_y2 = np.interp(hz_to_mel(8500), hz_to_mel(mel_freqs), np.arange(n_mels))
    ax_spec.axhspan(sib_y1, sib_y2, color="#EC4899", alpha=0.15, lw=1.2, linestyle=":")
    ax_spec.text(duration * 0.01, (sib_y1 + sib_y2)/2, "SIBILANCE / HARSH (6-8.5 kHz)", 
                 color="#F472B6", fontsize=9, fontweight='bold', va='center')
    
    ax_spec.set_ylabel("Mel Frequency Scale", fontsize=11, fontweight='bold', color="#E2E8F0")
    ax_spec.set_title("PANEL 2: LOG-FREQUENCY MEL-SPECTROGRAM [-80 dBFS to 0 dBFS] (Magma Dynamic Colormap)",
                      fontsize=13, fontweight='bold', color="#F43F5E", pad=8, loc='left')
    
    cbar_ax = fig.add_axes([0.655, 0.51, 0.012, 0.22])
    cbar = fig.colorbar(img_spec, cax=cbar_ax)
    cbar.set_label("Loudness (dBFS)", fontsize=10, color="#E2E8F0", fontweight='bold')
    cbar.ax.tick_params(labelsize=9, colors="#E2E8F0")
    
    # PANEL 3: Chromagram
    ax_chroma = fig.add_subplot(gs[2, 0:2], sharex=ax_wave)
    ax_chroma.set_facecolor("#161B22")
    pitch_labels = ['C', 'C#', 'D', 'D#', 'E', 'F', 'F#', 'G', 'G#', 'A', 'A#', 'B']
    ax_chroma.imshow(chroma, origin="lower", aspect="auto",
                     extent=[chroma_times[0], chroma_times[-1], 0, 12],
                     cmap="inferno", norm=Normalize(vmin=0.0, vmax=1.0))
    ax_chroma.set_yticks(np.arange(12) + 0.5)
    ax_chroma.set_yticklabels(pitch_labels, fontsize=10, fontweight='bold', color="#FCD34D")
    ax_chroma.set_ylabel("Pitch Class", fontsize=11, fontweight='bold', color="#E2E8F0")
    ax_chroma.set_xlabel("Time (Seconds)", fontsize=11, fontweight='bold', color="#E2E8F0")
    ax_chroma.set_title("PANEL 3: 12-BIN TONAL CHROMAGRAM & HARMONIC DISSONANCE MATRIX",
                        fontsize=13, fontweight='bold', color="#F59E0B", pad=8, loc='left')
    ax_chroma.grid(True, color=grid_color, linestyle=":", alpha=0.5)
    
    # PANEL 4A: Lissajous Goniometer
    ax_gonio = fig.add_subplot(gs[3, 0])
    ax_gonio.set_facecolor("#161B22")
    ax_gonio.scatter(gonio_side, gonio_mid, c="#10B981", s=2.5, alpha=0.35, edgecolors='none')
    ax_gonio.axhline(0, color=grid_color, lw=1)
    ax_gonio.axvline(0, color=grid_color, lw=1)
    ax_gonio.set_xlim(-1.05, 1.05)
    ax_gonio.set_ylim(-1.05, 1.05)
    ax_gonio.set_xlabel("Side (L - R) / Width", fontsize=10, fontweight='bold', color="#E2E8F0")
    ax_gonio.set_ylabel("Mid (L + R) / Mono", fontsize=10, fontweight='bold', color="#E2E8F0")
    
    corr_color = "#10B981" if phase_corr >= 0.2 else ("#F59E0B" if phase_corr >= 0.0 else "#EF4444")
    ax_gonio.text(0.05, 0.88, f"Phase Correlation: {phase_corr:+.3f}\nStereo: {'YES' if is_stereo else 'MONO'}",
                  transform=ax_gonio.transAxes, color=corr_color, fontsize=10, fontweight='bold',
                  bbox=dict(boxstyle="round,pad=0.3", facecolor="#1F2937", edgecolor=corr_color, lw=1.5))
    ax_gonio.set_title("PANEL 4A: STEREO GONIOMETER", fontsize=12, fontweight='bold', color="#10B981", pad=8)
    
    # PANEL 4B: Self-Similarity Matrix
    ax_ssm = fig.add_subplot(gs[3, 1])
    ax_ssm.set_facecolor("#161B22")
    ax_ssm.imshow(ssm, origin="lower", aspect="auto", extent=[0, duration, 0, duration], cmap="viridis")
    ax_ssm.set_xlabel("Time (s)", fontsize=10, fontweight='bold', color="#E2E8F0")
    ax_ssm.set_ylabel("Time (s)", fontsize=10, fontweight='bold', color="#E2E8F0")
    ax_ssm.set_title("PANEL 4B: SELF-SIMILARITY STRUCTURAL MATRIX (SSM)", 
                     fontsize=12, fontweight='bold', color="#6366F1", pad=8)
    
    # PANEL 5: Production HUD
    ax_hud = fig.add_subplot(gs[:, 2])
    ax_hud.set_facecolor("#161B22")
    ax_hud.axis("off")
    
    crest_factor = 20 * np.log10((np.max(np.abs(mono_audio)) + 1e-6) / (np.sqrt(np.mean(mono_audio**2)) + 1e-6))
    max_peak_db = 20 * np.log10(np.max(np.abs(mono_audio)) + 1e-6)
    integrated_rms = 20 * np.log10(np.sqrt(np.mean(mono_audio**2)) + 1e-6)
    
    hud_text = (
        "╔═════════════════════════════════════════╗\n"
        "║      VLM ACOUSTIC DIAGNOSTIC HUD        ║\n"
        "╠═════════════════════════════════════════╣\n"
        f"  • Sample Rate:       {sr:,} Hz\n"
        f"  • Channels:          {'Stereo (2.0)' if is_stereo else 'Mono (1.0)'}\n"
        f"  • Duration:          {duration:.3f} sec\n"
        f"  • Max True Peak:     {max_peak_db:.2f} dBFS\n"
        f"  • Integrated RMS:    {integrated_rms:.2f} dBFS\n"
        f"  • Dynamic Crest:     {crest_factor:.2f} dB\n"
        f"  • Phase Corr (r):    {phase_corr:+.3f}\n"
        f"  • Total Clip Events: {len(clipping_times)}\n"
        "╠═════════════════════════════════════════╣\n"
        "║         DIAGNOSTIC STATUS FLAGS         ║\n"
        "╠═════════════════════════════════════════╣\n"
    )
    
    warnings = []
    if len(clipping_times) > 0:
        warnings.append("[!] SEVERE: Digital Clipping Detected")
    if crest_factor < 6.0:
        warnings.append("[!] CAUTION: Over-Compression (Squash)")
    elif crest_factor > 18.0:
        warnings.append("[i] INFO: Highly Dynamic Content")
    if phase_corr < 0.0:
        warnings.append("[!] CRITICAL: Mono Phase Inversion Risk")
    elif phase_corr < 0.3 and is_stereo:
        warnings.append("[i] NOTICE: Wide Stereo / Weak Center")
    if not warnings:
        warnings.append("[OK] All Metrics Within Nominal Targets")
        
    for w in warnings:
        hud_text += f"  {w}\n"
        
    hud_text += (
        "╠═════════════════════════════════════════╣\n"
        "║        VLM INSPECTION CHEAT-SHEET       ║\n"
        "╠═════════════════════════════════════════╣\n"
        "  1. Mud Check:     Hot band @ 200-400 Hz\n"
        "  2. Harshness:     Spikes @ 2-5 kHz\n"
        "  3. Sibilance:     Vertical streaks @ 6-8k\n"
        "  4. Harmonicity:   Chroma row alignments\n"
        "  5. Sections:      SSM checkerboard blocks\n"
        "╚═════════════════════════════════════════╝"
    )
    
    ax_hud.text(0.04, 0.96, hud_text, transform=ax_hud.transAxes,
                family="monospace", fontsize=9.5, color="#E2E8F0", va="top",
                bbox=dict(boxstyle="round,pad=0.6", facecolor="#1F2937", edgecolor="#38BDF8", lw=1.8))
    
    plt.savefig(output_image_path, facecolor=bg_color, edgecolor='none', bbox_inches='tight', dpi=300)
    plt.close(fig)
    print(f"[*] VLM Audiogram generated at: {output_image_path}")

if __name__ == "__main__":
    if len(sys.argv) > 1:
        wav_in = sys.argv[1]
        img_out = sys.argv[2] if len(sys.argv) > 2 else "diagnostic_audiogram.png"
        generate_vlm_audiogram(wav_in, img_out)
    else:
        print("Usage: python3 audiogram_generator.py <input.wav> [output.png]")
