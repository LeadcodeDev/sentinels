use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::button::{Button, ButtonCustomVariant, ButtonVariants};
use gpui_component::divider::Divider;
use gpui_component::progress::Progress;
use gpui_component::{Disableable, Sizable, Size, h_flex, v_flex};

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

    v_flex()
        .w(px(SIDEBAR_WIDTH))
        .h_full()
        .flex_shrink_0()
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
            v_flex()
                .flex_shrink_0()
                .when(phase == GamePhase::Preparing, |this| {
                    this.child(
                        Button::new("start_wave")
                            .primary()
                            .label("Lancer la vague")
                            .on_click(cx.listener(|screen, _, _window, _cx| {
                                screen.game_state.start_wave();
                            })),
                    )
                })
                .when(phase == GamePhase::GameOver, |this| {
                    this.child(
                        v_flex()
                            .gap_2()
                            .items_center()
                            .child(div().text_sm().text_color(rgb(0xff4444)).child("GAME OVER"))
                            .child(
                                Button::new("back_lobby")
                                    .danger()
                                    .label("Retour au lobby")
                                    .on_click(cx.listener(|screen, _, _window, cx| {
                                        screen.game_running = false;
                                        cx.emit(
                                            crate::screens::play::PlayScreenEvent::ReturnToLobby,
                                        );
                                    })),
                            ),
                    )
                }),
        )
}

fn stat_row(label: &'static str, value: String, color: impl Into<Hsla>) -> impl IntoElement {
    h_flex()
        .items_center()
        .justify_between()
        .child(div().text_xs().text_color(rgb(0xaaaaaa)).child(label))
        .child(div().text_sm().text_color(color.into()).child(value))
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
    v_flex()
        .gap_2()
        // HP bar using Progress component
        .child(
            v_flex()
                .gap_1()
                .child(div().text_xs().text_color(rgb(0xaaaaaa)).child("PV"))
                .child(
                    Progress::new()
                        .value((hp / max_hp) * 100.0)
                        .bg(rgb(0xff4444)),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(rgb(0xff4444))
                        .child(format!("{:.0}/{:.0}", hp, max_hp)),
                ),
        )
        // Stats using consistent stat_row helper
        .child(stat_row("Or", format!("{}", gold), rgb(0xffd700)))
        .child(stat_row("Pepites", format!("{}", pepites), rgb(0xcc66ff)))
        .child(stat_row("Vague", format!("{}", wave), rgb(0xffffff)))
        .child(stat_row("Score", format!("{}", score), rgb(0xffffff)))
        .child(stat_row("Tours", format!("{}", tower_count), rgb(0xffffff)))
}

fn tower_grid_section(gold: u32, cx: &mut Context<PlayScreen>) -> impl IntoElement + use<> {
    let neutral = tower_icon(TowerElement::Neutral, gold, cx);
    let fire = tower_icon(TowerElement::Fire, gold, cx);
    let water = tower_icon(TowerElement::Water, gold, cx);
    let electric = tower_icon(TowerElement::Electric, gold, cx);
    let earth = tower_icon(TowerElement::Earth, gold, cx);

    v_flex()
        .gap_2()
        .child(Divider::horizontal().color(Hsla {
            h: 0.0,
            s: 0.0,
            l: 0.25,
            a: 1.0,
        }))
        .child(div().text_xs().text_color(rgb(0xaaaaaa)).child("Tours"))
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

        let row = h_flex()
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
                    .compact()
                    .with_size(Size::Small)
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
        .danger()
        .label(format!("Vendre ({}g)", sell_value))
        .compact()
        .with_size(Size::Small)
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
            .child(Divider::horizontal().color(Hsla {
                h: 0.0,
                s: 0.0,
                l: 0.25,
                a: 1.0,
            }))
            // Header
            .child(
                div()
                    .text_sm()
                    .text_color(color)
                    .child(format!("{} Nv.{}", element_name, level)),
            )
            // Stats
            .child(
                v_flex()
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
            // Upgrades header
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

    let bg_color = Hsla {
        h: color.h,
        s: color.s,
        l: color.l,
        a: if can_afford { 0.2 } else { 0.05 },
    };
    let border_color = Hsla {
        h: color.h,
        s: color.s,
        l: color.l,
        a: if can_afford { 0.8 } else { 0.3 },
    };
    let hover_color = Hsla {
        h: color.h,
        s: color.s,
        l: color.l,
        a: if can_afford { 0.35 } else { 0.05 },
    };
    let fg_color = Hsla {
        h: color.h,
        s: color.s,
        l: color.l,
        a: if can_afford { 1.0 } else { 0.4 },
    };

    Button::new(SharedString::from(format!("tower_icon_{:?}", element)))
        .custom(
            ButtonCustomVariant::new(cx)
                .color(bg_color)
                .foreground(fg_color)
                .border(border_color)
                .hover(hover_color)
                .active(hover_color),
        )
        .label("\u{25C6}")
        .disabled(!can_afford)
        .tooltip(SharedString::from(format!("{} - {} or", name, cost)))
        .on_click(cx.listener(move |screen, _, _window, _cx| {
            if can_afford {
                screen.game_state.placement_mode = Some(element);
            }
        }))
}
