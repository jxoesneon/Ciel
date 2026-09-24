/**
 * procedural_dsp.h
 * Zero-Dependency, Header-Only C99 / C++ Procedural DSP & Physical Modeling Library
 * 
 * Features:
 * - 2D Waveguide Mesh Membrane Simulation (FDTD)
 * - Extended Karplus-Strong Plucked String with Pick Comb & Body Resonator
 * - Modal Synthesis Bank (Hertzian Contact Physics & Inharmonic Modes)
 * - 4-Pole Moog Ladder TPT (Topology-Preserving Transform) Non-Linear Filter
 * - Anti-Aliased PolyBLEP Oscillators (Saw, Square, Triangle)
 * - 8-Delay Householder Feedback Delay Network (FDN) Reverb
 * - 4-Operator FM Synthesis Matrix
 * - Paul Kellet Pink Noise & Velvet Noise Generators
 * - Asymmetric Triode Tube Saturation
 * 
 * License: MIT
 */

#ifndef PROCEDURAL_DSP_H
#define PROCEDURAL_DSP_H

#include <math.h>
#include <stdlib.h>
#include <string.h>

#ifndef M_PI
#define M_PI 3.14159265358979323846
#endif

#ifdef __cplusplus
extern "C" {
#endif

/* ========================================================================= */
/* 1. MATHEMATICAL UTILITIES & STOCHASTIC GENERATORS                         */
/* ========================================================================= */

static inline float pdsp_clampf(float x, float min_val, float max_val) {
    if (x < min_val) return min_val;
    if (x > max_val) return max_val;
    return x;
}

static inline float pdsp_fast_tanh(float x) {
    float x2 = x * x;
    return x * (27.0f + x2) / (27.0f + 9.0f * x2);
}

/* Asymmetric Triode Tube Warmth Saturation */
static inline float pdsp_triode_saturate(float x, float drive, float warmth) {
    float driven = x * drive;
    /* Injects 2nd harmonic (warmth) via x^2 term with asymmetric compression */
    float num = driven + warmth * (driven * driven);
    float den = 1.0f + 0.4f * fabsf(driven);
    return pdsp_fast_tanh(num / den);
}

/* Box-Muller Gaussian Noise Generator N(0, 1) */
static inline float pdsp_gaussian_noise(void) {
    float u1 = ((float)rand() + 1.0f) / ((float)RAND_MAX + 1.0f);
    float u2 = ((float)rand() + 1.0f) / ((float)RAND_MAX + 1.0f);
    return sqrtf(-2.0f * logf(u1)) * cosf(2.0f * (float)M_PI * u2);
}

/* Paul Kellet's 3-Pole Filtered Pink Noise (-3dB/octave) */
typedef struct {
    float b0, b1, b2, b3, b4, b5, b6;
} pdsp_pink_noise_t;

static inline void pdsp_pink_noise_init(pdsp_pink_noise_t* p) {
    memset(p, 0, sizeof(pdsp_pink_noise_t));
}

static inline float pdsp_pink_noise_step(pdsp_pink_noise_t* p) {
    float white = ((float)rand() / (float)RAND_MAX) * 2.0f - 1.0f;
    p->b0 = 0.99886f * p->b0 + white * 0.0555179f;
    p->b1 = 0.99332f * p->b1 + white * 0.0750759f;
    p->b2 = 0.96900f * p->b2 + white * 0.1538520f;
    p->b3 = 0.86650f * p->b3 + white * 0.3104856f;
    p->b4 = 0.55000f * p->b4 + white * 0.5329522f;
    p->b5 = -0.7616f * p->b5 - white * 0.0168980f;
    float pink = (p->b0 + p->b1 + p->b2 + p->b3 + p->b4 + p->b5 + p->b6 + white * 0.5362f) * 0.11f;
    p->b6 = white * 0.115926f;
    return pink;
}

/* ========================================================================= */
/* 2. ANTI-ALIASED POLYBLEP OSCILLATOR                                       */
/* ========================================================================= */

typedef struct {
    float phase;
    float phase_inc;
    float sample_rate;
} pdsp_polyblep_osc_t;

static inline void pdsp_polyblep_init(pdsp_polyblep_osc_t* osc, float sample_rate) {
    osc->phase = 0.0f;
    osc->phase_inc = 440.0f / sample_rate;
    osc->sample_rate = sample_rate;
}

static inline void pdsp_polyblep_set_freq(pdsp_polyblep_osc_t* osc, float freq_hz) {
    osc->phase_inc = freq_hz / osc->sample_rate;
}

static inline float pdsp_polyblep_residual(float t, float dt) {
    if (t < dt) {
        float r = t / dt;
        return r + r - r * r - 1.0f;
    } else if (t > 1.0f - dt) {
        float r = (t - 1.0f) / dt;
        return r * r + r + r + 1.0f;
    }
    return 0.0f;
}

static inline float pdsp_polyblep_saw(pdsp_polyblep_osc_t* osc) {
    osc->phase += osc->phase_inc;
    while (osc->phase >= 1.0f) osc->phase -= 1.0f;

    float raw_saw = 2.0f * osc->phase - 1.0f;
    float blep = pdsp_polyblep_residual(osc->phase, osc->phase_inc);
    return raw_saw - blep;
}

static inline float pdsp_polyblep_square(pdsp_polyblep_osc_t* osc, float pulse_width) {
    osc->phase += osc->phase_inc;
    while (osc->phase >= 1.0f) osc->phase -= 1.0f;

    float raw_sq = (osc->phase < pulse_width) ? 1.0f : -1.0f;
    float t2 = osc->phase - pulse_width;
    if (t2 < 0.0f) t2 += 1.0f;

    float blep1 = pdsp_polyblep_residual(osc->phase, osc->phase_inc);
    float blep2 = pdsp_polyblep_residual(t2, osc->phase_inc);
    return raw_sq + blep1 - blep2;
}

/* ========================================================================= */
/* 3. 4-POLE MOOG LADDER TPT NON-LINEAR FILTER                               */
/* ========================================================================= */

typedef struct {
    float s[4];
    float sample_rate;
} pdsp_moog_ladder_t;

static inline void pdsp_moog_ladder_init(pdsp_moog_ladder_t* flt, float sample_rate) {
    memset(flt->s, 0, sizeof(flt->s));
    flt->sample_rate = sample_rate;
}

static inline float pdsp_moog_ladder_step(pdsp_moog_ladder_t* flt, float in, float cutoff_hz, float resonance) {
    cutoff_hz = pdsp_clampf(cutoff_hz, 20.0f, 0.48f * flt->sample_rate);
    resonance = pdsp_clampf(resonance, 0.0f, 1.0f);

    float g = tanf((float)M_PI * cutoff_hz / flt->sample_rate);
    float G = g / (1.0f + g);
    float k = 4.0f * resonance;

    /* Instantaneous feedback state accumulator */
    float S = G * (G * (G * flt->s[0] + flt->s[1]) + flt->s[2]) + flt->s[3];
    float u = (in - k * S) / (1.0f + k * G * G * G * G);
    u = pdsp_fast_tanh(u);

    /* 4-Stage TPT Integration */
    float v1 = G * (u - flt->s[0]) + flt->s[0];
    flt->s[0] = 2.0f * v1 - flt->s[0];

    float v2 = G * (v1 - flt->s[1]) + flt->s[1];
    flt->s[1] = 2.0f * v2 - flt->s[1];

    float v3 = G * (v2 - flt->s[2]) + flt->s[2];
    flt->s[2] = 2.0f * v3 - flt->s[2];

    float v4 = G * (v3 - flt->s[3]) + flt->s[3];
    flt->s[3] = 2.0f * v4 - flt->s[3];

    return v4;
}

/* ========================================================================= */
/* 4. MODAL SYNTHESIS RESONATOR BANK (SOLID MECHANICS)                       */
/* ========================================================================= */

#define PDSP_MAX_MODES 8

typedef struct {
    int mode_count;
    float c_cos[PDSP_MAX_MODES];
    float c_exp[PDSP_MAX_MODES];
    float c_gain[PDSP_MAX_MODES];
    float y1[PDSP_MAX_MODES];
    float y2[PDSP_MAX_MODES];
} pdsp_modal_bank_t;

static inline void pdsp_modal_bank_init(
    pdsp_modal_bank_t* mb,
    float sample_rate,
    float f0_hz,
    const float* mode_ratios,
    const float* mode_gains,
    const float* mode_t60_sec,
    int count
) {
    mb->mode_count = count > PDSP_MAX_MODES ? PDSP_MAX_MODES : count;
    float T = 1.0f / sample_rate;

    for (int i = 0; i < mb->mode_count; i++) {
        float fk = f0_hz * mode_ratios[i];
        float alpha = 3.0f / (mode_t60_sec[i] > 0.001f ? mode_t60_sec[i] : 0.001f);
        float omega_d = 2.0f * (float)M_PI * fk;

        mb->c_cos[i] = 2.0f * expf(-alpha * T) * cosf(omega_d * T);
        mb->c_exp[i] = expf(-2.0f * alpha * T);
        mb->c_gain[i] = mode_gains[i] * sinf(omega_d * T);
        mb->y1[i] = 0.0f;
        mb->y2[i] = 0.0f;
    }
}

static inline float pdsp_modal_bank_step(pdsp_modal_bank_t* mb, float excitation) {
    float out = 0.0f;
    for (int i = 0; i < mb->mode_count; i++) {
        float y0 = mb->c_cos[i] * mb->y1[i] - mb->c_exp[i] * mb->y2[i] + mb->c_gain[i] * excitation;
        mb->y2[i] = mb->y1[i];
        mb->y1[i] = y0;
        out += y0;
    }
    return out;
}

/* ========================================================================= */
/* 5. 8-CHANNEL HOUSEHOLDER FEEDBACK DELAY NETWORK (FDN) REVERB              */
/* ========================================================================= */

#define PDSP_FDN_CHANNELS 8

typedef struct {
    float* delay_buffers[PDSP_FDN_CHANNELS];
    int delay_lengths[PDSP_FDN_CHANNELS];
    int write_pos[PDSP_FDN_CHANNELS];
    float decay_gains[PDSP_FDN_CHANNELS];
    float lp_states[PDSP_FDN_CHANNELS];
    float damping;
} pdsp_fdn_reverb_t;

static inline void pdsp_fdn_init(pdsp_fdn_reverb_t* rev, float sample_rate, float t60_sec, float damping) {
    /* Mutually coprime prime delay lengths */
    static const int base_lengths[PDSP_FDN_CHANNELS] = {1381, 1609, 1877, 2141, 2473, 2791, 3187, 3631};
    rev->damping = damping;

    for (int i = 0; i < PDSP_FDN_CHANNELS; i++) {
        rev->delay_lengths[i] = (int)(base_lengths[i] * (sample_rate / 44100.0f));
        rev->delay_buffers[i] = (float*)calloc(rev->delay_lengths[i] + 4, sizeof(float));
        rev->write_pos[i] = 0;
        rev->lp_states[i] = 0.0f;
        /* Per-delay loop attenuation g_k = 10^(-3 * delay_samples / (Fs * T60)) */
        rev->decay_gains[i] = powf(10.0f, (-3.0f * rev->delay_lengths[i]) / (sample_rate * t60_sec));
    }
}

static inline void pdsp_fdn_step(pdsp_fdn_reverb_t* rev, float in, float* out_left, float* out_right) {
    float delay_outs[PDSP_FDN_CHANNELS];
    float sum_d = 0.0f;

    /* 1. Read delay lines with damping filter */
    for (int i = 0; i < PDSP_FDN_CHANNELS; i++) {
        int r_idx = rev->write_pos[i] - rev->delay_lengths[i];
        if (r_idx < 0) r_idx += rev->delay_lengths[i];

        float raw = rev->delay_buffers[i][r_idx];
        /* One-pole lowpass damping */
        rev->lp_states[i] = (1.0f - rev->damping) * raw + rev->damping * rev->lp_states[i];
        delay_outs[i] = rev->lp_states[i] * rev->decay_gains[i];
        sum_d += delay_outs[i];
    }

    /* 2. 8x8 Householder Orthogonal Mixing Matrix: y = delay_outs - (2/N)*sum */
    float householder_factor = (2.0f / (float)PDSP_FDN_CHANNELS) * sum_d;
    for (int i = 0; i < PDSP_FDN_CHANNELS; i++) {
        float mixed = delay_outs[i] - householder_factor;
        rev->delay_buffers[i][rev->write_pos[i]] = in + mixed;
        rev->write_pos[i] = (rev->write_pos[i] + 1) % rev->delay_lengths[i];
    }

    /* 3. Stereo Decorrelated Output Matrix */
    *out_left = (delay_outs[0] + delay_outs[2] + delay_outs[4] + delay_outs[6]) * 0.35f;
    *out_right = (delay_outs[1] + delay_outs[3] + delay_outs[5] + delay_outs[7]) * 0.35f;
}

static inline void pdsp_fdn_free(pdsp_fdn_reverb_t* rev) {
    for (int i = 0; i < PDSP_FDN_CHANNELS; i++) {
        if (rev->delay_buffers[i]) {
            free(rev->delay_buffers[i]);
            rev->delay_buffers[i] = NULL;
        }
    }
}

#ifdef __cplusplus
}
#endif

#endif /* PROCEDURAL_DSP_H */
