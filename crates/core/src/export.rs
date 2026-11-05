//! STL and 3MF writers.

use crate::mesh::{triangle_normal, Mesh};
use std::fmt::Write as _;

/// Binary STL.
pub fn stl(mesh: &Mesh, name: &str) -> Vec<u8> {
    let mut out = Vec::with_capacity(84 + mesh.triangles.len() * 50);
    let mut header = [0u8; 80];
    let label = format!("generator_3mf {name}");
    let n = label.len().min(80);
    header[..n].copy_from_slice(&label.as_bytes()[..n]);
    out.extend_from_slice(&header);
    out.extend_from_slice(&(mesh.triangles.len() as u32).to_le_bytes());
    for t in &mesh.triangles {
        let a = mesh.vertices[t[0] as usize];
        let b = mesh.vertices[t[1] as usize];
        let c = mesh.vertices[t[2] as usize];
        for v in triangle_normal(a, b, c).into_iter().chain(a).chain(b).chain(c) {
            out.extend_from_slice(&(v as f32).to_le_bytes());
        }
        out.extend_from_slice(&[0, 0]);
    }
    out
}

fn xml_escape(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            '&' => "&amp;".to_string(),
            '<' => "&lt;".to_string(),
            '>' => "&gt;".to_string(),
            '"' => "&quot;".to_string(),
            '\'' => "&apos;".to_string(),
            c if (c as u32) < 0x20 => " ".to_string(),
            c => c.to_string(),
        })
        .collect()
}

fn fmt_num(s: &mut String, v: f64) {
    let v = if v.abs() < 5e-7 { 0.0 } else { v };
    let mut t = format!("{v:.5}");
    while t.ends_with('0') {
        t.pop();
    }
    if t.ends_with('.') {
        t.pop();
    }
    s.push_str(&t);
}

/// 3MF package (Core specification, millimetres) with a display colour.
/// `color` is `[r, g, b]`; the object is placed at `(offset_x, offset_y)`.
pub fn three_mf(mesh: &Mesh, name: &str, color: [u8; 3], offset: [f64; 2]) -> Vec<u8> {
    let mut model = String::with_capacity(mesh.triangles.len() * 60 + mesh.vertices.len() * 50);
    let title = xml_escape(name);
    let _ = write!(
        model,
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
<model unit=\"millimeter\" xml:lang=\"en-US\" xmlns=\"http://schemas.microsoft.com/3dmanufacturing/core/2015/02\">\n\
 <metadata name=\"Title\">{title}</metadata>\n\
 <metadata name=\"Application\">generator_3mf {ver}</metadata>\n\
 <resources>\n\
  <basematerials id=\"1\">\n\
   <base name=\"{title}\" displaycolor=\"#{r:02X}{g:02X}{b:02X}FF\"/>\n\
  </basematerials>\n\
  <object id=\"2\" name=\"{title}\" type=\"model\" pid=\"1\" pindex=\"0\">\n\
   <mesh>\n\
    <vertices>\n",
        ver = crate::VERSION,
        r = color[0],
        g = color[1],
        b = color[2],
    );
    for v in &mesh.vertices {
        model.push_str("     <vertex x=\"");
        fmt_num(&mut model, v[0]);
        model.push_str("\" y=\"");
        fmt_num(&mut model, v[1]);
        model.push_str("\" z=\"");
        fmt_num(&mut model, v[2]);
        model.push_str("\"/>\n");
    }
    model.push_str("    </vertices>\n    <triangles>\n");
    for t in &mesh.triangles {
        let _ = writeln!(model, "     <triangle v1=\"{}\" v2=\"{}\" v3=\"{}\"/>", t[0], t[1], t[2]);
    }
    model.push_str("    </triangles>\n   </mesh>\n  </object>\n </resources>\n <build>\n");
    let mut tr = String::new();
    fmt_num(&mut tr, offset[0]);
    tr.push(' ');
    fmt_num(&mut tr, offset[1]);
    let _ = write!(model, "  <item objectid=\"2\" transform=\"1 0 0 0 1 0 0 0 1 {tr} 0\"/>\n </build>\n</model>\n");

    let content_types = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
<Types xmlns=\"http://schemas.openxmlformats.org/package/2006/content-types\">\n\
 <Default Extension=\"rels\" ContentType=\"application/vnd.openxmlformats-package.relationships+xml\"/>\n\
 <Default Extension=\"model\" ContentType=\"application/vnd.ms-package.3dmanufacturing-3dmodel+xml\"/>\n\
</Types>\n";
    let rels = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
<Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\">\n\
 <Relationship Target=\"/3D/3dmodel.model\" Id=\"rel0\" Type=\"http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel\"/>\n\
</Relationships>\n";
    let mut zip = ZipWriter::default();
    zip.add("[Content_Types].xml", content_types.as_bytes());
    zip.add("_rels/.rels", rels.as_bytes());
    zip.add("3D/3dmodel.model", model.as_bytes());
    zip.finish()
}

