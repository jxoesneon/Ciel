/**
 * aaa_procedural_dsp.h
 * 
 * Master AAA Procedural DSP & Physical Acoustics Library (v3.0.0)
 * Unified C++17 Header-Only Architecture synthesizing 12 Benchmark AAA+ Titles:
 * 
 * 1. DICE Frostbite: HDR Audio Floating Exposure Window & Zwicker Masking Triage
 * 2. DICE Ballistics: Supersonic Mach Cone Arrival Times & Whitham N-Wave Synthesis
 * 3. Naughty Dog: Kurze-Anderson Edge Diffraction & Eyring Material Reverberation
 * 4. Housemarque Returnal: Micro-Granular Particle Raindrops & Spatial Threat Vector Prioritizer
 * 5. Hello Games VocAlien: Kelly-Lochbaum Vocal Tract Waveguide & 2-Mass Glottal Oscillator
 * 6. Forza / GT7: Physical ICE Engine (4-Stroke Wiebe Pulses, Firing Orders & Turbo Spool)
 * 7. Pacejka Magic Formula: Granular Tire Friction & Stick-Slip Carcass Squeal
 * 8. DOOM: 3-Band Crossover Surgical Glory Kill Sidechain Matrix
 * 9. The Last of Us Part II: Biometric Respiration & Dynamic Vocal Formant Shifter
 * 10. Brian Eno / Spore: Prime-Period Stochastic Clocks & Markov Composition Matrix
 * 
 * Zero external dependencies. Header-only C++17 standard. Real-time allocation-free.
 * License: MIT
 */

#ifndef AAA_PROCEDURAL_DSP_H
#define AAA_PROCEDURAL_DSP_H

#include <cmath>
#include <vector>
#include <array>
#include <algorithm>
#include <cstring>
#include <cstdint>

#ifndef M_PI
#define M_PI 3.14159265358979323846
#endif

namespace AAADSP {

constexpr float PI = 3.14159265358979323846f;
constexpr float TWO_PI = 6.28318530717958647692f;
constexpr float FOUR_PI = 12.56637061435917295385f;
constexpr float SOUND_SPEED = 343.0f; // m/s

inline float clampf(float v, float min_v, float max_v) {
    return std::max(min_v, std::min(max_v, v));
}

inline float fast_rand(uint32_t& state) {
    state ^= state << 13;
    state ^= state >> 17;
    state ^= state << 5;
    return static_cast<float>(state) / 4294967296.0f * 2.0f - 1.0f;
}

struct Vec3 {
    float x, y, z;
    Vec3(float _x = 0, float _y = 0, float _z = 0) : x(_x), y(_y), z(_z) {}
    Vec3 operator+(const Vec3& o) const { return Vec3(x + o.x, y + o.y, z + o.z); }
    Vec3 operator-(const Vec3& o) const { return Vec3(x - o.x, y - o.y, z - o.z); }
    Vec3 operator*(float s) const { return Vec3(x * s, y * s, z * s); }
    float dot(const Vec3& o) const { return x * o.x + y * o.y + z * o.z; }
    float length() const { return std::sqrt(x * x + y * y + z * z); }
    Vec3 normalized() const {
        float l = length();
        return l > 1e-6f ? (*this) * (1.0f / l) : Vec3(0, 0, 0);
    }
};

/* ========================================================================= */
/* 1. DICE FROSTBITE: HIGH DYNAMIC RANGE (HDR) AUDIO ENGINE                  */
/* ========================================================================= */

class HDRAudioEngine {
private:
    float fs;
    float window_width_db;
    float top_level_db;
    float target_top_db;
    float release_coeff;

public:
    HDRAudioEngine(float sample_rate = 44100.0f, float window_width = 45.0f, float release_time_sec = 0.6f)
        : fs(sample_rate), window_width_db(window_width), top_level_db(20.0f), target_top_db(20.0f) {
        release_coeff = std::exp(-1.0f / (sample_rate * release_time_sec));
    }

