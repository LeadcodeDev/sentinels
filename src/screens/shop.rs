use gpui::*;
use gpui_component::Disableable;
use gpui_component::button::{Button, ButtonVariants};

use crate::app::{Screen, SentinelsApp};
use crate::data::{SHOP_UPGRADES, SaveData};

pub fn render(save_data: &mut SaveData, cx: &mut Context<SentinelsApp>) -> impl IntoElement {
    let total_gold = save_data.total_gold;

    let upgrades: Vec<_> = SHOP_UPGRADES
        .iter()
        .map(|def| {
            let current_level = save_data.get_upgrade_level(def.id);
            let can_buy = current_level < def.max_level && total_gold >= def.cost(current_level);
            (def, current_level, can_buy)
        })
        .collect();

    div()
        .size_full()
        .flex()
        .flex_col()
        .items_center()
        .gap_4()
        .p_6()
        .child(
            div()
                .w_full()
                .flex()
                .items_center()
                .justify_between()
                .child(Button::new("back").label("Retour").on_click(cx.listener(
                    |app, _, _window, cx| {
                        app.navigate_to(Screen::Lobby, cx);
                    },
                )))
                .child(
                    div()
                        .text_color(rgb(0xffd700))
                        .child(format!("Or: {}", total_gold)),
                ),
        )
        .child(
            div()
                .text_lg()
                .text_color(rgb(0xffffff))
                .child("Boutique - Ameliorations permanentes"),
        )
        .child(
            div()
                .flex()
                .flex_col()
                .gap_2()
                .w_full()
                .max_w(px(500.))
                .children(upgrades.into_iter().map(|(def, level, can_buy)| {
                    let id = def.id;
                    let cost = def.cost(level);

                    div()
                        .flex()
                        .items_center()
                        .justify_between()
                        .p_3()
                        .rounded_md()
                        .bg(rgb(0x2a2a4a))
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .gap_1()
                                .child(div().text_color(rgb(0xffffff)).text_sm().child(def.name))
                                .child(div().text_color(rgb(0x888888)).text_xs().child(format!(
                                    "{} (Niv. {}/{})",
                                    def.description, level, def.max_level
                                ))),
                        )
                        .child(if level >= def.max_level {
                            Button::new(SharedString::from(format!("max_{}", id)))
                                .label("MAX")
                                .disabled(true)
                                .into_any_element()
                        } else {
                            let id_owned = id.to_string();
                            Button::new(SharedString::from(format!("buy_{}", id)))
                                .label(format!("{} or", cost))
                                .disabled(!can_buy)
                                .on_click(cx.listener(move |app, _, _window, cx| {
                                    app.save_data.purchase_upgrade(&id_owned, cost);
                                    cx.notify();
                                }))
                                .into_any_element()
                        })
                })),
        )
}
