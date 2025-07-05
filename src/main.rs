use gpui::{
    Application, Background, Bounds, Context, Path, PathBuilder, Pixels, Render, Window,
    WindowBounds, WindowOptions, canvas, div, point, prelude::*, px, rgb, size,
};
mod math;
use math::latex_to_paths;

const DEFAULT_WINDOW_WIDTH: Pixels = px(1024.0);
const DEFAULT_WINDOW_HEIGHT: Pixels = px(768.0);

struct PaintingViewer {
    default_lines: Vec<Path<Pixels>>,
    _painting: bool,
}

impl PaintingViewer {
    fn new(_window: &mut Window, _cx: &mut Context<Self>) -> Self {
        let lines = latex_to_paths();
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
                        move |_, _, _| gpui::Size::new(Pixels(500.), Pixels(500.)), // Find out how to specify the size of the canvas
                        move |_, _, window, _| {
                            for path in default_lines {
                                window.paint_path(path.clone().scale(scale), rgb(0x000000));
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
