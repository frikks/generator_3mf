//! Software z-buffer renderer for the 3D preview.
//!
//! Rendering on the CPU keeps the preview identical on desktop, Android and
//! the web, with correct occlusion and outlined edges.

use crate::theme::Palette;
use eframe::egui::{self, Color32, ColorImage, Rect, Sense, TextureHandle, TextureOptions};
use generator_3mf_core::Mesh;

type V3 = [f32; 3];

fn sub(a: V3, b: V3) -> V3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}
fn dot(a: V3, b: V3) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}
fn cross(a: V3, b: V3) -> V3 {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}
fn norm(a: V3) -> V3 {
    let l = dot(a, a).sqrt();
    if l > 0.0 {
        [a[0] / l, a[1] / l, a[2] / l]
    } else {
        a
    }
}
fn scale(a: V3, s: f32) -> V3 {
    [a[0] * s, a[1] * s, a[2] * s]
}
fn add(a: V3, b: V3) -> V3 {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

const DEFAULT_YAW: f32 = -0.65;
const DEFAULT_PITCH: f32 = 0.6;
const FOV: f32 = 0.55;

#[derive(Clone, Copy, PartialEq)]
struct Key {
    version: u64,
    yaw: f32,
    pitch: f32,
    zoom: f32,
    w: usize,
    h: usize,
    color: [u8; 3],
    edge: [u8; 4],
}

pub struct Preview {
    tris: Vec<[V3; 3]>,
    normals: Vec<V3>,
    center: V3,
    radius: f32,
    version: u64,
    yaw: f32,
    pitch: f32,
    zoom: f32,
    texture: Option<TextureHandle>,
    key: Option<Key>,
}

impl Default for Preview {
    fn default() -> Self {
        Self {
            tris: Vec::new(),
            normals: Vec::new(),
            center: [0.0; 3],
            radius: 1.0,
            version: 0,
            yaw: DEFAULT_YAW,
            pitch: DEFAULT_PITCH,
            zoom: 1.0,
            texture: None,
            key: None,
        }
    }
}

impl Preview {
    pub fn set_mesh(&mut self, mesh: &Mesh) {
        self.tris.clear();
        self.normals.clear();
        let f = |p: [f64; 3]| [p[0] as f32, p[1] as f32, p[2] as f32];
        for t in &mesh.triangles {
            let a = f(mesh.vertices[t[0] as usize]);
            let b = f(mesh.vertices[t[1] as usize]);
            let c = f(mesh.vertices[t[2] as usize]);
            self.normals.push(norm(cross(sub(b, a), sub(c, a))));
            self.tris.push([a, b, c]);
        }
        let (lo, hi) = mesh.bounds();
        self.center = [
            ((lo[0] + hi[0]) / 2.0) as f32,
            ((lo[1] + hi[1]) / 2.0) as f32,
            ((lo[2] + hi[2]) / 2.0) as f32,
        ];
        let d = [
            (hi[0] - lo[0]) as f32,
            (hi[1] - lo[1]) as f32,
            (hi[2] - lo[2]) as f32,
        ];
        self.radius = (dot(d, d).sqrt() / 2.0).max(1.0);
        self.version += 1;
    }

    pub fn reset_view(&mut self) {
        self.yaw = DEFAULT_YAW;
        self.pitch = DEFAULT_PITCH;
        self.zoom = 1.0;
    }

    pub fn show(&mut self, ui: &mut egui::Ui, rect: Rect, pal: &Palette, color: [u8; 3]) {
        let resp = ui.allocate_rect(rect, Sense::click_and_drag());
        if resp.dragged() {
            let d = resp.drag_delta();
            self.yaw -= d.x * 0.008;
            self.pitch = (self.pitch + d.y * 0.008).clamp(-1.45, 1.45);
        }
        if resp.double_clicked() {
            self.reset_view();
        }
        if resp.hovered() || resp.dragged() {
            let (zoom, scroll) = ui.input(|i| (i.zoom_delta(), i.smooth_scroll_delta.y));
            let mut z = self.zoom * zoom;
            if scroll != 0.0 {
                z *= (scroll * 0.0025).exp();
            }
            self.zoom = z.clamp(0.3, 8.0);
        }
        let painter = ui.painter_at(rect);
        painter.rect_filled(rect, 12.0, pal.preview_bg);
        if self.tris.is_empty() {
            return;
        }
        let ppp = ui.ctx().pixels_per_point();
        let mut w = (rect.width() * ppp).round().max(1.0) as usize;
        let mut h = (rect.height() * ppp).round().max(1.0) as usize;
        let max_px = 1_800_000usize;
        if w * h > max_px {
            let s = ((max_px as f32) / (w * h) as f32).sqrt();
            w = ((w as f32) * s) as usize;
            h = ((h as f32) * s) as usize;
        }
        let edge = pal.preview_edge.to_array();
        let key = Key {
            version: self.version,
            yaw: self.yaw,
            pitch: self.pitch,
            zoom: self.zoom,
            w,
            h,
            color,
            edge,
        };
        if self.key != Some(key) || self.texture.is_none() {
            let ss = if w * h <= 700_000 { 2 } else { 1 };
            let img = self.render(w, h, ss, color, pal.preview_edge);
            match &mut self.texture {
                Some(t) => t.set(img, TextureOptions::LINEAR),
                None => {
                    self.texture = Some(ui.ctx().load_texture(
                        "preview",
                        img,
                        TextureOptions::LINEAR,
                    ))
                }
            }
            self.key = Some(key);
        }
        if let Some(t) = &self.texture {
            painter.image(
                t.id(),
                rect,
                Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                Color32::WHITE,
            );
        }
    }

    fn render(&self, w: usize, h: usize, ss: usize, color: [u8; 3], edge: Color32) -> ColorImage {
        let rw = w * ss;
        let rh = h * ss;
        let dir = [
            self.pitch.cos() * self.yaw.sin(),
            -self.pitch.cos() * self.yaw.cos(),
            self.pitch.sin(),
        ];
        let dist = self.radius / (FOV / 2.0).sin() / self.zoom;
        let eye = add(self.center, scale(dir, dist));
        let fwd = scale(dir, -1.0);
        let right = norm(cross(fwd, [0.0, 0.0, 1.0]));
        let up = cross(right, fwd);
        let focal = (rw.min(rh) as f32 / 2.0) / (FOV / 2.0).tan();
        let (cx, cy) = (rw as f32 / 2.0, rh as f32 / 2.0);
        let light = norm(add(
            add(scale(fwd, -1.0), scale(up, 0.9)),
            scale(right, -0.5),
        ));
        let fill = norm(add(scale(fwd, -1.0), scale(right, 0.8)));
        let head = scale(fwd, -1.0);
        let base = [color[0] as f32, color[1] as f32, color[2] as f32];

        let mut depth = vec![0.0f32; rw * rh];
        let mut ids = vec![u32::MAX; rw * rh];
        let near = 0.05 * dist;
        for (ti, tri) in self.tris.iter().enumerate() {
            let n = self.normals[ti];
            if dot(n, sub(tri[0], eye)) >= 0.0 {
                continue;
            }
            let mut sp = [[0.0f32; 3]; 3];
            let mut behind = false;
            for k in 0..3 {
                let v = sub(tri[k], eye);
                let z = dot(v, fwd);
                if z < near {
                    behind = true;
                    break;
                }
                sp[k] = [
                    cx + dot(v, right) / z * focal,
                    cy - dot(v, up) / z * focal,
                    1.0 / z,
                ];
            }
            if behind {
                continue;
            }
            let area = (sp[1][0] - sp[0][0]) * (sp[2][1] - sp[0][1])
                - (sp[2][0] - sp[0][0]) * (sp[1][1] - sp[0][1]);
            if area.abs() < 1e-8 {
                continue;
            }
            let x0 = sp
                .iter()
                .map(|p| p[0])
                .fold(f32::MAX, f32::min)
                .floor()
                .max(0.0) as i64;
            let x1 = sp
                .iter()
                .map(|p| p[0])
                .fold(f32::MIN, f32::max)
                .ceil()
                .min(rw as f32 - 1.0) as i64;
            let y0 = sp
                .iter()
                .map(|p| p[1])
                .fold(f32::MAX, f32::min)
                .floor()
                .max(0.0) as i64;
            let y1 = sp
                .iter()
                .map(|p| p[1])
                .fold(f32::MIN, f32::max)
                .ceil()
                .min(rh as f32 - 1.0) as i64;
            if x0 > x1 || y0 > y1 {
                continue;
            }
            let inv = 1.0 / area;
            for py in y0..=y1 {
                let fy = py as f32 + 0.5;
                for px in x0..=x1 {
                    let fx = px as f32 + 0.5;
                    let w0 = ((sp[1][0] - fx) * (sp[2][1] - fy)
                        - (sp[2][0] - fx) * (sp[1][1] - fy))
                        * inv;
                    let w1 = ((sp[2][0] - fx) * (sp[0][1] - fy)
                        - (sp[0][0] - fx) * (sp[2][1] - fy))
                        * inv;
                    let w2 = 1.0 - w0 - w1;
                    if w0 < -1e-5 || w1 < -1e-5 || w2 < -1e-5 {
                        continue;
                    }
                    let iz = w0 * sp[0][2] + w1 * sp[1][2] + w2 * sp[2][2];
                    let idx = py as usize * rw + px as usize;
                    if iz > depth[idx] {
                        depth[idx] = iz;
                        ids[idx] = ti as u32;
                    }
                }
            }
        }

        // Shade, outline edges and downsample.
        let shade: Vec<[f32; 3]> = self
            .normals
            .iter()
            .map(|&n| {
                let k = 0.34
                    + 0.38 * dot(n, light).max(0.0)
                    + 0.22 * dot(n, fill).max(0.0)
                    + 0.24 * dot(n, head).max(0.0);
                let k = k.min(1.15);
                [base[0] * k, base[1] * k, base[2] * k]
            })
            .collect();
        let edge_c = [edge.r() as f32, edge.g() as f32, edge.b() as f32];
        let mut px = vec![[0.0f32; 4]; rw * rh];
        for y in 0..rh {
            for x in 0..rw {
                let i = y * rw + x;
                let id = ids[i];
                if id == u32::MAX {
                    continue;
                }
                let mut c = shade[id as usize];
                let n = self.normals[id as usize];
                let mut is_edge = false;
                for (dx, dy) in [(1i64, 0i64), (0, 1), (-1, 0), (0, -1)] {
                    let nx = x as i64 + dx * ss as i64;
                    let ny = y as i64 + dy * ss as i64;
                    if nx < 0 || ny < 0 || nx >= rw as i64 || ny >= rh as i64 {
                        continue;
                    }
                    let j = ny as usize * rw + nx as usize;
                    let o = ids[j];
                    if o == u32::MAX {
                        is_edge = true;
                        break;
                    }
                    if o != id {
                        let no = self.normals[o as usize];
                        let dz = (1.0 / depth[j] - 1.0 / depth[i]).abs();
                        if dot(n, no) < 0.8 || dz > 0.004 * dist {
                            is_edge = true;
                            break;
                        }
                    }
                }
                if is_edge {
                    for k in 0..3 {
                        c[k] = c[k] * 0.45 + edge_c[k] * 0.55;
                    }
                }
                px[i] = [c[0].min(255.0), c[1].min(255.0), c[2].min(255.0), 255.0];
            }
        }
        let mut out = Vec::with_capacity(w * h);
        let norm_ss = 1.0 / (ss * ss) as f32;
        for y in 0..h {
            for x in 0..w {
                let mut acc = [0.0f32; 4];
                for sy in 0..ss {
                    for sx in 0..ss {
                        let p = px[(y * ss + sy) * rw + x * ss + sx];
                        for k in 0..4 {
                            acc[k] += p[k];
                        }
                    }
                }
                let a = acc[3] * norm_ss;
                // Colours are already premultiplied because background pixels are zero.
                out.push(Color32::from_rgba_premultiplied(
                    (acc[0] * norm_ss) as u8,
                    (acc[1] * norm_ss) as u8,
                    (acc[2] * norm_ss) as u8,
                    a as u8,
                ));
            }
        }
        ColorImage {
            size: [w, h],
            pixels: out,
        }
    }
}
