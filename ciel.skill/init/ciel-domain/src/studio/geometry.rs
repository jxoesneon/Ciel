//! Port of geometry_core.py — DSU, 3D integer spatial hashing, buffered OBJ
//! parser, and geometric area/bbox utilities.

use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub struct DisjointSetUnion {
    parent: Vec<usize>,
    rank: Vec<usize>,
    pub count: usize,
}

impl DisjointSetUnion {
    pub fn new(size: usize) -> Self {
        DisjointSetUnion {
            parent: (0..size).collect(),
            rank: vec![0; size],
            count: size,
        }
    }
    pub fn find(&mut self, i: usize) -> usize {
        let mut root = i;
        while root != self.parent[root] {
            root = self.parent[root];
        }
        let mut curr = i;
        while curr != root {
            let nxt = self.parent[curr];
            self.parent[curr] = root;
            curr = nxt;
        }
        root
    }
    pub fn union(&mut self, i: usize, j: usize) -> bool {
        let ri = self.find(i);
        let rj = self.find(j);
        if ri != rj {
            if self.rank[ri] < self.rank[rj] {
                self.parent[ri] = rj;
            } else if self.rank[ri] > self.rank[rj] {
                self.parent[rj] = ri;
            } else {
                self.parent[rj] = ri;
                self.rank[ri] += 1;
            }
            self.count -= 1;
            return true;
        }
        false
    }
}

/// Large prime integer hash for 3D grid cell coordinates.
pub fn hash_grid_3d(gx: i64, gy: i64, gz: i64) -> u64 {
    let p1: i64 = 73856093;
    let p2: i64 = 19349663;
    let p3: i64 = 83492791;
    ((gx.wrapping_mul(p1) ^ gy.wrapping_mul(p2) ^ gz.wrapping_mul(p3)) & 0x7FFFFFFF) as u64
}

pub type Vec3 = [f64; 3];
pub type Vec2 = [f64; 2];

#[derive(Default)]
pub struct MeshData {
    pub vertices: Vec<Vec3>,
    pub normals: Vec<Vec3>,
    pub texcoords: Vec<Vec2>,
    pub faces: Vec<Vec<usize>>,
    /// per-face texcoord indices; `usize::MAX` encodes Python `None`
    pub face_uvs: Vec<Vec<Option<usize>>>,
    pub face_normals: Vec<Vec<Option<usize>>>,
}

fn parse_index(tok: &str) -> Option<i64> {
    if tok.is_empty() {
        return None;
    }
    tok.parse::<i64>().ok()
}

/// High-speed OBJ parser with a vertex count ceiling (MemoryError twin).
pub fn parse_obj_buffered(filepath: &str, max_vertices: usize) -> Result<MeshData, String> {
    let file = File::open(filepath).map_err(|e| format!("{}: {}", filepath, e))?;
    let reader = BufReader::with_capacity(1 << 20, file);
    let mut mesh = MeshData::default();

    for line in reader.lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => continue, // errors='ignore'
        };
        if line.is_empty()
            || line.starts_with('#')
            || line.starts_with('\r')
            || line.starts_with('\n')
        {
            continue;
        }
        let tokens: Vec<&str> = line.split_whitespace().collect();
        if tokens.is_empty() {
            continue;
        }
        match tokens[0] {
            "v" => {
                if mesh.vertices.len() >= max_vertices {
                    return Err(format!(
                        "Vertex count exceeds pure Rust limit ({max_vertices}). Delegate to Blender headless engine."
                    ));
                }
                if tokens.len() >= 4 {
                    mesh.vertices.push([
                        tokens[1].parse().unwrap_or(0.0),
                        tokens[2].parse().unwrap_or(0.0),
                        tokens[3].parse().unwrap_or(0.0),
                    ]);
                }
            }
            "vt" => {
                if tokens.len() >= 2 {
                    let u: f64 = tokens[1].parse().unwrap_or(0.0);
                    let v: f64 = if tokens.len() > 2 {
                        tokens[2].parse().unwrap_or(0.0)
                    } else {
                        0.0
                    };
                    mesh.texcoords.push([u, v]);
                }
            }
            "vn" => {
                if tokens.len() >= 4 {
                    mesh.normals.push([
                        tokens[1].parse().unwrap_or(0.0),
                        tokens[2].parse().unwrap_or(0.0),
                        tokens[3].parse().unwrap_or(0.0),
                    ]);
                }
            }
            "f" => {
                let mut f_v = Vec::new();
                let mut f_vt = Vec::new();
                let mut f_vn = Vec::new();
                for p in &tokens[1..] {
                    let parts: Vec<&str> = p.split('/').collect();
                    let mut v_idx = parse_index(parts[0]).map(|x| x - 1).unwrap_or(0);
                    let mut vt_idx = if parts.len() > 1 {
                        parse_index(parts[1]).map(|x| x - 1)
                    } else {
                        None
                    };
                    let mut vn_idx = if parts.len() > 2 {
                        parse_index(parts[2]).map(|x| x - 1)
                    } else {
                        None
                    };
                    // negative indices are relative to current array length
                    if v_idx < 0 {
                        v_idx = mesh.vertices.len() as i64 + v_idx + 1;
                    }
                    if let Some(vt) = vt_idx {
                        if vt < 0 {
                            vt_idx = Some(mesh.texcoords.len() as i64 + vt + 1);
                        }
                    }
                    if let Some(vn) = vn_idx {
                        if vn < 0 {
                            vn_idx = Some(mesh.normals.len() as i64 + vn + 1);
                        }
                    }
                    f_v.push(v_idx.max(0) as usize);
                    f_vt.push(vt_idx.map(|x| x.max(0) as usize));
                    f_vn.push(vn_idx.map(|x| x.max(0) as usize));
                }
                mesh.faces.push(f_v);
                mesh.face_uvs.push(f_vt);
                mesh.face_normals.push(f_vn);
            }
            _ => {}
        }
    }
    Ok(mesh)
}