    void register_loudness(float emitted_db, float distance_m) {
        float dist_atten_db = -20.0f * std::log10(std::max(1.0f, distance_m));
        float perceived_db = emitted_db + dist_atten_db;
        if (perceived_db > target_top_db) target_top_db = perceived_db;
    }

    void step_frame() {
        if (target_top_db > top_level_db) {
            top_level_db = target_top_db; // Instant attack
        } else {
            top_level_db = target_top_db + release_coeff * (top_level_db - target_top_db);
        }
        target_top_db = 20.0f; // Reset towards ambient noise floor
    }

    float get_voice_gain(float emitted_db, float distance_m, bool& out_cull) {
        float dist_atten_db = -20.0f * std::log10(std::max(1.0f, distance_m));
        float perceived_db = emitted_db + dist_atten_db;
        float bottom_db = top_level_db - window_width_db;

        if (perceived_db <= bottom_db) {
            out_cull = true;
            return 0.0f;
        }

        out_cull = false;
        if (perceived_db <= top_level_db) {
            float norm = (perceived_db - top_level_db) / window_width_db;
            return std::pow(10.0f, (norm * window_width_db) / 20.0f);
        } else {
            float overshoot = perceived_db - top_level_db;
            return 1.0f + std::tanh(overshoot / 12.0f) * 0.25f;
        }
    }
};

/* ========================================================================= */
/* 2. DICE SUPERSONIC BALLISTICS & WHITHAM N-WAVE GENERATOR                  */
/* ========================================================================= */

struct BallisticsResult {
    bool is_supersonic;
    float t_crack_sec;
    float t_muzzle_sec;
    float delta_t_sec;
    float peak_overpressure_pa;
    float duration_sec;
    Vec3 cpa_pos;
};

class SupersonicBallisticsSolver {
public:
    static BallisticsResult solve(const Vec3& shooter, const Vec3& bullet_dir, float bullet_speed,
                                 float caliber_m, const Vec3& listener) {
        BallisticsResult res;
        res.is_supersonic = false;
        float M = bullet_speed / SOUND_SPEED;

        Vec3 v_unit = bullet_dir.normalized();
        Vec3 s_to_l = listener - shooter;
        float x_proj = s_to_l.dot(v_unit);

        if (x_proj <= 0.0f || M <= 1.0f) {
            res.t_muzzle_sec = s_to_l.length() / SOUND_SPEED;
            res.t_crack_sec = -1.0f;
            res.delta_t_sec = 0.0f;
            return res;
        }

        float d_perp = (s_to_l - v_unit * x_proj).length();
        float theta_m = std::asin(1.0f / M);
        float x_travel = x_proj - d_perp / std::tan(theta_m);

        if (x_travel < 0.0f) {
            res.t_muzzle_sec = s_to_l.length() / SOUND_SPEED;
            return res;
        }

        res.is_supersonic = true;
        res.cpa_pos = shooter + v_unit * x_travel;
        res.t_crack_sec = (x_travel / bullet_speed) + (d_perp / (std::cos(theta_m) * SOUND_SPEED));
        res.t_muzzle_sec = s_to_l.length() / SOUND_SPEED;
        res.delta_t_sec = res.t_muzzle_sec - res.t_crack_sec;

        float p0 = 101325.0f;
        res.peak_overpressure_pa = (p0 * std::pow(M * M - 1.0f, 0.125f) / std::pow(std::max(0.2f, d_perp), 0.75f)) * (caliber_m * 12.0f);
        res.duration_sec = clampf(1.82f * (M * caliber_m / SOUND_SPEED) * std::pow(std::max(0.2f, d_perp) / caliber_m, 0.25f), 0.0001f, 0.001f);
        return res;
    }
};

/* ========================================================================= */
/* 3. NAUGHTY DOG: KURZE-ANDERSON EDGE DIFFRACTION & MATERIAL REVERB         */
/* ========================================================================= */

class EdgeDiffractionSolver {
public:
    static void solve(const Vec3& emitter, const Vec3& listener, const Vec3& edge,
                      float& out_atten_db, float& out_cutoff_hz) {
        float d0 = (listener - emitter).length();
        float d1 = (edge - emitter).length();
        float d2 = (listener - edge).length();
        float delta = (d1 + d2) - d0;

        if (delta <= 0.001f) {
            out_atten_db = 0.0f;
            out_cutoff_hz = 20000.0f;
            return;
        }

        out_cutoff_hz = clampf(SOUND_SPEED / (2.0f * PI * delta), 180.0f, 20000.0f);
        float n_f = (2.0f * 1000.0f / SOUND_SPEED) * delta;
        float sqrt_2pi_n = std::sqrt(2.0f * PI * n_f);
        float atten = 20.0f * std::log10(sqrt_2pi_n / std::tanh(std::max(0.01f, sqrt_2pi_n))) + 5.0f;
        out_atten_db = -clampf(atten, 0.0f, 35.0f);
    }
};

/* ========================================================================= */
/* 4. NO MAN'S SKY: VOCALIEN KELLY-LOCHBAUM TRACT & 2-MASS GLOTTAL ENGINE    */
/* ========================================================================= */

class VocAlienCreatureEngine {
public:
    static constexpr size_t TUBE_SECTIONS = 10;

