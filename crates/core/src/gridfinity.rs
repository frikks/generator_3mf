//! Gridfinity parts on the open 42 mm × 7 mm standard.

use crate::mesh::Mesh;
use crate::solid::{build, circle, rounded_rect, Ring, Slab, P2};

pub const PITCH: f64 = 42.0;
pub const HEIGHT_UNIT: f64 = 7.0;
const CLEARANCE: f64 = 0.5;
const BIN_RADIUS: f64 = 3.75;
const PLATE_RADIUS: f64 = 4.0;
const FOOT_HEIGHT: f64 = 4.75;
const FLOOR_TOP: f64 = 7.0;
const MAGNET_D: f64 = 6.5;
const MAGNET_DEPTH: f64 = 2.4;
const MAGNET_OFFSET: f64 = 13.0;
const SCREW_D: f64 = 3.0;
const LIP_HEIGHT: f64 = 3.8;
const LABEL_DEPTH: f64 = 12.0;
const LABEL_THICKNESS: f64 = 1.2;
const SEG: usize = 8;
const CIRCLE_SEG: usize = 32;
const SCOOP_STEPS: usize = 10;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Bin,
    Baseplate,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Params {
    pub kind: Kind,
    pub units_x: u32,
    pub units_y: u32,
    pub height_u: u32,
    pub div_x: u32,
    pub div_y: u32,
    pub lip: bool,
    pub magnets: bool,
    pub scoop: bool,
    pub label: bool,
    pub screws: bool,
    pub wall: f64,
    /// Magnet pockets for baseplates.
    pub plate_magnets: bool,
}

impl Default for Params {
    fn default() -> Self {
        Self {
            kind: Kind::Bin,
            units_x: 2,
            units_y: 1,
            height_u: 6,
            div_x: 1,
            div_y: 1,
            lip: true,
            magnets: true,
            scoop: true,
            label: false,
            screws: false,
            wall: 1.2,
            plate_magnets: false,
        }
    }
}

pub const UNITS_RANGE: (u32, u32) = (1, 8);
pub const HEIGHT_RANGE: (u32, u32) = (2, 18);
pub const DIV_RANGE: (u32, u32) = (1, 8);
pub const WALL_RANGE: (f64, f64) = (0.8, 2.4);

/// Non-fatal adjustments the generator had to make.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Warning {
    CompartmentsTooSmall,
    LabelSkipped,
    ScoopReduced,
}

impl Params {
    pub fn clamped(&self) -> Self {
        let mut p = self.clone();
        p.units_x = p.units_x.clamp(UNITS_RANGE.0, UNITS_RANGE.1);
        p.units_y = p.units_y.clamp(UNITS_RANGE.0, UNITS_RANGE.1);
        p.height_u = p.height_u.clamp(HEIGHT_RANGE.0, HEIGHT_RANGE.1);
        p.div_x = p.div_x.clamp(DIV_RANGE.0, DIV_RANGE.1);
        p.div_y = p.div_y.clamp(DIV_RANGE.0, DIV_RANGE.1);
        p.wall = if p.wall.is_finite() { p.wall.clamp(WALL_RANGE.0, WALL_RANGE.1) } else { 1.2 };
        p
    }

    /// Outer size in millimetres (x, y, z).
    pub fn size(&self) -> [f64; 3] {
        let p = self.clamped();
        match p.kind {
            Kind::Bin => [
                p.units_x as f64 * PITCH - CLEARANCE,
                p.units_y as f64 * PITCH - CLEARANCE,
                p.height_u as f64 * HEIGHT_UNIT + if p.lip { LIP_HEIGHT } else { 0.0 },
            ],
            Kind::Baseplate => [
                p.units_x as f64 * PITCH,
                p.units_y as f64 * PITCH,
                plate_floor(&p) + PLATE_PROFILE,
            ],
        }
    }

    /// Short file-friendly description, e.g. `bin_2x1x6`.
    pub fn slug(&self) -> String {
        let p = self.clamped();
        match p.kind {
            Kind::Bin => format!("bin_{}x{}x{}", p.units_x, p.units_y, p.height_u),
            Kind::Baseplate => format!("baseplate_{}x{}", p.units_x, p.units_y),
        }
    }
}

/// Generates the part mesh. Coordinates are millimetres, centred in X/Y with
/// the bottom at Z = 0.
pub fn generate(params: &Params) -> (Mesh, Vec<Warning>) {
    let p = params.clamped();
    match p.kind {
        Kind::Bin => bin(&p),
        Kind::Baseplate => (baseplate(&p), Vec::new()),
    }
}

