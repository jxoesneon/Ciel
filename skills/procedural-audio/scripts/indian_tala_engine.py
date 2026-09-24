"""
===============================================================================
INDIAN TALA & GENERATIVE TIHAI ENGINE
===============================================================================
Provides authentic time cycles (Avartam, Vibhag, Sam, Khali) and mathematically
solves polyrhythmic Tihai cadences:
    3 * Phrase + 2 * Dum = TargetCycles * TalaLength + 1
===============================================================================
"""

from typing import List, Dict, Optional

class TalaEngine:
    TALAS = {
        "tintal":   {"name": "Tintal",   "beats": 16, "vibhags": [4, 4, 4, 4], "claps": [1, 5, 13], "khali": [9]},
        "jhaptal":  {"name": "Jhaptal",  "beats": 10, "vibhags": [2, 3, 2, 3], "claps": [1, 3, 8],  "khali": [6]},
        "rupak":    {"name": "Rupak",    "beats": 7,  "vibhags": [3, 2, 2],    "claps": [4, 6],     "khali": [1]},
        "ektaal":   {"name": "Ektaal",   "beats": 12, "vibhags": [2, 2, 2, 2, 2, 2], "claps": [1, 5, 9, 11], "khali": [3, 7]},
        "keherwa":  {"name": "Keherwa",  "beats": 8,  "vibhags": [4, 4],       "claps": [1],        "khali": [5]}
    }

    def __init__(self, tala_name: str = "tintal"):
        self.tala = self.TALAS.get(tala_name.lower(), self.TALAS["tintal"])

    def generate_tihai(self, target_cycle: int = 1, preferred_dum: float = 1.0) -> Dict:
        """
        Solves: 3 * Phrase + 2 * Dum = TargetCycles * TalaLength + 1
        """
        L = self.tala["beats"]
        total_target_beats = target_cycle * L + 1

        best_solution = None
        min_dum_diff = 999.0

        for dum_steps in range(0, 16):
            dum = dum_steps * 0.5  # Supports half-beat rests
            remaining = total_target_beats - 2 * dum
            if remaining > 0 and (remaining / 3.0) == round(remaining / 3.0, 4):
                phrase = remaining / 3.0
                diff = abs(dum - preferred_dum)
                if diff < min_dum_diff:
                    min_dum_diff = diff
                    best_solution = {
                        "phrase_beats": phrase,
                        "dum_beats": dum,
                        "total_target_beats": total_target_beats,
                        "tala": self.tala["name"],
                        "start_beat_in_cycle": (L - (int(total_target_beats - 1) % L)) % L + 1
                    }
        return best_solution or {"error": "No integer solution found"}

    def get_tala_grid(self) -> List[str]:
        """Returns step-by-step metric breakdown of current Tala cycle."""
        L = self.tala["beats"]
        grid = []
        for b in range(1, L + 1):
            if b == 1:
                grid.append("SAM(1)")
            elif b in self.tala["claps"]:
                grid.append(f"CLAP({b})")
            elif b in self.tala["khali"]:
                grid.append(f"KHALI({b})")
            else:
                grid.append(f"beat({b})")
        return grid


if __name__ == "__main__":
    for t_name in ["tintal", "jhaptal", "rupak", "ektaal"]:
        t_engine = TalaEngine(t_name)
        print(f"\n--- TALA: {t_engine.tala['name']} (Beats: {t_engine.tala['beats']}) ---")
        print("Metric Grid:", t_engine.get_tala_grid())
        tihai = t_engine.generate_tihai(target_cycle=1, preferred_dum=1.0)
        print("Tihai 1-Cycle Resolution:", tihai)
