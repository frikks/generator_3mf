//! Indexed triangle mesh with welding, T-junction repair and checks.

use std::collections::HashMap;

pub type P3 = [f64; 3];

#[derive(Clone, Debug, Default)]
pub struct Mesh {
    pub vertices: Vec<P3>,
    pub triangles: Vec<[u32; 3]>,
}

fn sub(a: P3, b: P3) -> P3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}
fn cross(a: P3, b: P3) -> P3 {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}
fn dot(a: P3, b: P3) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

pub fn triangle_normal(a: P3, b: P3, c: P3) -> P3 {
    let n = cross(sub(b, a), sub(c, a));
    let l = dot(n, n).sqrt();
    if l > 0.0 {
        [n[0] / l, n[1] / l, n[2] / l]
    } else {
        [0.0, 0.0, 1.0]
    }
}

/// Builds a mesh from loose triangles, merging coincident vertices.
pub struct MeshBuilder {
    tol: f64,
    cell: f64,
    grid: HashMap<(i64, i64, i64), Vec<u32>>,
    mesh: Mesh,
}

impl MeshBuilder {
    pub fn new() -> Self {
        Self {
            tol: 1e-5,
            cell: 1e-3,
            grid: HashMap::new(),
            mesh: Mesh::default(),
        }
    }

    fn key(&self, p: P3) -> (i64, i64, i64) {
        (
            (p[0] / self.cell).floor() as i64,
            (p[1] / self.cell).floor() as i64,
            (p[2] / self.cell).floor() as i64,
        )
    }

    pub fn vertex(&mut self, p: P3) -> u32 {
        let k = self.key(p);
        for dx in -1..=1 {
            for dy in -1..=1 {
                for dz in -1..=1 {
                    if let Some(list) = self.grid.get(&(k.0 + dx, k.1 + dy, k.2 + dz)) {
                        for &i in list {
                            let q = self.mesh.vertices[i as usize];
                            let d = sub(p, q);
                            if dot(d, d) <= self.tol * self.tol {
                                return i;
                            }
                        }
                    }
                }
            }
        }
        let i = self.mesh.vertices.len() as u32;
        self.mesh.vertices.push(p);
        self.grid.entry(k).or_default().push(i);
        i
    }

    pub fn triangle(&mut self, a: P3, b: P3, c: P3) {
        let ia = self.vertex(a);
        let ib = self.vertex(b);
        let ic = self.vertex(c);
        if ia != ib && ib != ic && ia != ic {
            self.mesh.triangles.push([ia, ib, ic]);
        }
    }

    pub fn finish(self) -> Mesh {
        let mut m = self.mesh;
        fix_t_junctions(&mut m);
        m.remove_degenerate();
        m
    }
}

