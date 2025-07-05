use gpui::TransformationMatrix;
use rex::font::backend::ttf_parser::TtfMathFont;
use rex::layout::LayoutDimensions;
use rex::parser::color::RGBA;
use rex::render::{Backend, Cursor, Role};
use rex::{FontBackend, GraphicsBackend, font::common::GlyphId};

/// Backend for TinySkia renderer
pub struct GPUIBackend {
    paths: Vec<gpui::Path<gpui::Pixels>>,
    rects: Vec<gpui::PaintQuad>,
    /// Transform to convert from position according to ReX Renderer backend
    /// to coordinates canvas
    layout_to_canvas: gpui::TransformationMatrix,
    scale: f32,
}

impl GPUIBackend {
    pub fn new(dims: LayoutDimensions, scale: f64) -> Self {
        let scale = scale as f32;
        let layout_to_pixmap = TransformationMatrix {
            rotation_scale: [[scale, 0.], [0., scale]],
            translation: [0., dims.height as f32 * scale],
        };
        // Transform::from_translate(0.0, dims.height as f32).post_scale(scale, scale);

        Self {
            paths: vec![],
            rects: vec![],
            layout_to_canvas: layout_to_pixmap,
            scale,
        }
    }
    /// Returns data ready to be drawn onto canvas
    pub fn paths_and_rects(self) -> (Vec<gpui::Path<gpui::Pixels>>, Vec<gpui::PaintQuad>) {
        (self.paths, self.rects)
    }
}

impl FontBackend<TtfMathFont<'_>> for GPUIBackend {
    fn symbol(&mut self, pos: Cursor, gid: GlyphId, scale: f64, ctx: &TtfMathFont<'_>) {
        // Make the tiny_skia path builder implement the necessary trait to draw
        // the glyph with the TtfMathFont font backend
        let font_to_canvas: TransformationMatrix = {
            let scale = self.scale * scale as f32;
            let fm = ctx.font_matrix();

            self.layout_to_canvas
                .clone()
                .compose(TransformationMatrix {
                    rotation_scale: [[scale, 0.], [0., scale]],
                    translation: [pos.x as f32, pos.y as f32],
                })
                .compose(TransformationMatrix {
                    // font matrix 'should' only involve these components:
                    rotation_scale: [[fm.sx, 0.], [0., -fm.sy]],
                    translation: [0., 0.],
                })
        };

        struct Builder {
            open_path: gpui::PathBuilder,
            font_to_canvas: TransformationMatrix,
        }

        impl Builder {
            fn font_to_pixels(&self, x: f32, y: f32) -> gpui::Point<gpui::Pixels> {
                self.font_to_canvas.apply(gpui::Point::new(
                    // These don't really correspond to to pixel coordinates, (it's actually the font's)
                    // This is just to typecheck with TransformationMatrix the way that it has been written.
                    gpui::Pixels::from(x),
                    gpui::Pixels::from(y),
                ))
            }
        }

        impl rex::font::backend::ttf_parser::ttf_parser_crate::OutlineBuilder for Builder {
            fn move_to(&mut self, x: f32, y: f32) {
                self.open_path.move_to(self.font_to_pixels(x, y));
            }
            fn line_to(&mut self, x: f32, y: f32) {
                self.open_path.line_to(self.font_to_pixels(x, y));
            }
            fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
                self.open_path
                    .curve_to(self.font_to_pixels(x1, y1), self.font_to_pixels(x, y));
            }
            fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
                self.open_path.cubic_bezier_to(
                    self.font_to_pixels(x, y),
                    self.font_to_pixels(x1, y1),
                    self.font_to_pixels(x2, y2),
                );
            }
            fn close(&mut self) {
                self.open_path.close();
            }
        }

        let mut builder = Builder {
            open_path: gpui::PathBuilder::fill(),
            font_to_canvas,
        };
        ctx.font().outline_glyph(gid.into(), &mut builder);
        if let Ok(path) = builder.open_path.build() {
            self.paths.push(path);
        }
    }
}

impl Backend<TtfMathFont<'_>> for GPUIBackend {}

impl GraphicsBackend for GPUIBackend {
    fn bbox(&mut self, _pos: Cursor, _width: f64, _height: f64, _role: Role) {}
    fn rule(&mut self, _pos: Cursor, _width: f64, _height: f64) {}
    fn begin_color(&mut self, RGBA(_r, _g, _b, _a): RGBA) {}
    fn end_color(&mut self) {}
}

