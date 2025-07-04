use gpui::{
    Application, Background, Bounds, Context, Path, PathBuilder, Pixels, Render, Window,
    WindowBounds, WindowOptions, canvas, div, point, prelude::*, px, rgb, size,
};

const DEFAULT_WINDOW_WIDTH: Pixels = px(1024.0);
const DEFAULT_WINDOW_HEIGHT: Pixels = px(768.0);

struct PaintingViewer {
    default_lines: Vec<(Path<Pixels>, Background)>,
    _painting: bool,
}

impl PaintingViewer {
    fn new(_window: &mut Window, _cx: &mut Context<Self>) -> Self {
        let mut lines = vec![];

        // draw a lightening bolt ⚡
        let mut builder = PathBuilder::fill();
        builder.add_polygon(
            &[
                point(px(150.), px(200.)),
                point(px(200.), px(125.)),
                point(px(200.), px(175.)),
                point(px(250.), px(100.)),
            ],
            false,
        );
        let path = builder.build().unwrap();
        lines.push((path, rgb(0x1d4ed8).into()));

        Self {
            default_lines: lines.clone(),
            _painting: false,
        }
    }
}

impl Render for PaintingViewer {
    fn render(&mut self, window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        window.request_animation_frame();

        let default_lines = self.default_lines.clone();
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
                        move |_, _, _| {},
                        move |_, _, window, _| {
                            for (path, color) in default_lines {
                                window.paint_path(path.clone().scale(scale), color);
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
