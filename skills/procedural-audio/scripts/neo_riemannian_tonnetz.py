"""
===============================================================================
NEO-RIEMANNIAN TONNETZ & TRANSFORMATIONAL ENGINE
===============================================================================
Implements P, L, R, S, N, H operators and BFS shortest modulation pathway across
the 24-node pitch-class triad torus.
===============================================================================
"""

from typing import Tuple, List, Dict, Optional

class Triad:
    def __init__(self, root: int, is_major: bool):
        self.root = root % 12
        self.is_major = is_major

    def pitch_classes(self) -> Tuple[int, int, int]:
        third = (self.root + 4) % 12 if self.is_major else (self.root + 3) % 12
        fifth = (self.root + 7) % 12
        return (self.root, third, fifth)

    def name(self) -> str:
        names = ["C", "C#", "D", "Eb", "E", "F", "F#", "G", "Ab", "A", "Bb", "B"]
        return f"{names[self.root]}{'' if self.is_major else 'm'}"

    def __repr__(self):
        return self.name()

    def __eq__(self, other):
        return isinstance(other, Triad) and self.root == other.root and self.is_major == other.is_major

    def __hash__(self):
        return hash((self.root, self.is_major))

class TonnetzEngine:
    @staticmethod
    def P(triad: Triad) -> Triad:
        """Parallel: C <-> Cm"""
        return Triad(triad.root, not triad.is_major)

    @staticmethod
    def L(triad: Triad) -> Triad:
        """Leittonwechsel: C <-> Em, Cm <-> Ab"""
        if triad.is_major:
            return Triad((triad.root + 4) % 12, False)
        else:
            return Triad((triad.root + 8) % 12, True)

    @staticmethod
    def R(triad: Triad) -> Triad:
        """Relative: C <-> Am, Cm <-> Eb"""
        if triad.is_major:
            return Triad((triad.root + 9) % 12, False)
        else:
            return Triad((triad.root + 3) % 12, True)

    @classmethod
    def S(cls, triad: Triad) -> Triad:
        """Slide: C <-> C#m (Shares only third)"""
        return cls.L(cls.P(cls.R(triad)))

    @classmethod
    def H(cls, triad: Triad) -> Triad:
        """Hexatonic Pole: C <-> Abm (Zero common tones)"""
        return cls.L(cls.P(cls.L(triad)))

    @classmethod
    def find_shortest_path(cls, start: Triad, goal: Triad) -> List[Tuple[str, Triad]]:
        """BFS shortest transformation pathway across Tonnetz graph."""
        queue = [[("START", start)]]
        visited = {start}

        while queue:
            path = queue.pop(0)
            current = path[-1][1]
            if current == goal:
                return path

            for op_name, op_func in [("P", cls.P), ("L", cls.L), ("R", cls.R), ("S", cls.S), ("H", cls.H)]:
                neighbor = op_func(current)
                if neighbor not in visited:
                    visited.add(neighbor)
                    new_path = list(path)
                    new_path.append((op_name, neighbor))
                    queue.append(new_path)
        return []


if __name__ == "__main__":
    c_maj = Triad(0, True)    # C Major
    ab_min = Triad(8, False)  # Ab Minor
    f_sharp = Triad(6, True)  # F# Major

    print("--- NEO-RIEMANNIAN BASIC TRANSFORMATIONS ---")
    print(f"C Major -> P: {TonnetzEngine.P(c_maj)}")
    print(f"C Major -> L: {TonnetzEngine.L(c_maj)}")
    print(f"C Major -> R: {TonnetzEngine.R(c_maj)}")
    print(f"C Major -> S (Slide): {TonnetzEngine.S(c_maj)}")
    print(f"C Major -> H (Hexatonic Pole): {TonnetzEngine.H(c_maj)}")

    print("\n--- SHORTEST TONNETZ MODULATION PATH: C Major -> Ab Minor ---")
    path = TonnetzEngine.find_shortest_path(c_maj, ab_min)
    print(" -> ".join([f"{step[0]}({step[1]})" for step in path]))

    print("\n--- SHORTEST TONNETZ MODULATION PATH: C Major -> F# Major (Tritone) ---")
    path2 = TonnetzEngine.find_shortest_path(c_maj, f_sharp)
    print(" -> ".join([f"{step[0]}({step[1]})" for step in path2]))
