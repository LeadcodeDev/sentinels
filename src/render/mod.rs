pub mod shapes;

use gpui::*;

use crate::game::GameState;
use crate::game::Point2D;
use crate::game::elemental::TowerElement;
use shapes::*;

pub struct PlacementPreview {
    pub element: TowerElement,
    pub game_pos: Point2D,
    pub range: f32,
}

pub fn render_game(
    game: &GameState,
    _viewport_size: Size<Pixels>,
    placement_preview: Option<PlacementPreview>,
) -> impl IntoElement {
    let player = game.player.clone();
    let shield = game.shield.clone();
    let towers = game.towers.clone();
    let enemies = game.enemies.clone();
    let projectiles = game.projectiles.clone();
    let aoe_splashes = game.aoe_splashes.clone();
    let elapsed = game.elapsed;
    let selected_tower = game.selected_tower;

    canvas(
        move |_bounds, _window, _cx| {},
        move |bounds, _, window, _cx| {
            let center = point(
                bounds.origin.x + bounds.size.width / 2.0,
                bounds.origin.y + bounds.size.height / 2.0,
            );

            // Draw tower range circles
            for (i, tower) in towers.iter().enumerate() {
                let alpha = if selected_tower == Some(i) { 0.3 } else { 0.1 };
                draw_circle_outline(
                    window,
                    center,
                    &tower.position,
                    tower.attack_range,
                    Hsla {
                        h: 0.0,
                        s: 0.0,
                        l: 0.5,
                        a: alpha,
                    },
                );
            }

            // Draw player range circle
            draw_circle_outline(
                window,
                center,
                &player.position,
                player.attack_range,
                Hsla {
                    h: 0.0,
                    s: 0.0,
                    l: 1.0,
                    a: 0.15,
                },
            );

            // Draw towers
            for tower in &towers {
                draw_tower(window, center, tower);
            }

            // Draw enemies
            for enemy in &enemies {
                draw_enemy(window, center, enemy, elapsed);
            }

            // Draw projectiles
            for proj in &projectiles {
                draw_projectile(window, center, proj);
            }

            // Draw AoE splash effects
            for splash in &aoe_splashes {
                draw_aoe_splash(window, center, splash);
            }

            // Draw player
            draw_player(window, center, &player);

            // Draw shield
            if shield.is_unlocked() {
                draw_shield(window, center, &shield);
            }

            // Draw placement preview (ghost tower + dashed range circle)
            if let Some(ref preview) = placement_preview {
                let color = preview.element.color();
                let ghost_color = Hsla {
                    h: color.h,
                    s: color.s,
                    l: color.l,
                    a: 0.4,
                };
                let screen_pos = point(
                    center.x + px(preview.game_pos.x),
                    center.y + px(preview.game_pos.y),
                );
                // Ghost tower diamond
                draw_polygon(
                    window,
                    screen_pos,
                    15.0,
                    4,
                    ghost_color,
                    std::f32::consts::PI / 4.0,
                );
                // Dashed range circle
                draw_dashed_circle_outline(
                    window,
                    center,
                    &preview.game_pos,
                    preview.range,
                    Hsla {
                        h: color.h,
                        s: color.s,
                        l: color.l,
                        a: 0.5,
                    },
                    16,
                );
            }
        },
    )
    .size_full()
}
