use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::button::{Button, ButtonCustomVariant, ButtonVariants};
use gpui_component::checkbox::Checkbox;
use gpui_component::divider::Divider;
use gpui_component::progress::Progress;
use gpui_component::{Disableable, Sizable, Size, h_flex, v_flex};

use crate::data::tower_defs::{SkillType, TowerKind, get_def};
use crate::game::tower::SkillState;
use crate::game::{GamePhase, GameState};
use crate::screens::play::PlayScreen;

const SIDEBAR_MIN_WIDTH: f32 = 200.0;

/// Calcule la largeur de la sidebar (1/6 de l'écran, minimum 200px)
pub fn sidebar_width(screen_width: f32) -> f32 {
    (screen_width / 6.0).max(SIDEBAR_MIN_WIDTH)
}

pub fn render_sidebar(
    game: &GameState,
    selected_skill_tab: Option<usize>,
    cx: &mut Context<PlayScreen>,
) -> impl IntoElement + use<> {
    let hp = game.player.hp;
    let max_hp = game.player.max_hp;
    let phase = game.phase;
    let tower_count = game.towers.len();
    let max_towers = game.max_towers;
    let player_gold = game.economy.gold;
    let shield = game.shield.clone();
    let current_sidebar_width = sidebar_width(game.viewport_size.0);

    let selected_section = selected_tower_section(game, selected_skill_tab, cx);

    v_flex()
        .w(px(current_sidebar_width))
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
        // Stats section (only HP and shield now)
        .child(stats_section(hp, max_hp, &shield))
        // Tower grid section
        .child(tower_grid_section(player_gold, tower_count, max_towers, cx))
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
                }),
        )
}

/// Renders the top-left overlay with Wave, Gold, Pepites
pub fn render_top_left_stats(game: &GameState) -> impl IntoElement {
    let gold = game.economy.gold;
    let pepites = game.economy.pepites;
    let wave = game.economy.wave_number;

    div()
        .absolute()
        .top_2()
        .left_2()
        .flex()
        .gap_3()
        .child(
            h_flex()
                .items_center()
                .gap_1()
                .child(div().text_xs().text_color(rgb(0x888888)).child("Vague"))
                .child(
                    div()
                        .text_sm()
                        .text_color(rgb(0xffffff))
                        .child(format!("{}", wave)),
                ),
        )
        .child(
            h_flex()
                .items_center()
                .gap_1()
                .child(div().text_xs().text_color(rgb(0x888888)).child("Or"))
                .child(
                    div()
                        .text_sm()
                        .text_color(rgb(0xffd700))
                        .child(format!("{}", gold)),
                ),
        )
        .child(
            h_flex()
                .items_center()
                .gap_1()
                .child(div().text_xs().text_color(rgb(0x888888)).child("Pepites"))
                .child(
                    div()
                        .text_sm()
                        .text_color(rgb(0xcc66ff))
                        .child(format!("{}", pepites)),
                ),
        )
}

fn stats_section(hp: f32, max_hp: f32, shield: &crate::game::Shield) -> impl IntoElement {
    let shield_unlocked = shield.is_unlocked();
    let shield_active = shield.active;
    let shield_hp = shield.hp;
    let shield_max_hp = shield.max_hp;
    let shield_regen_timer = shield.regen_timer;

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
        // Shield bar (only if unlocked)
        .when(shield_unlocked, |this| {
            this.child(
                v_flex()
                    .gap_1()
                    .child(div().text_xs().text_color(rgb(0xaaaaaa)).child("Bouclier"))
                    .child(
                        Progress::new()
                            .value(if shield_active {
                                (shield_hp / shield_max_hp) * 100.0
                            } else {
                                0.0
                            })
                            .bg(rgb(0x44ccdd)),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(0x44ccdd))
                            .child(if shield_active {
                                format!("{:.0}/{:.0}", shield_hp, shield_max_hp)
                            } else {
                                format!("Regen {:.0}s", shield_regen_timer)
                            }),
                    ),
            )
        })
}

fn tower_grid_section(
    gold: u32,
    tower_count: usize,
    max_towers: u32,
    cx: &mut Context<PlayScreen>,
) -> impl IntoElement + use<> {
    let slots_full = tower_count >= max_towers as usize;

    let mut tower_icons: Vec<AnyElement> = Vec::new();
    for kind in TowerKind::all() {
        tower_icons.push(tower_icon(*kind, gold, slots_full, cx).into_any_element());
    }

    v_flex()
        .gap_2()
        .child(Divider::horizontal().color(Hsla {
            h: 0.0,
            s: 0.0,
            l: 0.25,
            a: 1.0,
        }))
        .child(
            h_flex()
                .items_center()
                .justify_between()
                .child(div().text_xs().text_color(rgb(0xaaaaaa)).child("Tours"))
                .child(
                    div()
                        .text_xs()
                        .text_color(if slots_full {
                            rgb(0xff6666)
                        } else {
                            rgb(0x888888)
                        })
                        .child(format!("{}/{}", tower_count, max_towers)),
                ),
        )
        .child(div().flex().flex_wrap().gap_2().children(tower_icons))
}

