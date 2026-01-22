use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::divider::Divider;
use gpui_component::{Disableable, Sizable, Size, h_flex, v_flex};

use crate::game::GameState;
use crate::screens::play::PlayScreen;
use crate::ui::hud::sidebar_width;

pub fn render_tower_popover(
    game: &GameState,
    cx: &mut Context<PlayScreen>,
) -> Option<impl IntoElement + use<>> {
    let tower_idx = game.selected_tower?;
    let tower = game.towers.get(tower_idx)?;

    let viewport = game.viewport_size;
    let sidebar_w = sidebar_width();
    let canvas_width = viewport.0 - sidebar_w;
    let center_x = canvas_width / 2.0;
    let center_y = viewport.1 / 2.0;

    let screen_x = center_x + tower.position.x;
    let screen_y = center_y + tower.position.y;

    let element_name = tower.element.name();
    let color = tower.element.color();
    let damage = tower.attack_damage;
    let range = tower.attack_range;
    let speed = tower.attack_speed;
    let level = tower.level;
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

    // Position popover to the right of the tower, offset by tower radius
    let popover_x = screen_x + 30.0;
    let popover_y = screen_y - 80.0;

    let sell_btn = Button::new("sell_tower")
        .danger()
        .label(format!("Vendre ({}g)", sell_value))
        .compact()
        .with_size(Size::Small)
        .on_click(cx.listener(move |screen, _, _window, _cx| {
            if let Some(idx) = screen.game_state.selected_tower {
                screen.game_state.sell_tower(idx);
            }
        }));

    let mut upgrade_elements: Vec<AnyElement> = Vec::new();
    for &(upgrade_type, level, max_level, cost, bonus, can_afford) in &upgrades {
        let label = if level >= max_level {
            format!("{} MAX", upgrade_type.name())
        } else {
            format!("{} +{:.0} ({}g)", upgrade_type.name(), bonus, cost)
        };
        let id = SharedString::from(format!("upgrade_{:?}", upgrade_type));
        let is_maxed = level >= max_level;

        let btn = h_flex()
            .items_center()
            .justify_between()
            .gap_2()
            .child(
                div()
                    .text_xs()
                    .text_color(rgb(0xaaaaaa))
                    .child(format!("{}/{}", level, max_level)),
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
        upgrade_elements.push(btn.into_any_element());
    }

    let popover = v_flex()
        .id("tower_popover")
        .absolute()
        .left(px(popover_x))
        .top(px(popover_y))
        .w(px(200.0))
        .on_mouse_down(MouseButton::Left, |_, _, _| {})
        .on_mouse_down(MouseButton::Right, |_, _, _| {})
        .p_3()
        .rounded_md()
        .bg(Hsla {
            h: 0.0,
            s: 0.0,
            l: 0.1,
            a: 0.95,
        })
        .border_1()
        .border_color(Hsla {
            h: color.h,
            s: color.s,
            l: color.l,
            a: 0.6,
        })
        .gap_2()
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
        // Separator
        .child(Divider::horizontal().color(Hsla {
            h: 0.0,
            s: 0.0,
            l: 0.3,
            a: 1.0,
        }))
        // Upgrades header
        .child(
            div()
                .text_xs()
                .text_color(rgb(0xaaaaaa))
                .child("Ameliorations"),
        )
        // Upgrade buttons
        .children(upgrade_elements)
        // Separator
        .child(Divider::horizontal().color(Hsla {
            h: 0.0,
            s: 0.0,
            l: 0.3,
            a: 1.0,
        }))
        // Sell button
        .child(sell_btn);

    Some(popover)
}
