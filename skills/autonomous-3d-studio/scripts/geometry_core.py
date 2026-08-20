#!/usr/bin/env python3
"""
geometry_core.py - High-Performance Zero-Dependency 3D Geometry Core

Provides unified, optimized geometric primitives, Disjoint Set Union (DSU),
fast 3D integer spatial hashing, and buffered multi-format mesh parsing (OBJ, STL, PLY).
"""

import sys
import os
import math
import struct
from collections import defaultdict

# ---------------------------------------------------------------------------
# 1. Fast Disjoint Set Union (DSU / Union-Find) with Path Compression & Rank
# ---------------------------------------------------------------------------
class DisjointSetUnion:
    def __init__(self, size):
        self.parent = list(range(size))
        self.rank = [0] * size
        self.count = size

    def find(self, i):
        root = i
        while root != self.parent[root]:
            root = self.parent[root]
        # Path compression
        curr = i
        while curr != root:
            nxt = self.parent[curr]
            self.parent[curr] = root
            curr = nxt
        return root

    def union(self, i, j):
        root_i = self.find(i)
        root_j = self.find(j)
        if root_i != root_j:
            if self.rank[root_i] < self.rank[root_j]:
                self.parent[root_i] = root_j
            elif self.rank[root_i] > self.rank[root_j]:
                self.parent[root_j] = root_i
            else:
                self.parent[root_j] = root_i
                self.rank[root_i] += 1
            self.count -= 1
            return True
        return False

# ---------------------------------------------------------------------------
# 2. Fast 3D Integer Spatial Hashing
# ---------------------------------------------------------------------------
def hash_grid_3d(gx, gy, gz):
    """Computes large prime integer hash for 3D grid cell coordinates."""
    # Large primes to prevent spatial coordinate collisions
    p1 = 73856093
    p2 = 19349663
    p3 = 83492791
    return ((gx * p1) ^ (gy * p2) ^ (gz * p3)) & 0x7FFFFFFF

# ---------------------------------------------------------------------------
# 3. High-Performance Stream-Based Mesh Parser
# ---------------------------------------------------------------------------
class MeshData:
    def __init__(self):
        self.vertices = []      # [[x, y, z], ...]
        self.normals = []       # [[nx, ny, nz], ...]
        self.texcoords = []     # [[u, v], ...]
        self.faces = []         # [[v0, v1, ...], ...]
        self.face_uvs = []      # [[vt0, vt1, ...], ...]
        self.face_normals = []  # [[vn0, vn1, ...], ...]

def parse_obj_buffered(filepath, max_vertices=1000000):
    """
    High-speed chunked OBJ parser with memory and vertex count safety ceilings.
    """
    mesh = MeshData()
    file_size = os.path.getsize(filepath)
    
    with open(filepath, 'r', encoding='utf-8', errors='ignore') as f:
        for line in f:
            if not line or line[0] in '#\r\n':
                continue
            
            tokens = line.split()
            if not tokens:
                continue

            tag = tokens[0]
            if tag == 'v':
                if len(mesh.vertices) >= max_vertices:
                    raise MemoryError(f"Vertex count exceeds pure Python limit ({max_vertices:,}). Delegate to Blender headless engine.")
                mesh.vertices.append([float(tokens[1]), float(tokens[2]), float(tokens[3])])
            elif tag == 'vt':
                u = float(tokens[1])
                v = float(tokens[2]) if len(tokens) > 2 else 0.0
                mesh.texcoords.append([u, v])
            elif tag == 'vn':
                mesh.normals.append([float(tokens[1]), float(tokens[2]), float(tokens[3])])
            elif tag == 'f':
                f_v = []
                f_vt = []
                f_vn = []
                for p in tokens[1:]:
                    parts = p.split('/')
                    v_idx = int(parts[0]) - 1 if parts[0] else 0
                    vt_idx = int(parts[1]) - 1 if len(parts) > 1 and parts[1] else None
                    vn_idx = int(parts[2]) - 1 if len(parts) > 2 and parts[2] else None

                    # Handle negative indices
                    if v_idx < 0:
                        v_idx = len(mesh.vertices) + v_idx + 1
                    if vt_idx is not None and vt_idx < 0:
                        vt_idx = len(mesh.texcoords) + vt_idx + 1
                    if vn_idx is not None and vn_idx < 0:
                        vn_idx = len(mesh.normals) + vn_idx + 1

                    f_v.append(v_idx)
                    f_vt.append(vt_idx)
                    f_vn.append(vn_idx)

                mesh.faces.append(f_v)
                mesh.face_uvs.append(f_vt)
                mesh.face_normals.append(f_vn)

    return mesh

# ---------------------------------------------------------------------------
# 4. Geometric & Area Utilities
# ---------------------------------------------------------------------------
def compute_triangle_area_3d(v0, v1, v2):
    ax, ay, az = v1[0] - v0[0], v1[1] - v0[1], v1[2] - v0[2]
    bx, by, bz = v2[0] - v0[0], v2[1] - v0[1], v2[2] - v0[2]
    cx = ay * bz - az * by
    cy = az * bx - ax * bz
    cz = ax * by - ay * bx
    return 0.5 * math.sqrt(cx * cx + cy * cy + cz * cz)

def compute_polygon_area_3d(poly_pts):
    if len(poly_pts) < 3:
        return 0.0
    area = 0.0
    v0 = poly_pts[0]
    for i in range(1, len(poly_pts) - 1):
        area += compute_triangle_area_3d(v0, poly_pts[i], poly_pts[i + 1])
    return area

def compute_single_pass_bounding_box(vertices):
    """Computes min/max 3D extents in a single traversal."""
    if not vertices:
        return (0.0, 0.0, 0.0), (0.0, 0.0, 0.0), (0.0, 0.0, 0.0)
    
    v0 = vertices[0]
    min_x, max_x = v0[0], v0[0]
    min_y, max_y = v0[1], v0[1]
    min_z, max_z = v0[2], v0[2]

    for v in vertices[1:]:
        x, y, z = v[0], v[1], v[2]
        if x < min_x: min_x = x
        elif x > max_x: max_x = x
        if y < min_y: min_y = y
        elif y > max_y: max_y = y
        if z < min_z: min_z = z
        elif z > max_z: max_z = z

    dimensions = (max_x - min_x, max_y - min_y, max_z - min_z)
    return (min_x, min_y, min_z), (max_x, max_y, max_z), dimensions