fn selected_tower_section(
    game: &GameState,
    selected_skill_tab: Option<usize>,
    cx: &mut Context<PlayScreen>,
) -> Option<impl IntoElement + use<>> {
    let tower_idx = game.selected_tower?;
    let tower = game.towers.get(tower_idx)?;

    let color = tower.element.color();
    let name = tower.name;
    let sell_value = tower.sell_value();
    let gold = game.economy.gold;
    let has_notification_settings = tower.notification_settings.is_some();

    // Build skill icons row
    let def = get_def(tower.kind);
    let skill_count = def.skills.len();
    let selected_skill_idx = selected_skill_tab.unwrap_or(tower.active_skill_index.unwrap_or(0));
    let skill_icons: Vec<AnyElement> = (0..skill_count)
        .map(|skill_idx| {
            let skill_def = &def.skills[skill_idx];
            let skill_state = &tower.skills[skill_idx];
            let skill_icon = skill_def.icon;
            let skill_name = skill_def.name;
            let skill_type = skill_def.skill_type;
            let purchase_cost = skill_def.purchase_cost;

            let is_locked = matches!(skill_state, SkillState::Locked);
            let is_active = matches!(skill_state, SkillState::Active(_));
            let is_selected_tab = skill_idx == selected_skill_idx;

            // Determine button state and color
            let (bg_alpha, tooltip_text) = match skill_state {
                SkillState::Locked => (0.05, format!("{} - {}g", skill_name, purchase_cost)),
                SkillState::Purchased(_) => (0.2, format!("{}", skill_name)),
                SkillState::Active(_) => (0.4, format!("{} (active)", skill_name)),
            };

            // Color based on skill type
            let base_color = match skill_type {
                SkillType::Active => rgb(0x44aaff),  // Blue for active skills
                SkillType::Passive => rgb(0xffaa44), // Orange for passive skills
            };
            let base_hsla: Hsla = base_color.into();

            // Locked skills: no border, dimmed
            let (bg_color, border_color, hover_color, fg_color) = if is_locked {
                (
                    Hsla {
                        h: 0.0,
                        s: 0.0,
                        l: 0.15,
                        a: 0.5,
                    },
                    Hsla {
                        h: 0.0,
                        s: 0.0,
                        l: 0.0,
                        a: 0.0, // No border for locked
                    },
                    Hsla {
                        h: 0.0,
                        s: 0.0,
                        l: 0.2,
                        a: 0.6,
                    },
                    Hsla {
                        h: 0.0,
                        s: 0.0,
                        l: 0.5,
                        a: if gold >= purchase_cost { 0.8 } else { 0.4 },
                    },
                )
            } else if is_active {
                (
                    Hsla {
                        a: 0.5,
                        ..base_hsla
                    },
                    Hsla {
                        a: 1.0,
                        ..base_hsla
                    },
                    Hsla {
                        a: 0.6,
                        ..base_hsla
                    },
                    Hsla {
                        a: 1.0,
                        ..base_hsla
                    },
                )
            } else if is_selected_tab {
                // Purchased and selected tab
                (
                    Hsla {
                        a: 0.3,
                        ..base_hsla
                    },
                    Hsla {
                        a: 0.8,
                        ..base_hsla
                    },
                    Hsla {
                        a: 0.4,
                        ..base_hsla
                    },
                    Hsla {
                        a: 1.0,
                        ..base_hsla
                    },
                )
            } else {
                // Purchased but not selected
                (
                    Hsla {
                        a: bg_alpha,
                        ..base_hsla
                    },
                    Hsla {
                        a: 0.4,
                        ..base_hsla
                    },
                    Hsla {
                        a: bg_alpha + 0.15,
                        ..base_hsla
                    },
                    Hsla {
                        a: 0.8,
                        ..base_hsla
                    },
                )
            };

            // Label: add lock icon for locked skills
            let label_text = if is_locked {
                format!("\u{1F512}{}", skill_icon) // 🔒 + icon
            } else {
                skill_icon.to_string()
            };

            Button::new(SharedString::from(format!(
                "skill_icon_{}_{}",
                tower_idx, skill_idx
            )))
            .custom(
                ButtonCustomVariant::new(cx)
                    .color(bg_color)
                    .foreground(fg_color)
                    .border(border_color)
                    .hover(hover_color)
                    .active(hover_color),
            )
            .label(label_text)
            .tooltip(SharedString::from(tooltip_text))
            .on_click(cx.listener(move |screen, _, _window, _cx| {
                // Clicking on any tab just selects it to view its details
                screen.selected_skill_tab = Some(skill_idx);
            }))
            .into_any_element()
        })
        .collect();

    // Get the selected skill tab (defaults to active skill)
    let viewed_skill_idx = selected_skill_idx;
    let viewed_skill_def = &def.skills[viewed_skill_idx];
    let viewed_skill_state = &tower.skills[viewed_skill_idx];
    let viewed_skill_name = viewed_skill_def.name;
    let viewed_skill_desc = viewed_skill_def.description;
    let viewed_skill_cost = viewed_skill_def.purchase_cost;
    let is_viewed_skill_locked = matches!(viewed_skill_state, SkillState::Locked);
    let is_viewed_skill_active = matches!(viewed_skill_state, SkillState::Active(_));
    let is_viewed_skill_purchased = viewed_skill_state.is_purchased();
    let can_afford_skill = gold >= viewed_skill_cost;

    // Get upgrades for the viewed skill (not the active one)
    let upgrades = if has_notification_settings || !is_viewed_skill_purchased {
        Vec::new()
    } else {
        tower.get_skill_upgrades(viewed_skill_idx)
    };

    let mut stat_elements: Vec<AnyElement> = Vec::new();

    // Show viewed skill name and description
    stat_elements.push(
        div()
            .text_xs()
            .text_color(if is_viewed_skill_active {
                rgb(0x44aaff)
            } else if is_viewed_skill_locked {
                rgb(0x666666)
            } else {
                rgb(0xaaaaaa)
            })
            .child(if is_viewed_skill_active {
                format!("{} (active)", viewed_skill_name)
            } else if is_viewed_skill_locked {
                format!("{} (verrouille)", viewed_skill_name)
            } else {
                viewed_skill_name.to_string()
            })
            .into_any_element(),
    );
    if !viewed_skill_desc.is_empty() {
        stat_elements.push(
            div()
                .text_xs()
                .text_color(rgb(0x888888))
                .child(viewed_skill_desc)
                .into_any_element(),
        );
    }

    // Add "Buy" button if skill is locked
    if is_viewed_skill_locked {
        stat_elements.push(
            Button::new("buy_skill_btn")
                .primary()
                .label(format!("Acheter ({}g)", viewed_skill_cost))
                .compact()
                .with_size(Size::Small)
                .disabled(!can_afford_skill)
                .on_click(cx.listener(move |screen, _, _window, _cx| {
                    if let Some(idx) = screen.game_state.selected_tower {
                        screen.game_state.purchase_skill(idx, viewed_skill_idx);
                    }
                }))
                .into_any_element(),
        );
    }
    // Add "Activate" button if this skill is purchased but not active
    else if is_viewed_skill_purchased && !is_viewed_skill_active {
        stat_elements.push(
            Button::new("activate_skill_btn")
                .primary()
                .label("Activer")
                .compact()
                .with_size(Size::Small)
                .on_click(cx.listener(move |screen, _, _window, _cx| {
                    if let Some(idx) = screen.game_state.selected_tower {
                        screen.game_state.activate_skill(idx, viewed_skill_idx);
                    }
                }))
                .into_any_element(),
        );
    }

    for (upgrade_id, uname, prop) in &upgrades {
        let current_value = prop.value();
        let is_maxed = !prop.can_upgrade();
        let uid = *upgrade_id;
        let skill_idx_for_upgrade = viewed_skill_idx;

        // Format value
        let value_str = if current_value.fract() == 0.0 {
            format!("{:.0}", current_value)
        } else {
            format!("{:.2}", current_value)
        };

        let mut row = h_flex()
            .id(SharedString::from(format!("stat_row_{:?}", upgrade_id)))
            .items_center()
            .justify_between()
            .child(
                div()
                    .text_xs()
                    .text_color(rgb(0xcccccc))
                    .child(format!("{}: {}", uname, value_str)),
            );

        if !is_maxed {
            let cost = prop.cost();
            let bonus = prop.bonus_per_level;
            let can_afford = gold >= cost;
            let bonus_str = if bonus.fract() == 0.0 {
                format!("+{:.0}", bonus)
            } else {
                format!("+{:.1}", bonus)
            };

            row = row.child(
                h_flex()
                    .items_center()
                    .gap_1()
                    .child(div().text_xs().text_color(rgb(0x888888)).child(bonus_str))
                    .child(
                        Button::new(SharedString::from(format!("btn_upgrade_{:?}", uid)))
                            .label(format!("{}g", cost))
                            .compact()
                            .with_size(Size::XSmall)
                            .disabled(!can_afford)
                            .on_click(cx.listener(move |screen, _, _window, _cx| {
                                if let Some(idx) = screen.game_state.selected_tower {
                                    screen.game_state.upgrade_tower_skill(
                                        idx,
                                        skill_idx_for_upgrade,
                                        uid,
                                    );
                                }
                            })),
                    ),
            );
        }

        stat_elements.push(row.into_any_element());
    }

    // Target priority button (only if active skill allows it)
    let target_priority = tower.target_priority;
    let priority_btn = if tower.can_change_target() {
        Some(
            Button::new("sidebar_target_priority")
                .label(format!("Cible: {}", target_priority.display_name()))
                .compact()
                .with_size(Size::Small)
                .on_click(cx.listener(move |screen, _, _window, _cx| {
                    if let Some(idx) = screen.game_state.selected_tower {
                        if let Some(tower) = screen.game_state.towers.get_mut(idx) {
                            tower.cycle_target_priority();
                        }
                    }
                })),
        )
    } else {
        None
    };

    let move_cost = game.move_cost(tower_idx);
    let can_move = gold >= move_cost;
    let move_btn = Button::new("sidebar_move_tower")
        .label(format!("Deplacer ({}g)", move_cost))
        .compact()
        .with_size(Size::Small)
        .disabled(!can_move)
        .on_click(cx.listener(move |screen, _, _window, _cx| {
            if let Some(idx) = screen.game_state.selected_tower {
                screen.game_state.move_mode = Some(idx);
            }
        }));

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

    // Notification settings section (for Alarme tower)
    let notification_section = tower.notification_settings.as_ref().map(|settings| {
        let shield_broken = settings.shield_broken;
        let shield_low = settings.shield_low;

        v_flex()
            .gap_2()
            .child(Divider::horizontal().color(Hsla {
                h: 0.0,
                s: 0.0,
                l: 0.25,
                a: 1.0,
            }))
            .child(
                div()
                    .text_xs()
                    .text_color(rgb(0xaaaaaa))
                    .child("Notifications"),
            )
            .child(
                h_flex()
                    .items_center()
                    .gap_2()
                    .child(
                        Checkbox::new("notif_shield_broken")
                            .checked(shield_broken)
                            .on_click(cx.listener(move |screen, _, _window, _cx| {
                                if let Some(idx) = screen.game_state.selected_tower {
                                    if let Some(tower) = screen.game_state.towers.get_mut(idx) {
                                        if let Some(settings) = &mut tower.notification_settings {
                                            settings.shield_broken = !settings.shield_broken;
                                        }
                                    }
                                }
                            })),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(0xffffff))
                            .child("Bouclier brise"),
                    ),
            )
            .child(
                h_flex()
                    .items_center()
                    .gap_2()
                    .child(
                        Checkbox::new("notif_shield_low")
                            .checked(shield_low)
                            .on_click(cx.listener(move |screen, _, _window, _cx| {
                                if let Some(idx) = screen.game_state.selected_tower {
                                    if let Some(tower) = screen.game_state.towers.get_mut(idx) {
                                        if let Some(settings) = &mut tower.notification_settings {
                                            settings.shield_low = !settings.shield_low;
                                        }
                                    }
                                }
                            })),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(0xffffff))
                            .child("Bouclier <= 25%"),
                    ),
            )
    });

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
            .child(div().text_sm().text_color(color).child(name))
            // Skill icons row
            .child(
                v_flex()
                    .gap_1()
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(0xaaaaaa))
                            .child("Competences"),
                    )
                    .child(h_flex().gap_1().children(skill_icons)),
            )
            // Stats with inline upgrades (only for active skill)
            .children(stat_elements)
            // Target priority button (if tower attacks)
            .when_some(priority_btn, |this, btn| this.child(btn))
            // Notification settings (if available)
            .when_some(notification_section, |this, section| this.child(section))
            // Move
            .child(move_btn)
            // Sell
            .child(sell_btn),
    )
}

fn tower_icon(
    kind: TowerKind,
    gold: u32,
    slots_full: bool,
    cx: &mut Context<PlayScreen>,
) -> impl IntoElement + use<> {
    let def = get_def(kind);
    let cost = def.base_cost;
    let can_afford = gold >= cost && !slots_full;
    let color = def.element.color();
    let name = def.name;

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

    Button::new(SharedString::from(format!("tower_icon_{:?}", kind)))
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
                screen.game_state.placement_mode = Some(kind);
            }
        }))
}
