use gpui::{
    Application, BorderStyle, Bounds, Context, Corners, PaintQuad, Path, Pixels, Render, Window,
    WindowBounds, WindowOptions, canvas, div, prelude::*, px, rgb, size,
};
mod math;
use math::latex_to_paths;

const DEFAULT_WINDOW_WIDTH: Pixels = px(1024.0);
const DEFAULT_WINDOW_HEIGHT: Pixels = px(768.0);

struct PaintingViewer {
    paths: Vec<Path<Pixels>>,
    rects: Vec<Bounds<Pixels>>,
}

impl PaintingViewer {
    fn new(_window: &mut Window, _cx: &mut Context<Self>) -> Self {
        let (paths, rects) = latex_to_paths(
            r"e = \lim_{n \to \infty} \left(1 + \frac{1}{n}\right)^n",
            40.,
        );
        Self { paths, rects }
    }
}

impl Render for PaintingViewer {
    fn render(&mut self, window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        window.request_animation_frame();

        let paths = self.paths.clone();
        let rects = self.rects.clone();

        let window_size = window.bounds().size;
        let scale = window_size.width / DEFAULT_WINDOW_WIDTH;

        div()
            .font_family(".SystemUIFont")
            .bg(gpui::white())
            .size_full()
            .p_4()
            .child(
                div().size_full().child(
                    canvas(
                        move |_, _, _| {}, // TODO Find out how to reserve specific size for canvas
                        move |_, _, window, _| {
                            // TODO position within given bounds
                            for path in paths {
                                window.paint_path(path.clone().scale(scale), rgb(0x000000));
                            }
                            for rect in rects {
                                window.paint_quad(PaintQuad {
                                    bounds: rect,
                                    background: rgb(0x000000).into(),
                                    border_color: rgb(0x000000).into(),
                                    border_widths: gpui::Edges::default(),
                                    corner_radii: Corners::default(),
                                    border_style: BorderStyle::Solid,
                                });
                            }
                        },
                    )
                    .size_full(),
                ),
            )
    }
}

fn main() {
    Application::new().run(|cx| {
        cx.open_window(
            WindowOptions {
                focus: true,
                window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
                    None,
                    size(DEFAULT_WINDOW_WIDTH, DEFAULT_WINDOW_HEIGHT),
                    cx,
                ))),
                ..Default::default()
            },
            |window, cx| cx.new(|cx| PaintingViewer::new(window, cx)),
        )
        .unwrap();
        cx.activate(true);
    });
}
