//! Tray icons, embedded as SVG and rasterised at startup so they work
//! regardless of the user's icon theme.

use resvg::{tiny_skia, usvg};

const ON_SVG: &[u8] = include_bytes!("../icons/coffee-on.svg");
const OFF_SVG: &[u8] = include_bytes!("../icons/coffee-off.svg");

/// Sizes offered to the tray host; it picks the best fit for the panel.
const SIZES: [u32; 6] = [16, 22, 24, 32, 48, 64];

pub struct Icons {
    pub on: Vec<ksni::Icon>,
    pub off: Vec<ksni::Icon>,
}

impl Icons {
    pub fn load() -> Self {
        Self {
            on: render_all(ON_SVG),
            off: render_all(OFF_SVG),
        }
    }
}

fn render_all(svg: &[u8]) -> Vec<ksni::Icon> {
    let tree =
        usvg::Tree::from_data(svg, &usvg::Options::default()).expect("embedded icon SVG is valid");
    SIZES.iter().map(|&size| render(&tree, size)).collect()
}

fn render(tree: &usvg::Tree, size: u32) -> ksni::Icon {
    let mut pixmap = tiny_skia::Pixmap::new(size, size).expect("non-zero icon size");
    let scale = size as f32 / tree.size().width();
    resvg::render(
        tree,
        tiny_skia::Transform::from_scale(scale, scale),
        &mut pixmap.as_mut(),
    );

    // tiny-skia gives premultiplied RGBA; StatusNotifierItem wants
    // straight ARGB32 in network byte order.
    let data = pixmap
        .pixels()
        .iter()
        .flat_map(|p| {
            let c = p.demultiply();
            [c.alpha(), c.red(), c.green(), c.blue()]
        })
        .collect();

    ksni::Icon {
        width: size as i32,
        height: size as i32,
        data,
    }
}
