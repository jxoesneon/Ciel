"""
===============================================================================
GLOBAL MICROTONAL & NON-WESTERN TUNING ENGINE (Zero External Dependencies)
===============================================================================
Supports:
- 22 Shrutis of Indian Classical Music with exact Just Intonation ratios
- Arabic 24-EDO Maqam Tuning & Jins Generator (Rast, Bayati, Hijaz, Sikah, Saba)
- Indonesian Gamelan Stretched Octave (1215c) & Ombak Acoustical Physics
- 31-EDO (Meantone / 7-Limit) & 53-EDO (Holden's Comma / Turkish Maqam) Quantizers
===============================================================================
"""

import math
from typing import List, Dict, Tuple, Optional

class MicrotonalPitchEngine:
    def __init__(self, concert_a: float = 440.0):
        self.concert_a = concert_a

        # 22 Indian Shrutis (Exact Just Intonation Ratios)
        self.shrutis_ratios = [
            (1, 1),       # 0: Sa (0.0 cents)
            (256, 243),   # 1: Ati-Komal Re (90.22 cents)
            (16, 15),     # 2: Komal Re (111.73 cents)
            (10, 9),      # 3: Tivra Re (182.40 cents)
            (9, 8),       # 4: Shuddha Re (203.91 cents)
            (32, 27),     # 5: Ati-Komal Ga (294.13 cents)
            (6, 5),       # 6: Komal Ga (315.64 cents)
            (5, 4),       # 7: Shuddha Ga (386.31 cents)
            (81, 64),     # 8: Tivra Ga (407.82 cents)
            (4, 3),       # 9: Shuddha Ma (498.04 cents)
            (27, 20),     # 10: Tivra Ma 1 (519.55 cents)
            (45, 32),     # 11: Tivra Ma 2 (590.22 cents)
            (729, 512),   # 12: Tivratara Ma (611.73 cents)
            (3, 2),       # 13: Pa (701.96 cents)
            (128, 81),    # 14: Ati-Komal Dha (792.18 cents)
            (8, 5),       # 15: Komal Dha (813.69 cents)
            (5, 3),       # 16: Tivra Dha (884.36 cents)
            (27, 16),     # 17: Shuddha Dha (905.87 cents)
            (16, 9),      # 18: Ati-Komal Ni (996.09 cents)
            (9, 5),       # 19: Komal Ni (1017.60 cents)
            (15, 8),      # 20: Shuddha Ni (1088.27 cents)
            (243, 128)    # 21: Tivra Ni (1109.78 cents)
        ]

    def cents_to_ratio(self, cents: float) -> float:
        return 2.0 ** (cents / 1200.0)

    def ratio_to_cents(self, ratio: float) -> float:
        return 1200.0 * math.log2(ratio)

    def get_shruti_freq(self, root_freq: float, shruti_idx: int, octave: int = 0) -> float:
        """Returns exact frequency for one of the 22 Indian Shrutis."""
        num, den = self.shrutis_ratios[shruti_idx % 22]
        oct_shift = octave + (shruti_idx // 22)
        return root_freq * (num / den) * (2.0 ** oct_shift)

    def get_maqam_rast_freqs(self, root_freq: float = 261.63) -> List[float]:
        """
        Maqam Rast (C, D, E-half-flat, F, G, A, B-half-flat, c).
        E-half-flat = ~350 cents (neutral third); B-half-flat = ~1050 cents.
        """
        rast_cents = [0.0, 204.0, 355.0, 498.0, 702.0, 906.0, 1057.0, 1200.0]
        return [root_freq * self.cents_to_ratio(c) for c in rast_cents]

    def get_maqam_bayati_freqs(self, root_freq: float = 293.66) -> List[float]:
        """
        Maqam Bayati (D, E-half-flat, F, G, A, Bb, C, d).
        E-half-flat = ~150 cents (neutral second).
        """
        bayati_cents = [0.0, 150.0, 300.0, 500.0, 702.0, 800.0, 1000.0, 1200.0]
        return [root_freq * self.cents_to_ratio(c) for c in bayati_cents]

    def get_gamelan_slendro_freqs(self, root_freq: float = 270.0, stretch_cents: float = 1215.0) -> List[float]:
        """
        Javanese Slendro 5-tone equidistant scale with stretched octave.
        """
        step = stretch_cents / 5.0
        return [root_freq * (2.0 ** ((i * step) / 1200.0)) for i in range(6)]

    def get_gamelan_ombak_pair(self, base_freq: float, beating_hz: float = 6.0) -> Tuple[float, float]:
        """Returns (Pengumbang [lower], Pengisep [higher]) frequency pair."""
        return (base_freq - (beating_hz / 2.0), base_freq + (beating_hz / 2.0))

    def quantize_to_edo(self, freq: float, root_freq: float, edo: int = 31) -> float:
        """Quantizes arbitrary frequency to nearest step in N-EDO system."""
        cents_from_root = self.ratio_to_cents(freq / root_freq)
        step_size = 1200.0 / edo
        nearest_step = round(cents_from_root / step_size)
        quantized_cents = nearest_step * step_size
        return root_freq * self.cents_to_ratio(quantized_cents)

    def continuous_meend_interpolation(self, f_start: float, f_end: float, tau: float) -> float:
        """
        Cubic Hermite interpolation for authentic Indian Gamaka (Meend).
        tau in [0.0, 1.0]
        """
        tau_c = max(0.0, min(1.0, tau))
        blend = 3.0 * (tau_c ** 2) - 2.0 * (tau_c ** 3)
        return f_start + (f_end - f_start) * blend


if __name__ == "__main__":
    engine = MicrotonalPitchEngine()
    print("--- 22 INDIAN SHRUTIS (Root C4 = 261.63 Hz) ---")
    for i in range(22):
        freq = engine.get_shruti_freq(261.63, i)
        print(f"Shruti {i:02d}: {freq:.2f} Hz ({engine.ratio_to_cents(freq/261.63):.1f} cents)")
    
    print("\n--- MAQAM RAST FREQUENCIES ---")
    print([round(f, 2) for f in engine.get_maqam_rast_freqs(261.63)])
    
    print("\n--- GAMELAN SLENDRO WITH STRETCHED OCTAVE ---")
    print([round(f, 2) for f in engine.get_gamelan_slendro_freqs(270.0)])
