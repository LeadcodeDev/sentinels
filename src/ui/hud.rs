use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::Disableable;
use gpui_component::button::Button;
use gpui_component::tooltip::Tooltip;

use crate::data::tower_presets::get_preset;
use crate::game::elemental::TowerElement;
use crate::game::{GamePhase, GameState};
use crate::screens::play::PlayScreen;

const SIDEBAR_WIDTH: f32 = 200.0;

pub fn sidebar_width() -> f32 {
    SIDEBAR_WIDTH
}

pub fn render_sidebar(game: &GameState, cx: &mut Context<PlayScreen>) -> impl IntoElement + use<> {
    let hp = game.player.hp;
    let max_hp = game.player.max_hp;
    let gold = game.economy.gold;
    let pepites = game.economy.pepites;
    let wave = game.economy.wave_number;
    let score = game.economy.score;
    let phase = game.phase;
    let tower_count = game.towers.len();
    let player_gold = game.economy.gold;

    let selected_section = selected_tower_section(game, cx);

    div()
        .w(px(SIDEBAR_WIDTH))
        .h_full()
        .flex_shrink_0()
        .flex()
        .flex_col()
        .bg(Hsla {
            h: 0.0,
            s: 0.0,
            l: 0.08,
            a: 0.95,
        })
        .border_l_1()
        .border_color(Hsla {
            h: 0.0,
            s: 0.0,
            l: 0.2,
            a: 1.0,
        })
        .p_3()
        .gap_3()
        // Stats section
        .child(stats_section(
            hp,
            max_hp,
            gold,
            pepites,
            wave,
            score,
            tower_count,
        ))
        // Tower grid section
        .child(tower_grid_section(player_gold, cx))
        // Selected tower section (scrollable)
        .when_some(selected_section, |this, section| this.child(section))
        // Bottom: wave button or game over
        .child(
            div()
                .flex_shrink_0()
                .flex()
                .flex_col()
                .when(phase == GamePhase::Preparing, |this| {
                    this.child(Button::new("start_wave").label("Lancer la vague").on_click(
                        cx.listener(|screen, _, _window, _cx| {
                            screen.game_state.start_wave();
                        }),
                    ))
                })
                .when(phase == GamePhase::GameOver, |this| {
                    this.child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_2()
                            .items_center()
                            .child(div().text_sm().text_color(rgb(0xff4444)).child("GAME OVER"))
                            .child(Button::new("back_lobby").label("Retour au lobby").on_click(
                                cx.listener(|screen, _, _window, cx| {
                                    screen.game_running = false;
                                    cx.emit(crate::screens::play::PlayScreenEvent::ReturnToLobby);
                                }),
                            )),
                    )
                }),
        )
}

fn stats_section(
    hp: f32,
    max_hp: f32,
    gold: u32,
    pepites: u32,
    wave: u32,
    score: u32,
    tower_count: usize,
) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .gap_2()
        // HP bar
        .child(
            div()
                .flex()
                .flex_col()
                .gap_1()
                .child(div().text_xs().text_color(rgb(0xaaaaaa)).child("PV"))
                .child(
                    div()
                        .w_full()
                        .h(px(8.0))
                        .rounded_sm()
                        .bg(rgb(0x333333))
                        .child(
                            div()
                                .h_full()
                                .rounded_sm()
                                .bg(rgb(0xff4444))
                                .w(relative(hp / max_hp)),
                        ),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(rgb(0xff4444))
                        .child(format!("{:.0}/{:.0}", hp, max_hp)),
                ),
        )
        // Gold
        .child(
            div()
                .flex()
                .items_center()
                .gap_2()
                .child(div().text_xs().text_color(rgb(0xaaaaaa)).child("Or"))
                .child(
                    div()
                        .text_sm()
                        .text_color(rgb(0xffd700))
                        .child(format!("{}", gold)),
                ),
        )
        // Pepites
        .child(
            div()
                .flex()
                .items_center()
                .gap_2()
                .child(div().text_xs().text_color(rgb(0xaaaaaa)).child("Pepites"))
                .child(
                    div()
                        .text_sm()
                        .text_color(rgb(0xcc66ff))
                        .child(format!("{}", pepites)),
                ),
        )
        // Wave
        .child(
            div()
                .flex()
                .items_center()
                .gap_2()
                .child(div().text_xs().text_color(rgb(0xaaaaaa)).child("Vague"))
                .child(
                    div()
                        .text_sm()
                        .text_color(rgb(0xffffff))
                        .child(format!("{}", wave)),
                ),
        )
        // Score
        .child(
            div()
                .flex()
                .items_center()
                .gap_2()
                .child(div().text_xs().text_color(rgb(0xaaaaaa)).child("Score"))
                .child(
                    div()
                        .text_sm()
                        .text_color(rgb(0xffffff))
                        .child(format!("{}", score)),
                ),
        )
        // Tower count
        .child(
            div()
                .flex()
                .items_center()
                .gap_2()
                .child(div().text_xs().text_color(rgb(0xaaaaaa)).child("Tours"))
                .child(
                    div()
                        .text_sm()
                        .text_color(rgb(0xffffff))
                        .child(format!("{}", tower_count)),
                ),
        )
}

