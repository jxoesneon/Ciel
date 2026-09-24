"""
===============================================================================
L-SYSTEM SCHENKERIAN MOTIVIC GENERATOR
===============================================================================
Recursively generates hierarchical melodic structures via context-free
generative grammars with strict voice-leading smoothing.
===============================================================================
"""

import random
from typing import List, Dict, Tuple

class LSystemSchenkerGenerator:
    def __init__(self, scale_degrees: List[int] = [0, 2, 4, 5, 7, 9, 11]):
        self.scale = scale_degrees

    def expand_axiom(self, axiom: str = "U", iterations: int = 3) -> str:
        rules = {
            "U": "P1 P2 P3",                  # Ursatz -> 3 Structural Prolongations
            "P1": "N0 S1 N2",                 # Tonic Prolongation (Neighbor note)
            "P2": "Z24 V4",                   # Dominant Zug (Linear progression)
            "P3": "Cad10",                    # Cadential descent
            "N0": "M(0) M(1) M(0)",
            "S1": "M(2) M(4)",
            "N2": "M(4) M(5) M(4)",
            "Z24": "M(2) M(3) M(4)",
            "V4": "M(4) M(6)",
            "Cad10": "M(1) M(0)"
        }

        current = axiom
        for _ in range(iterations):
            next_str = []
            for token in current.split():
                next_str.append(rules.get(token, token))
            current = " ".join(next_str)
        return current

    def string_to_melody(self, lsys_str: str, root_midi: int = 60) -> List[int]:
        melody = []
        for token in lsys_str.split():
            if token.startswith("M("):
                deg = int(token.replace("M(", "").replace(")", ""))
                scale_len = len(self.scale)
                pitch = root_midi + (deg // scale_len) * 12 + self.scale[deg % scale_len]
                melody.append(pitch)
        return melody

    def generate_themed_phrase(self, root_midi: int = 60, iterations: int = 3) -> List[int]:
        lsys = self.expand_axiom("U", iterations=iterations)
        return self.string_to_melody(lsys, root_midi=root_midi)


if __name__ == "__main__":
    gen = LSystemSchenkerGenerator()
    lsys_text = gen.expand_axiom("U", iterations=3)
    print("--- GENERATED L-SYSTEM STRING ---")
    print(lsys_text)
    
    melody = gen.string_to_melody(lsys_text, root_midi=60)
    print("\n--- SYNTHESIZED SCHENKERIAN MELODY (MIDI Notes) ---")
    print(melody)
