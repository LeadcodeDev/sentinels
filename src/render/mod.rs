pub mod shapes;

use gpui::*;

use crate::game::GameState;
use shapes::*;

pub fn render_game(game: &GameState, viewport_size: Size<Pixels>) -> impl IntoElement {
    let player = game.player.clone();
    let towers = game.towers.clone();
    let enemies = game.enemies.clone();
    let projectiles = game.projectiles.clone();
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
                draw_enemy(window, center, enemy);
            }

            // Draw projectiles
            for proj in &projectiles {
                draw_projectile(window, center, proj);
            }

            // Draw player
            draw_player(window, center, &player);
        },
    )
    .size_full()
}