fn inset_rect(w: f64, d: f64, r: f64, inset: f64) -> Vec<P2> {
    rounded_rect(-w / 2.0 + inset, -d / 2.0 + inset, w / 2.0 - inset, d / 2.0 - inset, r - inset, SEG)
}

fn square(cx: f64, cy: f64, size: f64, r: f64) -> Vec<P2> {
    let h = size / 2.0;
    rounded_rect(cx - h, cy - h, cx + h, cy + h, r, SEG)
}

fn unit_centers(nx: u32, ny: u32) -> Vec<(f64, f64)> {
    let mut v = Vec::new();
    for j in 0..ny {
        for i in 0..nx {
            v.push((
                (i as f64 - (nx as f64 - 1.0) / 2.0) * PITCH,
                (j as f64 - (ny as f64 - 1.0) / 2.0) * PITCH,
            ));
        }
    }
    v
}

fn hole_points(centers: &[(f64, f64)]) -> Vec<(f64, f64)> {
    let mut v = Vec::new();
    for &(cx, cy) in centers {
        for (sx, sy) in [(-1.0, -1.0), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)] {
            v.push((cx + sx * MAGNET_OFFSET, cy + sy * MAGNET_OFFSET));
        }
    }
    v
}

#[derive(Clone, Copy)]
struct Comp {
    x0: f64,
    y0: f64,
    x1: f64,
    y1: f64,
    r: f64,
    front: bool,
    back: bool,
}

struct BinFeatures {
    floor: f64,
    div_top: f64,
    scoop_r: f64,
    label_len: f64,
    label_z0: f64,
    label_z1: f64,
}

impl BinFeatures {
    fn scoop_offset(&self, z: f64) -> f64 {
        if self.scoop_r <= 0.0 || z >= self.floor + self.scoop_r {
            return 0.0;
        }
        let r = self.scoop_r;
        let dz = self.floor + r - z;
        r - (r * r - dz * dz).max(0.0).sqrt()
    }
    fn label_offset(&self, z: f64) -> f64 {
        if self.label_len <= 0.0 || z <= self.label_z0 {
            0.0
        } else if z >= self.label_z1 {
            self.label_len
        } else {
            z - self.label_z0
        }
    }
}

fn comp_ring(c: &Comp, f: &BinFeatures, z: f64) -> Vec<P2> {
    let mut y0 = c.y0;
    let mut y1 = c.y1;
    if c.front {
        y0 += f.scoop_offset(z);
    }
    if c.back {
        y1 -= f.label_offset(z);
    }
    rounded_rect(c.x0, y0, c.x1, y1, c.r, SEG)
}

