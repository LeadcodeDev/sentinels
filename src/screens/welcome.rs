use gpui::*;
use gpui_component::v_flex;
use std::time::Duration;

use crate::app::{Screen, SentinelsApp};

pub fn render(cx: &mut Context<SentinelsApp>) -> impl IntoElement {
    cx.spawn(async |this: WeakEntity<SentinelsApp>, cx| {
        Timer::after(Duration::from_secs(2)).await;
        this.update(cx, |app, cx| {
            app.navigate_to(Screen::Lobby, cx);
        })
        .ok();
    })
    .detach();

    v_flex()
        .size_full()
        .items_center()
        .justify_center()
        .gap_4()
        .child(div().text_xl().text_color(rgb(0xffffff)).child("SENTINELS"))
        .child(
            div()
                .text_sm()
                .text_color(rgb(0x888888))
                .child("Chargement..."),
        )
}
