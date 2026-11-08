//! 2.5D solid builder.
//!
//! A part is described as a stack of slabs. Inside one slab every boundary is
//! a ring whose bottom and top outlines have the same vertex count, so walls
//! can be lofted (vertical walls or 45° chamfers). Horizontal faces between
//! slabs are computed with polygon booleans and triangulated.

use crate::mesh::{Mesh, MeshBuilder};
use i_overlay::core::fill_rule::FillRule;
use i_overlay::core::overlay_rule::OverlayRule;
use i_overlay::float::single::SingleFloatOverlay;
use i_triangle::float::triangulatable::Triangulatable;
use std::f64::consts::PI;

pub type P2 = [f64; 2];

#[derive(Clone, Debug)]
pub struct Ring {
    pub bottom: Vec<P2>,
    pub top: Vec<P2>,
    /// `true` for a void inside the material.
    pub hole: bool,
}

impl Ring {
    pub fn prism(points: Vec<P2>, hole: bool) -> Self {
        Self {
            top: points.clone(),
            bottom: points,
            hole,
        }
    }
    pub fn loft(bottom: Vec<P2>, top: Vec<P2>, hole: bool) -> Self {
        assert_eq!(
            bottom.len(),
            top.len(),
            "loft rings need equal vertex counts"
        );
        Self { bottom, top, hole }
    }
}

#[derive(Clone, Debug)]
pub struct Slab {
    pub z0: f64,
    pub z1: f64,
    pub rings: Vec<Ring>,
}

pub fn signed_area(p: &[P2]) -> f64 {
    let mut s = 0.0;
    for i in 0..p.len() {
        let a = p[i];
        let b = p[(i + 1) % p.len()];
        s += a[0] * b[1] - b[0] * a[1];
    }
    s * 0.5
}

/// Counter-clockwise rounded rectangle. Every call with the same `seg` returns
/// the same number of vertices, which allows lofting between sizes.
pub fn rounded_rect(x0: f64, y0: f64, x1: f64, y1: f64, r: f64, seg: usize) -> Vec<P2> {
    let r = r
        .max(0.05)
        .min((x1 - x0) * 0.5 - 1e-4)
        .min((y1 - y0) * 0.5 - 1e-4)
        .max(1e-3);
    let corners = [
        (x1 - r, y0 + r, -0.5 * PI),
        (x1 - r, y1 - r, 0.0),
        (x0 + r, y1 - r, 0.5 * PI),
        (x0 + r, y0 + r, PI),
    ];
    let mut out = Vec::with_capacity(4 * (seg + 1));
    for (cx, cy, a0) in corners {
        for i in 0..=seg {
            let a = a0 + 0.5 * PI * (i as f64) / (seg as f64);
            out.push([cx + r * a.cos(), cy + r * a.sin()]);
        }
    }
    out
}

pub fn circle(cx: f64, cy: f64, r: f64, seg: usize) -> Vec<P2> {
    (0..seg)
        .map(|i| {
            let a = 2.0 * PI * (i as f64) / (seg as f64);
            [cx + r * a.cos(), cy + r * a.sin()]
        })
        .collect()
}

fn section(rings: &[Ring], top: bool) -> Vec<Vec<P2>> {
    rings
        .iter()
        .map(|r| if top { r.top.clone() } else { r.bottom.clone() })
        .collect()
}

fn same_section(a: &[Vec<P2>], b: &[Vec<P2>]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.iter().zip(b).all(|(x, y)| {
        x.len() == y.len()
            && x.iter()
                .zip(y)
                .all(|(p, q)| (p[0] - q[0]).abs() < 1e-9 && (p[1] - q[1]).abs() < 1e-9)
    })
}

/// Region of `a` that is not covered by `b`.
fn difference(a: &[Vec<P2>], b: &[Vec<P2>]) -> Vec<Vec<Vec<P2>>> {
    if a.is_empty() {
        return Vec::new();
    }
    let a: Vec<Vec<P2>> = a.to_vec();
    let b: Vec<Vec<P2>> = b.to_vec();
    if b.is_empty() {
        let empty: Vec<Vec<P2>> = vec![];
        return a.overlay(&empty, OverlayRule::Subject, FillRule::EvenOdd);
    }
    a.overlay(&b, OverlayRule::Difference, FillRule::EvenOdd)
}

fn cap(builder: &mut MeshBuilder, region: Vec<Vec<Vec<P2>>>, z: f64, up: bool) {
    for shape in region {
        if shape.is_empty() {
            continue;
        }
        let tri = shape.triangulate().to_triangulation::<u32>();
        let pts = &tri.points;
        for t in tri.indices.as_chunks::<3>().0 {
            let a = pts[t[0] as usize];
            let b = pts[t[1] as usize];
            let c = pts[t[2] as usize];
            let area = (b[0] - a[0]) * (c[1] - a[1]) - (c[0] - a[0]) * (b[1] - a[1]);
            if area.abs() < 1e-12 {
                continue;
            }
            let ccw = area > 0.0;
            let (p, q) = if ccw == up { (b, c) } else { (c, b) };
            builder.triangle([a[0], a[1], z], [p[0], p[1], z], [q[0], q[1], z]);
        }
    }
}

/// Converts a slab stack into a closed triangle mesh.
pub fn build(slabs: &[Slab]) -> Mesh {
    let mut b = MeshBuilder::new();
    // Walls.
    for s in slabs {
        for ring in &s.rings {
            let mut bot = ring.bottom.clone();
            let mut top = ring.top.clone();
            let area = {
                let a = signed_area(&bot);
                if a.abs() > 1e-9 {
                    a
                } else {
                    signed_area(&top)
                }
            };
            let want_ccw = !ring.hole;
            if (area > 0.0) != want_ccw {
                bot.reverse();
                top.reverse();
            }
            let n = bot.len();
            for i in 0..n {
                let j = (i + 1) % n;
                let a0 = [bot[i][0], bot[i][1], s.z0];
                let b0 = [bot[j][0], bot[j][1], s.z0];
                let a1 = [top[i][0], top[i][1], s.z1];
                let b1 = [top[j][0], top[j][1], s.z1];
                b.triangle(a0, b0, b1);
                b.triangle(a0, b1, a1);
            }
        }
    }
    // Caps between slabs (and at the very bottom and top).
    for k in 0..=slabs.len() {
        let below = if k > 0 {
            section(&slabs[k - 1].rings, true)
        } else {
            Vec::new()
        };
        let above = if k < slabs.len() {
            section(&slabs[k].rings, false)
        } else {
            Vec::new()
        };
        let z = if k < slabs.len() {
            slabs[k].z0
        } else {
            slabs[k - 1].z1
        };
        if same_section(&below, &above) {
            continue;
        }
        cap(&mut b, difference(&below, &above), z, true);
        cap(&mut b, difference(&above, &below), z, false);
    }
    b.finish()
}