fn bin(p: &Params) -> (Mesh, Vec<Warning>) {
    let mut warnings = Vec::new();
    let [w, d, _] = p.size();
    let h = p.height_u as f64 * HEIGHT_UNIT;
    let t = p.wall;
    let centers = unit_centers(p.units_x, p.units_y);
    let holes = hole_points(&centers);
    let outline = inset_rect(w, d, BIN_RADIUS, 0.0);

    let mut slabs: Vec<Slab> = Vec::new();

    // Feet: 0.8 mm chamfer, 1.8 mm vertical, 2.15 mm chamfer.
    let foot = |size: f64, r: f64| -> Vec<Vec<P2>> {
        centers.iter().map(|&(cx, cy)| square(cx, cy, size, r)).collect()
    };
    let f0 = foot(35.6, 0.8);
    let f1 = foot(37.2, 1.6);
    let f2 = foot(41.5, BIN_RADIUS);
    let magnet_rings: Vec<Vec<P2>> = holes.iter().map(|&(x, y)| circle(x, y, MAGNET_D / 2.0, CIRCLE_SEG)).collect();
    let screw_rings: Vec<Vec<P2>> = holes.iter().map(|&(x, y)| circle(x, y, SCREW_D / 2.0, CIRCLE_SEG / 2)).collect();
    let holes_at = |z: f64| -> Vec<Ring> {
        if p.magnets && z < MAGNET_DEPTH {
            magnet_rings.iter().map(|r| Ring::prism(r.clone(), true)).collect()
        } else if p.screws {
            screw_rings.iter().map(|r| Ring::prism(r.clone(), true)).collect()
        } else {
            Vec::new()
        }
    };
    let mut foot_slab = |z0: f64, z1: f64, a: &[Vec<P2>], b: &[Vec<P2>]| {
        let mut rings: Vec<Ring> = a.iter().zip(b).map(|(x, y)| Ring::loft(x.clone(), y.clone(), false)).collect();
        rings.extend(holes_at((z0 + z1) / 2.0));
        slabs.push(Slab { z0, z1, rings });
    };
    foot_slab(0.0, 0.8, &f0, &f1);
    if p.magnets {
        foot_slab(0.8, MAGNET_DEPTH, &f1, &f1);
        foot_slab(MAGNET_DEPTH, 2.6, &f1, &f1);
    } else {
        foot_slab(0.8, 2.6, &f1, &f1);
    }
    foot_slab(2.6, FOOT_HEIGHT, &f1, &f2);

    // Solid floor above the feet.
    {
        let mut rings = vec![Ring::prism(outline.clone(), false)];
        if p.screws {
            rings.extend(screw_rings.iter().map(|r| Ring::prism(r.clone(), true)));
        }
        slabs.push(Slab { z0: FOOT_HEIGHT, z1: FLOOR_TOP, rings });
    }

    // Compartments.
    let div_top = if p.lip { h - (2.6 - t) } else { h };
    let iw = w - 2.0 * t;
    let id = d - 2.0 * t;
    let mut cw = (iw - (p.div_x as f64 - 1.0) * t) / p.div_x as f64;
    let mut cd = (id - (p.div_y as f64 - 1.0) * t) / p.div_y as f64;
    let (mut nx, mut ny) = (p.div_x, p.div_y);
    if cw < 4.0 || cd < 4.0 {
        warnings.push(Warning::CompartmentsTooSmall);
        if cw < 4.0 {
            nx = 1;
            cw = iw;
        }
        if cd < 4.0 {
            ny = 1;
            cd = id;
        }
    }
    let cavity = div_top - FLOOR_TOP;
    let mut scoop_r = 0.0;
    if p.scoop {
        let ideal = 12.0f64.min(cd * 0.45);
        scoop_r = ideal.min(cavity * 0.6);
        if scoop_r < ideal - 1e-9 {
            warnings.push(Warning::ScoopReduced);
        }
        if scoop_r < 1.0 {
            scoop_r = 0.0;
        }
    }
    let mut label_len = 0.0;
    let label_z1 = div_top - LABEL_THICKNESS;
    if p.label {
        let room_z = label_z1 - (FLOOR_TOP + scoop_r) - 0.5;
        let room_y = cd - scoop_r - 3.0;
        label_len = LABEL_DEPTH.min(room_z).min(room_y);
        if label_len < 4.0 {
            label_len = 0.0;
            warnings.push(Warning::LabelSkipped);
        }
    }
    let feat = BinFeatures {
        floor: FLOOR_TOP,
        div_top,
        scoop_r,
        label_len,
        label_z0: label_z1 - label_len,
        label_z1,
    };
    let rc = (BIN_RADIUS - t).max(0.6);
    let mut comps = Vec::new();
    for j in 0..ny {
        for i in 0..nx {
            let x0 = -w / 2.0 + t + i as f64 * (cw + t);
            let y0 = -d / 2.0 + t + j as f64 * (cd + t);
            comps.push(Comp { x0, y0, x1: x0 + cw, y1: y0 + cd, r: rc, front: true, back: j == ny - 1 });
        }
    }
    let mut zs = vec![FLOOR_TOP, div_top];
    if scoop_r > 0.0 {
        for k in 1..SCOOP_STEPS {
            zs.push(FLOOR_TOP + scoop_r * k as f64 / SCOOP_STEPS as f64);
        }
        zs.push(FLOOR_TOP + scoop_r);
    }
    if label_len > 0.0 {
        zs.push(feat.label_z0);
        zs.push(feat.label_z1);
    }
    zs.sort_by(|a, b| a.partial_cmp(b).unwrap());
    zs.dedup_by(|a, b| (*a - *b).abs() < 1e-6);
    for win in zs.windows(2) {
        let (z0, z1) = (win[0], win[1]);
        let mut rings = vec![Ring::prism(outline.clone(), false)];
        for c in &comps {
            rings.push(Ring::loft(comp_ring(c, &feat, z0), comp_ring(c, &feat, z1), true));
        }
        slabs.push(Slab { z0, z1, rings });
    }
    let _ = feat.div_top;

    // Stacking lip.
    if p.lip {
        let lip = |z0: f64, z1: f64, a: f64, b: f64| Slab {
            z0,
            z1,
            rings: vec![
                Ring::prism(outline.clone(), false),
                Ring::loft(inset_rect(w, d, BIN_RADIUS, a), inset_rect(w, d, BIN_RADIUS, b), true),
            ],
        };
        if div_top < h - 1e-6 {
            slabs.push(lip(div_top, h, t, 2.6));
        }
        slabs.push(lip(h, h + 1.9, 2.6, 0.7));
        slabs.push(lip(h + 1.9, h + 3.7, 0.7, 0.7));
        slabs.push(lip(h + 3.7, h + LIP_HEIGHT, 0.7, 0.6));
    }

    (build(&slabs), warnings)
}

