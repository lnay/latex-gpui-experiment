use gpui::{
    AbsoluteLength, App, Bounds, Element, GlobalElementId, InspectorElementId, IntoElement,
    LayoutId, Length, Path, Pixels, Size, TransformationMatrix, Window, px,
};
use gpui::{
    Application, BorderStyle, Context, Corners, PaintQuad, Render, WindowBounds, WindowOptions,
    canvas, div, prelude::*, rgb, size,
};
use rex::font::backend::ttf_parser::TtfMathFont;
use rex::layout::{Layout, LayoutDimensions};
use rex::parser::color::RGBA;
use rex::render::{Backend, Cursor, Role};
use rex::{FontBackend, GraphicsBackend, font::common::GlyphId};
use std::panic::Location;

/// ReX rendering backend to build paths ready to be drawn onto canvas
pub struct GPUIBackend {
    paths: Vec<gpui::Path<gpui::Pixels>>,
    rects: Vec<gpui::Bounds<gpui::Pixels>>,
    /// Transform to convert from position according to ReX Renderer backend
    /// to coordinates on canvas
    layout_to_canvas: gpui::TransformationMatrix,
}

impl GPUIBackend {
    pub fn new(dims: LayoutDimensions, scale: f64) -> Self {
        let scale = scale as f32;
        let layout_to_pixmap = TransformationMatrix {
            rotation_scale: [[scale, 0.], [0., scale]],
            translation: [0., dims.height as f32 * scale],
        };

        Self {
            paths: vec![],
            rects: vec![],
            layout_to_canvas: layout_to_pixmap,
        }
    }
    /// Returns data ready to be drawn onto canvas
    pub fn paths_and_rects(
        self,
    ) -> (
        Vec<gpui::Path<gpui::Pixels>>,
        Vec<gpui::Bounds<gpui::Pixels>>,
    ) {
        (self.paths, self.rects)
    }
}

impl FontBackend<TtfMathFont<'_>> for GPUIBackend {
    fn symbol(&mut self, pos: Cursor, gid: GlyphId, scale: f64, ctx: &TtfMathFont<'_>) {
        // Make the tiny_skia path builder implement the necessary trait to draw
        // the glyph with the TtfMathFont font backend
        let font_to_canvas: TransformationMatrix = {
            let scale = scale as f32;
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
    fn rule(&mut self, pos: Cursor, width: f64, height: f64) {
        use gpui::px;
        // Again the Pixels type here is misused, just to typecheck with the way TransformationMatrix is written
        let layout_top_left = gpui::Point::new(px(pos.x as f32), px(pos.y as f32));
        // Actual pixel position:
        let top_left = self.layout_to_canvas.apply(layout_top_left);
        // quick and dirty:
        let size = gpui::size(
            px(width as f32 * self.layout_to_canvas.rotation_scale[0][0]),
            px(height as f32 * self.layout_to_canvas.rotation_scale[1][1]),
        );
        self.rects.push(Bounds::new(top_left, size));
    }
    fn begin_color(&mut self, _: RGBA) {}
    fn end_color(&mut self) {}
}

pub fn latex_to_paths(
    equation: &str,
    // font: &TtfMathFont<'_>,
    font_size: f64,
) -> (Vec<gpui::Path<gpui::Pixels>>, Vec<Bounds<gpui::Pixels>>) {
    use rex::font::backend::ttf_parser::ttf_parser_crate::Face;
    use rex::layout::{Style, engine::LayoutBuilder};
    use rex::parser::parse as parse_latex;

    // This font stuff would ultimately be better if only performed once,
    // or maybe using the gpui font system, but gpui and its dependencies (like font-kit)
    // don't appear to read the font math table so cannot currently implement the `MathFont` trait
    // needed by ReX.
    let font =
        TtfMathFont::new(Face::parse(include_bytes!("../XITS_Math.otf"), 0).unwrap()).unwrap();

    let layout_engine = LayoutBuilder::new(&font)
        .font_size(font_size)
        .style(Style::Display)
        .build();

    let parse_nodes = parse_latex(equation).unwrap();
    let layout = layout_engine.layout(&parse_nodes).unwrap();
    let renderer = rex::Renderer::new();

    let mut backend = GPUIBackend::new(layout.size(), 1.);
    renderer.render(&layout, &mut backend);
    backend.paths_and_rects()
}

pub struct LatexElement {
    paths: Vec<Path<Pixels>>,
    rects: Vec<Bounds<Pixels>>,
    width: f32,
    height: f32,
}

impl LatexElement {
    pub fn new(equation: &str, font_size: f64) -> Self {
        use rex::font::backend::ttf_parser::ttf_parser_crate::Face;
        use rex::layout::{Style, engine::LayoutBuilder};
        use rex::parser::parse as parse_latex;

        // This font stuff would ultimately be better if only performed once,
        // or maybe using the gpui font system, but gpui and its dependencies (like font-kit)
        // don't appear to read the font math table so cannot currently implement the `MathFont` trait
        // needed by ReX.
        let font =
            TtfMathFont::new(Face::parse(include_bytes!("../XITS_Math.otf"), 0).unwrap()).unwrap();

        let layout_engine = LayoutBuilder::new(&font)
            .font_size(font_size)
            .style(Style::Display)
            .build();

        let parse_nodes = parse_latex(equation).unwrap();
        let layout = layout_engine.layout(&parse_nodes).unwrap();
        let width = layout.width.to_unitless() as f32;
        let height = (layout.depth - layout.height).to_unitless() as f32;
        let renderer = rex::Renderer::new();

        let mut backend = GPUIBackend::new(layout.size(), 1.);
        renderer.render(&layout, &mut backend);
        let (paths, rects) = backend.paths_and_rects();
        Self {
            paths,
            rects,
            height,
            width,
        }
    }
}

impl IntoElement for LatexElement {
    type Element = LatexElement;

    fn into_element(self) -> Self::Element {
        self
    }
}

impl Element for LatexElement {
    type RequestLayoutState = ();
    type PrepaintState = ();

    fn id(&self) -> Option<gpui::ElementId> {
        None
    }

    fn source_location(&self) -> Option<&'static Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _global_id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let size = Size::<Length> {
            width: px(self.width).into(),
            height: px(self.height).into(),
        };

        let layout_id = window.request_layout(
            gpui::Style {
                size,
                ..Default::default()
            },
            None,
            cx,
        );

        (layout_id, ())
    }

    fn prepaint(
        &mut self,
        _global_id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        _bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        _window: &mut Window,
        _cx: &mut App,
    ) -> Self::PrepaintState {
        ()
    }

    fn paint(
        &mut self,
        _global_id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        _prepaint_state: &mut Self::PrepaintState,
        window: &mut Window,
        _cx: &mut App,
    ) {
        // Paint all the rectangles (rules) first
        for rect in &self.rects {
            let translated_rect = Bounds::new(bounds.origin + rect.origin, rect.size);
            window.paint_quad(gpui::PaintQuad {
                bounds: translated_rect,
                background: rgb(0x000000).into(),
                border_color: rgb(0x000000).into(),
                border_widths: gpui::Edges::default(),
                corner_radii: Corners::default(),
                border_style: BorderStyle::Solid,
            });
        }

        // Paint all the paths (glyphs)
        for path in &self.paths {
            let mut translated_path = path.clone();
            // Translate the path to the correct position within the element's bounds
            for point in translated_path.vertices {}

            translated_path = translated_path.translate(bounds.origin);
            window.paint_path(translated_path, gpui::black());
        }
    }
}
