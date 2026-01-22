use gpui::*;
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::v_flex;

use crate::app::{Screen, SentinelsApp};
use crate::data::SaveData;

pub fn render(save_data: &SaveData, cx: &mut Context<SentinelsApp>) -> impl IntoElement {
    let best_score = save_data.best_score;
    let max_wave = save_data.max_wave;

    v_flex()
        .size_full()
        .items_center()
        .justify_center()
        .gap_6()
        .child(div().text_xl().text_color(rgb(0xffffff)).child("SENTINELS"))
        .child(div().text_sm().text_color(rgb(0xaaaaaa)).child(format!(
            "Meilleur score: {} | Vague max: {}",
            best_score, max_wave
        )))
        .child(
            v_flex()
                .gap_3()
                .items_center()
                .child(
                    Button::new("play")
                        .primary()
                        .label("Jouer")
                        .on_click(cx.listener(|app, _, _window, cx| {
                            app.navigate_to(Screen::Play, cx);
                        })),
                )
                .child(
                    Button::new("shop")
                        .ghost()
                        .label("Boutique")
                        .on_click(cx.listener(|app, _, _window, cx| {
                            app.navigate_to(Screen::Shop, cx);
                        })),
                )
                .child(
                    Button::new("quit")
                        .danger()
                        .label("Quitter")
                        .on_click(|_, _window, cx| {
                            cx.quit();
                        }),
                ),
        )
}
