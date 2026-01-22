use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::Disableable;
use gpui_component::button::{Button, ButtonVariants};

use crate::game::elemental::TowerElement;
use crate::game::{GamePhase, GameState};
use crate::screens::play::PlayScreen;

pub fn render_hud(game: &GameState, cx: &mut Context<PlayScreen>) -> impl IntoElement {
    let hp = game.player.hp;
    let max_hp = game.player.max_hp;
    let gold = game.economy.gold;
    let wave = game.economy.wave_number;
    let score = game.economy.score;
    let phase = game.phase;

    div()
        .size_full()
        .absolute()
        .top_0()
        .left_0()
        // Top bar
        .child(
            div()
                .absolute()
                .top_0()
                .left_0()
                .w_full()
                .p_4()
                .flex()
                .items_center()
                .justify_between()
                .child(
                    div()
                        .flex()
                        .gap_4()
                        .items_center()
                        .child(
                            div()
                                .text_sm()
                                .text_color(rgb(0xff4444))
                                .child(format!("PV: {:.0}/{:.0}", hp, max_hp)),
                        )
                        .child(
                            div()
                                .text_sm()
                                .text_color(rgb(0xffd700))
                                .child(format!("Or: {}", gold)),
                        ),
                )
                .child(
                    div()
                        .flex()
                        .gap_4()
                        .items_center()
                        .child(
                            div()
                                .text_sm()
                                .text_color(rgb(0xaaaaaa))
                                .child(format!("Vague: {}", wave)),
                        )
                        .child(
                            div()
                                .text_sm()
                                .text_color(rgb(0xffffff))
                                .child(format!("Score: {}", score)),
                        ),
                ),
        )
        // Bottom bar - tower placement
        .child(
            div()
                .absolute()
                .bottom_0()
                .left_0()
                .w_full()
                .p_4()
                .flex()
                .justify_center()
                .gap_3()
                .child(tower_button(TowerElement::Neutral, gold, cx))
                .child(tower_button(TowerElement::Fire, gold, cx))
                .child(tower_button(TowerElement::Water, gold, cx))
                .child(tower_button(TowerElement::Electric, gold, cx))
                .child(tower_button(TowerElement::Earth, gold, cx))
                .when(phase == GamePhase::Preparing, |this| {
                    this.child(
                        Button::new("start_wave")
                            .primary()
                            .label("Lancer la vague")
                            .on_click(cx.listener(|screen, _, _window, _cx| {
                                screen.game_state.start_wave();
                            })),
                    )
                }),
        )
        // Game over overlay
        .when(phase == GamePhase::GameOver, |this| {
            this.child(
                div()
                    .size_full()
                    .absolute()
                    .top_0()
                    .left_0()
                    .flex()
                    .flex_col()
                    .items_center()
                    .justify_center()
                    .gap_4()
                    .bg(Hsla {
                        h: 0.0,
                        s: 0.0,
                        l: 0.0,
                        a: 0.7,
                    })
                    .child(div().text_xl().text_color(rgb(0xff4444)).child("GAME OVER"))
                    .child(
                        div()
                            .text_color(rgb(0xffffff))
                            .child(format!("Score: {} | Vague: {}", score, wave)),
                    )
                    .child(Button::new("back_lobby").label("Retour au lobby").on_click(
                        cx.listener(|screen, _, _window, cx| {
                            screen.game_running = false;
                            cx.emit(crate::screens::play::PlayScreenEvent::ReturnToLobby);
                        }),
                    )),
            )
        })
}

fn tower_button(
    element: TowerElement,
    gold: u32,
    cx: &mut Context<PlayScreen>,
) -> impl IntoElement {
    use crate::data::tower_presets::get_preset;

    let preset = get_preset(element);
    let cost = preset.base_cost;
    let can_afford = gold >= cost;
    let name = element.name();

    Button::new(SharedString::from(format!("tower_{:?}", element)))
        .label(format!("{} ({})", name, cost))
        .disabled(!can_afford)
        .on_click(cx.listener(move |screen, _, _window, _cx| {
            screen.game_state.placement_mode = Some(element);
        }))
}