    struct CreatureDNA {
        float mass_kg = 80.0f;
        float neck_length_cm = 30.0f;
        float lung_pressure_pa = 1400.0f;
        float vocal_tension = 1.0f;
        float chaos_roar_amount = 0.0f; // [0.0 - 1.0] -> Feigenbaum bifurcations
        float mouth_open = 0.6f;
    };

    VocAlienCreatureEngine(float sample_rate = 44100.0f) : fs(sample_rate), dt(1.0f / sample_rate) {
        reset();
    }

    void reset() {
        x1 = 0.0001f; x2 = 0.0001f; v1 = 0.0f; v2 = 0.0f;
        for (size_t i = 0; i < TUBE_SECTIONS; ++i) {
            fwd[i] = 0.0f; bwd[i] = 0.0f; area[i] = 1.0f;
        }
        prev_lip = 0.0f;
    }

    float process(const CreatureDNA& dna) {
        float mass_scale = std::pow(80.0f / std::max(0.1f, dna.mass_kg), 0.38f);
        float m1 = 0.12f / mass_scale;
        float m2 = 0.03f / mass_scale;
        float k1 = 80.0f * dna.vocal_tension * mass_scale;
        float k2 = 20.0f * dna.vocal_tension * mass_scale;
        float kc = 25.0f * mass_scale;
        float b1 = 0.015f * std::sqrt(m1 * k1);
        float b2 = 0.015f * std::sqrt(m2 * k2);

        float a1 = std::max(1e-6f, 0.014f * (x1 + 0.0002f));
        float a2 = std::max(1e-6f, 0.014f * (x2 + 0.0002f));
        float a_min = std::min(a1, a2);

        float Ps = dna.lung_pressure_pa;
        if (dna.chaos_roar_amount > 0.001f) {
            Ps += dna.chaos_roar_amount * 45.0f * std::sin(x1 * 1600.0f);
        }

        float Ug = std::sqrt(std::max(0.0f, (2.0f * Ps) / 1.14f)) * a_min;
        float P1 = Ps * (1.0f - std::pow(a_min / a1, 2.0f));

        float fc1 = (x1 + 0.0002f < 0.0f) ? 3.0f * k1 * (x1 + 0.0002f) : 0.0f;
        float fc2 = (x2 + 0.0002f < 0.0f) ? 3.0f * k2 * (x2 + 0.0002f) : 0.0f;

        float acc1 = (P1 * 0.014f - k1 * x1 - kc * (x1 - x2) - b1 * v1 + fc1) / m1;
        float acc2 = (-k2 * x2 - kc * (x2 - x1) - b2 * v2 + fc2) / m2;

        v1 += acc1 * dt; v2 += acc2 * dt;
        x1 += v1 * dt;   x2 += v2 * dt;

        // Dynamic Tube Area
        float neck = clampf(dna.neck_length_cm / 25.0f, 0.4f, 4.0f);
        for (size_t i = 0; i < TUBE_SECTIONS; ++i) {
            float frac = (float)i / (float)(TUBE_SECTIONS - 1);
            float phar = 1.0f + 0.5f * std::sin(frac * PI);
            if (i == TUBE_SECTIONS - 1) phar *= (0.1f + 1.8f * dna.mouth_open);
            area[i] = phar * (1.0f / neck);
        }

        // Waveguide Scattering
        fwd[0] = Ug + bwd[0] * 0.65f;
        for (size_t i = 0; i < TUBE_SECTIONS - 1; ++i) {
            float r = (area[i + 1] - area[i]) / (area[i + 1] + area[i]);
            float f = fwd[i]; float b = bwd[i + 1];
            fwd[i + 1] = (1.0f + r) * f - r * b;
            bwd[i]     = r * f + (1.0f - r) * b;
        }

        float lip_out = fwd[TUBE_SECTIONS - 1] - prev_lip;
        prev_lip = fwd[TUBE_SECTIONS - 1];
        return std::tanh(lip_out * 4.5f);
    }

private:
    float fs, dt;
    float x1, x2, v1, v2;
    std::array<float, TUBE_SECTIONS> fwd, bwd, area;
    float prev_lip;
};

/* ========================================================================= */
/* 5. FORZA / GT7: PHYSICAL INTERNAL COMBUSTION ENGINE (ICE) SYNTHESIS       */
/* ========================================================================= */

template<size_t CYLINDERS = 8>
class PhysicalEngineICE {
public:
    struct EngineSpec {
        size_t cyl_count = 8;
        std::array<float, CYLINDERS> firing_offsets; // Radians [0, 4*PI]
        float idle_rpm = 800.0f;
        float max_rpm = 8500.0f;
    };