pub fn latex_to_paths(// latex: &str,
    // font: &TtfMathFont<'_>,
    // scale: f64,
) -> Vec<gpui::Path<gpui::Pixels>> {
    use rex::layout::{Style, engine::LayoutBuilder};
    static FONT: std::sync::LazyLock<TtfMathFont<'_>> = std::sync::LazyLock::new(|| {
        TtfMathFont::new(
            rex::font::backend::ttf_parser::ttf_parser_crate::Face::parse(
                include_bytes!("../XITS_Math.otf"),
                0,
            )
            .unwrap(),
        )
        .unwrap()
    });

    const FONT_SIZE: f64 = 80.0;
    let layout_engine = LayoutBuilder::new(&*FONT)
        .font_size(FONT_SIZE)
        .style(Style::Display)
        .build();

    let equation = r"e = \lim_{n \to \infty} \left(1 + \frac{1}{n}\right)^n";
    // let equation = r"x = 1";
    let parse_nodes = rex::parser::parse(equation).unwrap();

    let layout = layout_engine.layout(&parse_nodes).unwrap();

    let renderer = rex::Renderer::new();

    const SCALE: f64 = 1.; // non-1 scale broken
    let mut backend = GPUIBackend::new(layout.size(), SCALE);
    renderer.render(&layout, &mut backend);
    backend.paths_and_rects().0
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::LayoutBuilder;
//     use crate::Renderer;
//     use crate::font::backend::ttf_parser::TtfMathFont;
//     use crate::layout::Style;

//     #[cfg(feature = "ttfparser-fontparser")]
//     fn load_font<'a>(file: &'a [u8]) -> crate::font::backend::ttf_parser::TtfMathFont<'a> {
//         let font = ttf_parser::Face::parse(file, 0).unwrap();
//         TtfMathFont::new(font).unwrap()
//     }
//     #[test]
//     #[cfg(feature = "ttfparser-fontparser")]
//     fn test_tiny_skia_backend_ttfparser() {
//         // TODO: Add tests for TinySkiaBackend
//         let font_file: &[u8] = include_bytes!("../../resources/XITS_Math.otf");
//         let font = load_font(font_file);
//         let equation = "x_f = \\sqrt{\\frac{a + b}{c - d}}";
//         const FONT_SIZE: f64 = 16.0;
//         let layout_engine = LayoutBuilder::new(&font)
//             .font_size(FONT_SIZE)
//             .style(Style::Display)
//             .build();

//         let parse_nodes = crate::parser::parse(equation).unwrap();

//         let layout = layout_engine.layout(&parse_nodes).unwrap();

//         let renderer = Renderer::new();

//         const SCALE: f64 = 5.;
//         let mut tinyskia_backend = GPUIBackend::new(layout.size(), Color::WHITE, SCALE);
//         renderer.render(&layout, &mut tinyskia_backend);
//         tinyskia_backend
//             .pixmap()
//             .save_png("ttfparser-tinyskia.png")
//             .unwrap();
//     }

//     #[test]
//     #[cfg(feature = "fontrs-fontparser")]
//     fn test_tiny_skia_backend_fontrs() {
//         let font_file: &[u8] = include_bytes!("../../resources/FiraMath_Regular.otf");
//         let font = OpenTypeFont::parse(font_file).unwrap();
//         let equation = "x_f = {\\color{red}\\sqrt{\\frac{a + b}{c - d}}}";
//         const FONT_SIZE: f64 = 16.0;
//         let layout_engine = LayoutBuilder::new(&font)
//             .font_size(FONT_SIZE)
//             .style(Style::Display)
//             .build();

//         let parse_nodes = crate::parser::parse(equation).unwrap();

//         let layout = layout_engine.layout(&parse_nodes).unwrap();

//         let mut renderer = Renderer::new();
//         renderer.debug = true;

//         const SCALE: f64 = 5.;
//         let mut tinyskia_backend = GPUIBackend::new(layout.size(), Color::BLACK, SCALE);
//         renderer.render(&layout, &mut tinyskia_backend);
//         tinyskia_backend
//             .pixmap()
//             .save_png("fontrs-tinyskia.png")
//             .unwrap();
//     }
// }