const PLATE_PROFILE: f64 = 4.45;

fn plate_floor(p: &Params) -> f64 {
    if p.plate_magnets { 3.0 } else { 0.0 }
}

fn baseplate(p: &Params) -> Mesh {
    let w = p.units_x as f64 * PITCH;
    let d = p.units_y as f64 * PITCH;
    let outline = inset_rect(w, d, PLATE_RADIUS, 0.0);
    let centers = unit_centers(p.units_x, p.units_y);
    let fz = plate_floor(p);
    let pocket = |inset: f64| -> Vec<Vec<P2>> {
        centers.iter().map(|&(cx, cy)| square(cx, cy, PITCH - 2.0 * inset, PLATE_RADIUS - inset)).collect()
    };
    let mut slabs = Vec::new();
    if p.plate_magnets {
        let skin = fz - MAGNET_DEPTH;
        slabs.push(Slab { z0: 0.0, z1: skin, rings: vec![Ring::prism(outline.clone(), false)] });
        let mut rings = vec![Ring::prism(outline.clone(), false)];
        for (x, y) in hole_points(&centers) {
            rings.push(Ring::prism(circle(x, y, MAGNET_D / 2.0, CIRCLE_SEG), true));
        }
        slabs.push(Slab { z0: skin, z1: fz, rings });
    }
    let stage = |z0: f64, z1: f64, a: f64, b: f64| {
        let mut rings = vec![Ring::prism(outline.clone(), false)];
        for (x, y) in pocket(a).into_iter().zip(pocket(b)) {
            rings.push(Ring::loft(x, y, true));
        }
        Slab { z0, z1, rings }
    };
    slabs.push(stage(fz, fz + 2.15, 2.85, 0.7));
    slabs.push(stage(fz + 2.15, fz + 3.95, 0.7, 0.7));
    slabs.push(stage(fz + 3.95, fz + PLATE_PROFILE, 0.7, 0.2));
    build(&slabs)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn check(p: &Params) {
        let (m, _) = generate(p);
        let open = m.open_edges();
        let v = m.volume();
        assert!(open == 0, "{:?}: {} open edges", p, open);
        assert!(v > 0.0, "{:?}: volume {}", p, v);
        let (lo, hi) = m.bounds();
        let s = p.size();
        assert!((hi[0] - lo[0] - s[0]).abs() < 1e-3, "x size");
        assert!((hi[1] - lo[1] - s[1]).abs() < 1e-3, "y size");
        assert!((hi[2] - lo[2] - s[2]).abs() < 1e-3, "z size {:?} vs {:?}", hi[2] - lo[2], s[2]);
    }

    #[test]
    fn default_bin_is_closed() {
        check(&Params::default());
    }

    #[test]
    fn bin_variants_are_closed() {
        for (ux, uy) in [(1, 1), (2, 3), (4, 2)] {
            for hu in [2, 3, 6] {
                for (dx, dy) in [(1, 1), (3, 2), (8, 8)] {
                    for flags in 0..32u32 {
                        let p = Params {
                            kind: Kind::Bin,
                            units_x: ux,
                            units_y: uy,
                            height_u: hu,
                            div_x: dx,
                            div_y: dy,
                            lip: flags & 1 != 0,
                            magnets: flags & 2 != 0,
                            scoop: flags & 4 != 0,
                            label: flags & 8 != 0,
                            screws: flags & 16 != 0,
                            wall: 1.2,
                            plate_magnets: false,
                        };
                        check(&p);
                    }
                }
            }
        }
    }

    #[test]
    fn walls_are_closed() {
        for wall in [0.8, 1.6, 2.4] {
            check(&Params { wall, div_x: 2, div_y: 2, label: true, screws: true, ..Params::default() });
        }
    }

    #[test]
    fn baseplates_are_closed() {
        for (ux, uy) in [(1, 1), (3, 2), (5, 5)] {
            for m in [false, true] {
                check(&Params { kind: Kind::Baseplate, units_x: ux, units_y: uy, plate_magnets: m, ..Params::default() });
            }
        }
    }
}