/// Minimal ZIP writer (deflate), enough for OPC packages such as 3MF.
#[derive(Default)]
pub struct ZipWriter {
    data: Vec<u8>,
    central: Vec<u8>,
    count: u16,
}

impl ZipWriter {
    pub fn add(&mut self, name: &str, content: &[u8]) {
        let crc = crc32fast::hash(content);
        let packed = miniz_oxide::deflate::compress_to_vec(content, 6);
        let (method, body): (u16, &[u8]) = if packed.len() < content.len() { (8, &packed) } else { (0, content) };
        let offset = self.data.len() as u32;
        let name_b = name.as_bytes();
        // DOS date 1980-01-01 00:00 keeps output reproducible.
        let (time, date) = (0u16, 0x21u16);
        let d = &mut self.data;
        d.extend_from_slice(&0x0403_4b50u32.to_le_bytes());
        d.extend_from_slice(&20u16.to_le_bytes());
        d.extend_from_slice(&0u16.to_le_bytes());
        d.extend_from_slice(&method.to_le_bytes());
        d.extend_from_slice(&time.to_le_bytes());
        d.extend_from_slice(&date.to_le_bytes());
        d.extend_from_slice(&crc.to_le_bytes());
        d.extend_from_slice(&(body.len() as u32).to_le_bytes());
        d.extend_from_slice(&(content.len() as u32).to_le_bytes());
        d.extend_from_slice(&(name_b.len() as u16).to_le_bytes());
        d.extend_from_slice(&0u16.to_le_bytes());
        d.extend_from_slice(name_b);
        d.extend_from_slice(body);

        let c = &mut self.central;
        c.extend_from_slice(&0x0201_4b50u32.to_le_bytes());
        c.extend_from_slice(&20u16.to_le_bytes());
        c.extend_from_slice(&20u16.to_le_bytes());
        c.extend_from_slice(&0u16.to_le_bytes());
        c.extend_from_slice(&method.to_le_bytes());
        c.extend_from_slice(&time.to_le_bytes());
        c.extend_from_slice(&date.to_le_bytes());
        c.extend_from_slice(&crc.to_le_bytes());
        c.extend_from_slice(&(body.len() as u32).to_le_bytes());
        c.extend_from_slice(&(content.len() as u32).to_le_bytes());
        c.extend_from_slice(&(name_b.len() as u16).to_le_bytes());
        c.extend_from_slice(&0u16.to_le_bytes());
        c.extend_from_slice(&0u16.to_le_bytes());
        c.extend_from_slice(&0u16.to_le_bytes());
        c.extend_from_slice(&0u16.to_le_bytes());
        c.extend_from_slice(&0u32.to_le_bytes());
        c.extend_from_slice(&offset.to_le_bytes());
        c.extend_from_slice(name_b);
        self.count += 1;
    }

    pub fn finish(mut self) -> Vec<u8> {
        let cd_offset = self.data.len() as u32;
        let cd_size = self.central.len() as u32;
        self.data.extend_from_slice(&self.central);
        let d = &mut self.data;
        d.extend_from_slice(&0x0605_4b50u32.to_le_bytes());
        d.extend_from_slice(&0u16.to_le_bytes());
        d.extend_from_slice(&0u16.to_le_bytes());
        d.extend_from_slice(&self.count.to_le_bytes());
        d.extend_from_slice(&self.count.to_le_bytes());
        d.extend_from_slice(&cd_size.to_le_bytes());
        d.extend_from_slice(&cd_offset.to_le_bytes());
        d.extend_from_slice(&0u16.to_le_bytes());
        self.data
    }
}