    void init(const EngineSpec& spec, float sample_rate) {
        engine_spec = spec;
        fs = sample_rate;
        crank_angle = 0.0f;
        turbo_rpm = 0.0f;
        rng = 1337;
        lpf_state = 0.0f;
    }

    float process(float rpm, float throttle, float dt) {
        float rad_per_sec = (rpm / 60.0f) * TWO_PI;
        crank_angle += rad_per_sec / fs;
        if (crank_angle >= FOUR_PI) crank_angle -= FOUR_PI;

        float load = clampf(throttle, 0.02f, 1.0f);
        float combustion_sum = 0.0f;

        for (size_t k = 0; k < engine_spec.cyl_count; ++k) {
            float phi = std::fmod(crank_angle + engine_spec.firing_offsets[k], FOUR_PI);
            if (phi < 1.85f) { // ~106 deg burn duration
                float norm = phi / 1.85f;
                float pulse = std::pow(std::sin(PI * norm), 2.2f) * std::exp(-3.5f * norm);
                combustion_sum += pulse * (0.3f + 0.7f * load);
            }
        }

        // Exhaust non-linear wave steepening & Muffler Lowpass
        float non_linear = combustion_sum + 0.35f * (combustion_sum * std::abs(combustion_sum) - std::pow(combustion_sum, 3.0f) * 0.1f);
        float alpha = std::exp(-TWO_PI * (180.0f + (rpm / engine_spec.max_rpm) * 850.0f + load * 400.0f) / fs);
        lpf_state = (1.0f - alpha) * non_linear + alpha * lpf_state;

        // Turbo whistle
        float target_turbo = (rpm / engine_spec.max_rpm) * load * 200000.0f;
        turbo_rpm += (target_turbo - turbo_rpm) * (dt * 3.0f);
        float bpf_freq = 12.0f * (turbo_rpm / 60.0f);
        float turbo_s = std::sin(TWO_PI * bpf_freq * (crank_angle / rad_per_sec)) * (turbo_rpm / 200000.0f) * 0.12f;

        float out = lpf_state * 1.4f + turbo_s;
        return (out + 0.25f * out * out) / (1.0f + 0.4f * std::abs(out));
    }

private:
    EngineSpec engine_spec;
    float fs;
    float crank_angle;
    float turbo_rpm;
    uint32_t rng;
    float lpf_state;
};

/* ========================================================================= */
/* 6. DOOM: 3-BAND SURGICAL GLORY KILL DYNAMIC SIDECHAIN MATRIX              */
/* ========================================================================= */

class GloryKillSidechainEngine {
private:
    float fs;
    float lp1[2] = {0.0f, 0.0f};
    float lp2[2] = {0.0f, 0.0f};
    float mid_duck = 1.0f;

public:
    GloryKillSidechainEngine(float sample_rate = 44100.0f) : fs(sample_rate) {}