pub fn compute_triangle_area_3d(v0: Vec3, v1: Vec3, v2: Vec3) -> f64 {
    let (ax, ay, az) = (v1[0] - v0[0], v1[1] - v0[1], v1[2] - v0[2]);
    let (bx, by, bz) = (v2[0] - v0[0], v2[1] - v0[1], v2[2] - v0[2]);
    let cx = ay * bz - az * by;
    let cy = az * bx - ax * bz;
    let cz = ax * by - ay * bx;
    0.5 * (cx * cx + cy * cy + cz * cz).sqrt()
}

pub fn compute_polygon_area_3d(pts: &[Vec3]) -> f64 {
    if pts.len() < 3 {
        return 0.0;
    }
    let v0 = pts[0];
    (1..pts.len() - 1)
        .map(|i| compute_triangle_area_3d(v0, pts[i], pts[i + 1]))
        .sum()
}

/// Single-pass min/max extents → (min, max, dimensions).
pub fn compute_single_pass_bounding_box(vertices: &[Vec3]) -> (Vec3, Vec3, Vec3) {
    if vertices.is_empty() {
        return ([0.0; 3], [0.0; 3], [0.0; 3]);
    }
    let v0 = vertices[0];
    let (mut min_x, mut max_x) = (v0[0], v0[0]);
    let (mut min_y, mut max_y) = (v0[1], v0[1]);
    let (mut min_z, mut max_z) = (v0[2], v0[2]);
    for v in &vertices[1..] {
        let (x, y, z) = (v[0], v[1], v[2]);
        if x < min_x {
            min_x = x;
        } else if x > max_x {
            max_x = x;
        }
        if y < min_y {
            min_y = y;
        } else if y > max_y {
            max_y = y;
        }
        if z < min_z {
            min_z = z;
        } else if z > max_z {
            max_z = z;
        }
    }
    (
        [min_x, min_y, min_z],
        [max_x, max_y, max_z],
        [max_x - min_x, max_y - min_y, max_z - min_z],
    )
}

/// Grid index for a coordinate — Python `int()` truncates toward zero.
pub fn grid_coord(v: f64, origin: f64, cell_size: f64) -> i64 {
    if cell_size > 0.0 {
        ((v - origin) / cell_size).trunc() as i64
    } else {
        0
    }
}

pub fn grid_map_3d(vertices: &[Vec3], min_pt: Vec3, cell_size: f64) -> HashMap<u64, Vec<usize>> {
    let mut map: HashMap<u64, Vec<usize>> = HashMap::new();
    for (vi, v) in vertices.iter().enumerate() {
        let gx = grid_coord(v[0], min_pt[0], cell_size);
        let gy = grid_coord(v[1], min_pt[1], cell_size);
        let gz = grid_coord(v[2], min_pt[2], cell_size);
        map.entry(hash_grid_3d(gx, gy, gz)).or_default().push(vi);
    }
    map
}