fn tower_grid_section(gold: u32, cx: &mut Context<PlayScreen>) -> impl IntoElement + use<> {
    let neutral = tower_icon(TowerElement::Neutral, gold, cx);
    let fire = tower_icon(TowerElement::Fire, gold, cx);
    let water = tower_icon(TowerElement::Water, gold, cx);
    let electric = tower_icon(TowerElement::Electric, gold, cx);
    let earth = tower_icon(TowerElement::Earth, gold, cx);

    div()
        .flex()
        .flex_col()
        .gap_2()
        .child(
            div()
                .text_xs()
                .text_color(rgb(0xaaaaaa))
                .child("-- Tours --"),
        )
        .child(
            div()
                .flex()
                .flex_wrap()
                .gap_2()
                .child(neutral)
                .child(fire)
                .child(water)
                .child(electric)
                .child(earth),
        )
}

fn selected_tower_section(
    game: &GameState,
    cx: &mut Context<PlayScreen>,
) -> Option<impl IntoElement + use<>> {
    let tower_idx = game.selected_tower?;
    let tower = game.towers.get(tower_idx)?;

    let color = tower.element.color();
    let element_name = tower.element.name();
    let level = tower.level;
    let damage = tower.attack_damage;
    let range = tower.attack_range;
    let speed = tower.attack_speed;
    let is_aoe = tower.is_aoe;
    let aoe_radius = tower.aoe_radius;
    let sell_value = tower.sell_value();
    let gold = game.economy.gold;

    let upgrades: Vec<_> = tower
        .upgrades
        .iter()
        .map(|u| {
            let can_afford = u.level < u.max_level && gold >= u.cost();
            (
                u.upgrade_type,
                u.level,
                u.max_level,
                u.cost(),
                u.bonus_per_level(),
                can_afford,
            )
        })
        .collect();

    let mut upgrade_elements: Vec<AnyElement> = Vec::new();
    for &(upgrade_type, ulevel, max_level, cost, bonus, can_afford) in &upgrades {
        let is_maxed = ulevel >= max_level;
        let label = if is_maxed {
            format!("{} MAX", upgrade_type.name())
        } else {
            format!("{} +{:.0} ({}g)", upgrade_type.name(), bonus, cost)
        };
        let id = SharedString::from(format!("sidebar_upgrade_{:?}", upgrade_type));

        let row = div()
            .flex()
            .items_center()
            .justify_between()
            .child(
                div()
                    .text_xs()
                    .text_color(rgb(0x888888))
                    .child(format!("{}/{}", ulevel, max_level)),
            )
            .child(
                Button::new(id)
                    .label(label)
                    .disabled(is_maxed || !can_afford)
                    .on_click(cx.listener(move |screen, _, _window, _cx| {
                        if let Some(idx) = screen.game_state.selected_tower {
                            screen.game_state.upgrade_tower(idx, upgrade_type);
                        }
                    })),
            );
        upgrade_elements.push(row.into_any_element());
    }

    let sell_btn = Button::new("sidebar_sell_tower")
        .label(format!("Vendre ({}g)", sell_value))
        .on_click(cx.listener(move |screen, _, _window, _cx| {
            if let Some(idx) = screen.game_state.selected_tower {
                screen.game_state.sell_tower(idx);
            }
        }));

    Some(
        div()
            .id("selected_tower_panel")
            .flex_1()
            .overflow_y_scroll()
            .flex()
            .flex_col()
            .gap_2()
            .pt_2()
            .border_t_1()
            .border_color(Hsla {
                h: 0.0,
                s: 0.0,
                l: 0.25,
                a: 1.0,
            })
            // Header
            .child(
                div()
                    .text_sm()
                    .text_color(color)
                    .child(format!("{} Nv.{}", element_name, level)),
            )
            // Stats
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .text_xs()
                    .text_color(rgb(0xcccccc))
                    .child(format!("Degats: {:.1}", damage))
                    .child(format!("Portee: {:.0}", range))
                    .child(format!("Vitesse: {:.2}/s", speed))
                    .when(is_aoe, |this| {
                        this.child(format!("Zone: {:.0}", aoe_radius))
                    }),
            )
            // Upgrades
            .child(
                div()
                    .text_xs()
                    .text_color(rgb(0xaaaaaa))
                    .child("Ameliorations"),
            )
            .children(upgrade_elements)
            // Sell
            .child(sell_btn),
    )
}

fn tower_icon(
    element: TowerElement,
    gold: u32,
    cx: &mut Context<PlayScreen>,
) -> impl IntoElement + use<> {
    let preset = get_preset(element);
    let cost = preset.base_cost;
    let can_afford = gold >= cost;
    let color = element.color();
    let name = preset.name;

    let bg_alpha = if can_afford { 0.2 } else { 0.05 };
    let border_alpha = if can_afford { 0.8 } else { 0.3 };

    div()
        .id(SharedString::from(format!("tower_icon_{:?}", element)))
        .w(px(50.0))
        .h(px(50.0))
        .flex()
        .items_center()
        .justify_center()
        .rounded_md()
        .cursor_pointer()
        .bg(Hsla {
            h: color.h,
            s: color.s,
            l: color.l,
            a: bg_alpha,
        })
        .border_1()
        .border_color(Hsla {
            h: color.h,
            s: color.s,
            l: color.l,
            a: border_alpha,
        })
        .child(
            // Diamond symbol preview (rotated square via Unicode)
            div()
                .text_lg()
                .text_color(Hsla {
                    h: color.h,
                    s: color.s,
                    l: color.l,
                    a: if can_afford { 1.0 } else { 0.4 },
                })
                .child("\u{25C6}"), // ◆ diamond character
        )
        .tooltip(move |window, cx| {
            Tooltip::new(format!("{} - {} or", name, cost)).build(window, cx)
        })
        .on_mouse_down(
            MouseButton::Left,
            cx.listener(move |screen, _, _window, _cx| {
                if can_afford {
                    screen.game_state.placement_mode = Some(element);
                }
            }),
        )
}