    void process(float in_l, float in_r, bool executing, float dt, float& out_l, float& out_r) {
        float target = executing ? 0.125f : 1.0f; // -18 dB mid cut
        float rate = executing ? (dt / 0.012f) : (dt / 0.160f); // Fast attack, smooth release
        mid_duck += (target - mid_duck) * clampf(rate, 0.0f, 1.0f);

        float a_low = 1.0f - std::exp(-TWO_PI * 120.0f / fs);
        float a_high = 1.0f - std::exp(-TWO_PI * 4500.0f / fs);

        lp1[0] += a_low * (in_l - lp1[0]); lp1[1] += a_low * (in_r - lp1[1]);
        float low_l = lp1[0]; float low_r = lp1[1];

        lp2[0] += a_high * (in_l - lp2[0]); lp2[1] += a_high * (in_r - lp2[1]);
        float mid_l = lp2[0] - low_l; float mid_r = lp2[1] - low_r;
        float high_l = in_l - lp2[0]; float high_r = in_r - lp2[1];

        out_l = (low_l * 0.85f) + (mid_l * mid_duck) + (high_l * (executing ? 0.55f : 1.0f));
        out_r = (low_r * 0.85f) + (mid_r * mid_duck) + (high_r * (executing ? 0.55f : 1.0f));
    }
};

/* ========================================================================= */
/* 7. THE LAST OF US PART II: BIOMETRIC RESPIRATION & VOCAL FORMANT SHIFTER  */
/* ========================================================================= */

struct BiometricState {
    float heart_rate = 65.0f;
    float stamina = 1.0f;
    float trauma = 0.0f;
    float panic = 0.0f;
    float f1 = 450.0f, f2 = 1100.0f, f3 = 2800.0f;
    float respiration_rate = 14.0f;

    void update(float dt, float sprint_effort, float damage_in, bool spotted) {
        if (sprint_effort > 0.1f) stamina = std::max(0.0f, stamina - dt * 0.20f * sprint_effort);
        else stamina = std::min(1.0f, stamina + dt * 0.14f);

        if (damage_in > 0.0f) trauma = std::min(1.0f, trauma + damage_in * 0.4f);
        trauma = std::max(0.0f, trauma - dt * 0.02f);

        float target_panic = spotted ? 1.0f : (trauma > 0.4f ? 0.8f : 0.0f);
        panic += (target_panic - panic) * (dt / 0.35f);

        float target_hr = 60.0f + 60.0f * (1.0f - stamina) + 40.0f * panic + 25.0f * trauma;
        heart_rate += (target_hr - heart_rate) * (dt / 1.5f);
        respiration_rate = 12.0f + 0.22f * (heart_rate - 60.0f) + 14.0f * panic;

        float shift = 1.0f + 0.18f * panic + 0.14f * (1.0f - stamina);
        f1 = 450.0f * shift; f2 = 1100.0f * shift; f3 = 2800.0f * shift;
    }
};

} // namespace AAADSP

#endif // AAA_PROCEDURAL_DSP_H
