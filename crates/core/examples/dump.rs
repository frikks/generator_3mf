use generator_3mf_core::{export, generate, Kind, Params};

fn main() {
    let out = std::env::args().nth(1).unwrap_or_else(|| "out".into());
    std::fs::create_dir_all(&out).unwrap();
    let cases = [
        ("bin_default", Params::default()),
        (
            "bin_full",
            Params {
                units_x: 3,
                units_y: 2,
                height_u: 6,
                div_x: 3,
                div_y: 2,
                label: true,
                screws: true,
                ..Params::default()
            },
        ),
        (
            "bin_plain",
            Params {
                height_u: 3,
                lip: false,
                magnets: false,
                scoop: false,
                ..Params::default()
            },
        ),
        (
            "plate",
            Params {
                kind: Kind::Baseplate,
                units_x: 3,
                units_y: 2,
                ..Params::default()
            },
        ),
        (
            "plate_magnets",
            Params {
                kind: Kind::Baseplate,
                units_x: 2,
                units_y: 2,
                plate_magnets: true,
                ..Params::default()
            },
        ),
        (
            "bin_big",
            Params {
                units_x: 8,
                units_y: 8,
                height_u: 18,
                div_x: 8,
                div_y: 8,
                label: true,
                screws: true,
                ..Params::default()
            },
        ),
    ];
    for (name, p) in cases {
        let t = std::time::Instant::now();
        let (m, w) = generate(&p);
        let dt = t.elapsed();
        std::fs::write(format!("{out}/{name}.stl"), export::stl(&m, name)).unwrap();
        std::fs::write(
            format!("{out}/{name}.3mf"),
            export::three_mf(&m, name, [0xe0, 0x66, 0x2b], [128.0, 128.0]),
        )
        .unwrap();
        println!(
            "{name}: {} tris, open {}, vol {:.0} mm3, {:?}, warn {:?}",
            m.triangles.len(),
            m.open_edges(),
            m.volume(),
            dt,
            w
        );
    }
}