impl Default for MeshBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Splits horizontal edges that have other vertices lying on them.
///
/// The solid builder joins walls and caps only along horizontal edges, so
/// checking edges whose endpoints share the same height is sufficient.
fn fix_t_junctions(m: &mut Mesh) {
    const EPS: f64 = 1e-6;
    // Group vertices by height level.
    let mut levels: HashMap<i64, Vec<u32>> = HashMap::new();
    let lkey = |z: f64| (z * 1e4).round() as i64;
    for (i, v) in m.vertices.iter().enumerate() {
        levels.entry(lkey(v[2])).or_default().push(i as u32);
    }
    // Spatial grid per level.
    let cell = 1.0;
    let mut grids: HashMap<i64, HashMap<(i64, i64), Vec<u32>>> = HashMap::new();
    for (lk, list) in &levels {
        let g = grids.entry(*lk).or_default();
        for &i in list {
            let v = m.vertices[i as usize];
            g.entry(((v[0] / cell).floor() as i64, (v[1] / cell).floor() as i64))
                .or_default()
                .push(i);
        }
    }
    let mut out = Vec::with_capacity(m.triangles.len());
    let mut changed = false;
    for t in &m.triangles {
        let mut extra: [Vec<(f64, u32)>; 3] = [Vec::new(), Vec::new(), Vec::new()];
        let mut any = false;
        for e in 0..3 {
            let ia = t[e];
            let ib = t[(e + 1) % 3];
            let a = m.vertices[ia as usize];
            let b = m.vertices[ib as usize];
            if (a[2] - b[2]).abs() > 1e-7 {
                continue;
            }
            let Some(g) = grids.get(&lkey(a[2])) else {
                continue;
            };
            let dx = b[0] - a[0];
            let dy = b[1] - a[1];
            let len2 = dx * dx + dy * dy;
            if len2 < 1e-12 {
                continue;
            }
            let len = len2.sqrt();
            let x0 = ((a[0].min(b[0]) - EPS) / cell).floor() as i64;
            let x1 = ((a[0].max(b[0]) + EPS) / cell).floor() as i64;
            let y0 = ((a[1].min(b[1]) - EPS) / cell).floor() as i64;
            let y1 = ((a[1].max(b[1]) + EPS) / cell).floor() as i64;
            for gx in x0..=x1 {
                for gy in y0..=y1 {
                    let Some(list) = g.get(&(gx, gy)) else {
                        continue;
                    };
                    for &ip in list {
                        if ip == ia || ip == ib {
                            continue;
                        }
                        let p = m.vertices[ip as usize];
                        if (p[2] - a[2]).abs() > 1e-7 {
                            continue;
                        }
                        let px = p[0] - a[0];
                        let py = p[1] - a[1];
                        let tpar = (px * dx + py * dy) / len2;
                        if tpar * len <= EPS || (1.0 - tpar) * len <= EPS {
                            continue;
                        }
                        let dist = (px * dy - py * dx).abs() / len;
                        if dist <= EPS {
                            extra[e].push((tpar, ip));
                            any = true;
                        }
                    }
                }
            }
        }
        if !any {
            out.push(*t);
            continue;
        }
        changed = true;
        let mut ring: Vec<u32> = Vec::new();
        let mut edges_with_points = 0;
        for e in 0..3 {
            ring.push(t[e]);
            if !extra[e].is_empty() {
                edges_with_points += 1;
            }
            extra[e].sort_by(|x, y| x.0.partial_cmp(&y.0).unwrap());
            extra[e].dedup_by_key(|x| x.1);
            for &(_, ip) in &extra[e] {
                ring.push(ip);
            }
        }
        if edges_with_points == 1 {
            // Fan from the vertex opposite the split edge.
            let e = (0..3).find(|&e| !extra[e].is_empty()).unwrap();
            let apex = t[(e + 2) % 3];
            let mut chain = vec![t[e]];
            chain.extend(extra[e].iter().map(|x| x.1));
            chain.push(t[(e + 1) % 3]);
            for w in chain.windows(2) {
                out.push([w[0], w[1], apex]);
            }
        } else {
            // Fan from a new centroid vertex.
            let a = m.vertices[t[0] as usize];
            let b = m.vertices[t[1] as usize];
            let c = m.vertices[t[2] as usize];
            let cen = [
                (a[0] + b[0] + c[0]) / 3.0,
                (a[1] + b[1] + c[1]) / 3.0,
                (a[2] + b[2] + c[2]) / 3.0,
            ];
            let ic = m.vertices.len() as u32;
            m.vertices.push(cen);
            for i in 0..ring.len() {
                out.push([ring[i], ring[(i + 1) % ring.len()], ic]);
            }
        }
    }
    if changed {
        m.triangles = out;
    }
}

impl Mesh {
    pub fn remove_degenerate(&mut self) {
        let v = &self.vertices;
        self.triangles.retain(|t| {
            if t[0] == t[1] || t[1] == t[2] || t[0] == t[2] {
                return false;
            }
            let n = cross(
                sub(v[t[1] as usize], v[t[0] as usize]),
                sub(v[t[2] as usize], v[t[0] as usize]),
            );
            dot(n, n) > 1e-18
        });
    }

    pub fn bounds(&self) -> (P3, P3) {
        let mut lo = [f64::MAX; 3];
        let mut hi = [f64::MIN; 3];
        for v in &self.vertices {
            for k in 0..3 {
                lo[k] = lo[k].min(v[k]);
                hi[k] = hi[k].max(v[k]);
            }
        }
        if self.vertices.is_empty() {
            return ([0.0; 3], [0.0; 3]);
        }
        (lo, hi)
    }

    /// Signed volume in mm³ (positive for outward-facing triangles).
    pub fn volume(&self) -> f64 {
        let mut s = 0.0;
        for t in &self.triangles {
            let a = self.vertices[t[0] as usize];
            let b = self.vertices[t[1] as usize];
            let c = self.vertices[t[2] as usize];
            s += dot(a, cross(b, c));
        }
        s / 6.0
    }

    /// Number of directed edges without a matching opposite edge.
    /// Zero means the mesh is closed and consistently oriented.
    pub fn open_edges(&self) -> usize {
        let mut map: HashMap<(u32, u32), i32> = HashMap::new();
        for t in &self.triangles {
            for e in 0..3 {
                let a = t[e];
                let b = t[(e + 1) % 3];
                *map.entry((a, b)).or_insert(0) += 1;
            }
        }
        let mut bad = 0;
        for (&(a, b), &n) in &map {
            let back = map.get(&(b, a)).copied().unwrap_or(0);
            if n != 1 || back != 1 {
                bad += 1;
            }
        }
        bad
    }

    pub fn translate(&mut self, d: P3) {
        for v in &mut self.vertices {
            v[0] += d[0];
            v[1] += d[1];
            v[2] += d[2];
        }
    }
}
