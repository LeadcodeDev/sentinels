mod app;
mod data;
mod game;
mod render;
mod screens;
mod ui;

use gpui::*;

fn main() {
    let app = Application::new().with_assets(gpui_component_assets::Assets);

    app.run(move |cx| {
        gpui_component::init(cx);

        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
                    None,
                    size(px(1200.), px(800.)),
                    cx,
                ))),
                titlebar: Some(TitlebarOptions {
                    title: Some("Sentinels".into()),
                    ..Default::default()
                }),
                ..Default::default()
            },
            |_window, cx| {
                let view = cx.new(|cx| app::SentinelsApp::new(cx));
                let any_view: AnyView = view.into();
                cx.new(|cx| gpui_component::Root::new(any_view, _window, cx))
            },
        )
        .unwrap();
    });
}
