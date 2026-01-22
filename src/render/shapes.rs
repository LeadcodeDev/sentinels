use gpui::*;

use crate::game::enemy::Enemy;
use crate::game::player::Player;
use crate::game::tower::Tower;
use crate::game::{Point2D, Projectile, ProjectileSource};

fn to_screen(center: Point<Pixels>, game_pos: &Point2D) -> Point<Pixels> {
    point(center.x + px(game_pos.x), center.y + px(game_pos.y))
}

pub fn draw_polygon(
    window: &mut Window,
    screen_pos: Point<Pixels>,
    radius: f32,
    sides: u32,
    color: Hsla,
    rotation: f32,
) {
    if sides < 3 {
        return;
    }

    let points: Vec<Point<Pixels>> = (0..sides)
        .map(|i| {
            let angle = rotation + (2.0 * std::f32::consts::PI * i as f32) / sides as f32;
            point(
                screen_pos.x + px(radius * angle.cos()),
                screen_pos.y + px(radius * angle.sin()),
            )
        })
        .collect();

    let mut path = Path::new(points[0]);
    for p in &points[1..] {
        path.line_to(*p);
    }
    path.line_to(points[0]);

    window.paint_path(path, color);
}

pub fn draw_circle(window: &mut Window, screen_pos: Point<Pixels>, radius: f32, color: Hsla) {
    draw_polygon(window, screen_pos, radius, 32, color, 0.0);
}

pub fn draw_circle_outline(
    window: &mut Window,
    center: Point<Pixels>,
    game_pos: &Point2D,
    radius: f32,
    color: Hsla,
) {
    let screen_pos = to_screen(center, game_pos);
    let segments = 32u32;

    for i in 0..segments {
        let angle1 = (2.0 * std::f32::consts::PI * i as f32) / segments as f32;
        let angle2 = (2.0 * std::f32::consts::PI * (i + 1) as f32) / segments as f32;

        let p1 = point(
            screen_pos.x + px(radius * angle1.cos()),
            screen_pos.y + px(radius * angle1.sin()),
        );
        let p2 = point(
            screen_pos.x + px(radius * angle2.cos()),
            screen_pos.y + px(radius * angle2.sin()),
        );

        // Draw thin quad as line segment
        let dx = f32::from(p2.x) - f32::from(p1.x);
        let dy = f32::from(p2.y) - f32::from(p1.y);
        let len = (dx * dx + dy * dy).sqrt();
        if len < 0.1 {
            continue;
        }
        let nx = -dy / len;
        let ny = dx / len;
        let half_width = 0.5;

        let points = vec![
            point(p1.x + px(nx * half_width), p1.y + px(ny * half_width)),
            point(p2.x + px(nx * half_width), p2.y + px(ny * half_width)),
            point(p2.x - px(nx * half_width), p2.y - px(ny * half_width)),
            point(p1.x - px(nx * half_width), p1.y - px(ny * half_width)),
        ];

        let mut path = Path::new(points[0]);
        for p in &points[1..] {
            path.line_to(*p);
        }
        path.line_to(points[0]);
        window.paint_path(path, color);
    }
}

pub fn draw_dashed_circle_outline(
    window: &mut Window,
    center: Point<Pixels>,
    game_pos: &Point2D,
    radius: f32,
    color: Hsla,
    dash_count: u32,
) {
    let screen_pos = to_screen(center, game_pos);
    let segments = dash_count * 2; // alternating dash/gap

    for i in 0..segments {
        if i % 2 != 0 {
            continue; // skip gap segments
        }

        let angle1 = (2.0 * std::f32::consts::PI * i as f32) / segments as f32;
        let angle2 = (2.0 * std::f32::consts::PI * (i + 1) as f32) / segments as f32;

        let p1 = point(
            screen_pos.x + px(radius * angle1.cos()),
            screen_pos.y + px(radius * angle1.sin()),
        );
        let p2 = point(
            screen_pos.x + px(radius * angle2.cos()),
            screen_pos.y + px(radius * angle2.sin()),
        );

        let dx = f32::from(p2.x) - f32::from(p1.x);
        let dy = f32::from(p2.y) - f32::from(p1.y);
        let len = (dx * dx + dy * dy).sqrt();
        if len < 0.1 {
            continue;
        }
        let nx = -dy / len;
        let ny = dx / len;
        let half_width = 1.0;

        let points = vec![
            point(p1.x + px(nx * half_width), p1.y + px(ny * half_width)),
            point(p2.x + px(nx * half_width), p2.y + px(ny * half_width)),
            point(p2.x - px(nx * half_width), p2.y - px(ny * half_width)),
            point(p1.x - px(nx * half_width), p1.y - px(ny * half_width)),
        ];

        let mut path = Path::new(points[0]);
        for p in &points[1..] {
            path.line_to(*p);
        }
        path.line_to(points[0]);
        window.paint_path(path, color);
    }
}

pub fn draw_player(window: &mut Window, center: Point<Pixels>, player: &Player) {
    let color = player.element.color();

    // Outer glow
    draw_circle(
        window,
        center,
        player.radius + 4.0,
        Hsla {
            h: color.h,
            s: color.s,
            l: color.l,
            a: 0.2,
        },
    );

    // Main body
    draw_circle(window, center, player.radius, color);

    // Inner highlight
    draw_circle(
        window,
        center,
        player.radius * 0.4,
        Hsla {
            h: 0.0,
            s: 0.0,
            l: 1.0,
            a: 0.5,
        },
    );
}

pub fn draw_enemy(window: &mut Window, center: Point<Pixels>, enemy: &Enemy) {
    let screen_pos = to_screen(center, &enemy.position);
    let sides = enemy.shape.sides();

    let color = if enemy.is_boss {
        Hsla {
            h: 0.0,
            s: 0.8,
            l: 0.3,
            a: 1.0,
        }
    } else {
        Hsla {
            h: 0.0,
            s: 0.7,
            l: 0.5,
            a: 1.0,
        }
    };

    let rotation = (enemy.id as f32) * 0.5;
    draw_polygon(window, screen_pos, enemy.radius, sides, color, rotation);

    // HP bar
    draw_hp_bar(window, screen_pos, enemy.hp, enemy.max_hp, enemy.radius);
}

pub fn draw_tower(window: &mut Window, center: Point<Pixels>, tower: &Tower) {
    let screen_pos = to_screen(center, &tower.position);
    let color = tower.element.color();

    // Diamond shape (rotated square)
    draw_polygon(
        window,
        screen_pos,
        tower.radius,
        4,
        color,
        std::f32::consts::PI / 4.0,
    );

    // Level dots below
    let dot_count = tower.level.min(5);
    for i in 0..dot_count {
        let offset_x = (i as f32 - (dot_count as f32 - 1.0) / 2.0) * 6.0;
        draw_circle(
            window,
            point(
                screen_pos.x + px(offset_x),
                screen_pos.y + px(tower.radius + 8.0),
            ),
            2.0,
            Hsla {
                h: 0.0,
                s: 0.0,
                l: 1.0,
                a: 0.8,
            },
        );
    }
}

pub fn draw_projectile(window: &mut Window, center: Point<Pixels>, proj: &Projectile) {
    let screen_pos = to_screen(center, &proj.current_pos);
    let origin_screen = to_screen(center, &proj.origin);

    // Color: element color for player/tower projectiles, white for enemy projectiles
    let color = match proj.source {
        ProjectileSource::Player | ProjectileSource::Tower(_) => proj.element.color(),
        ProjectileSource::Enemy(_) => Hsla {
            h: 0.0,
            s: 0.0,
            l: 0.9,
            a: 1.0,
        },
    };

    // Trail line from origin to current position
    let trail_color = Hsla {
        h: color.h,
        s: color.s,
        l: color.l,
        a: 0.3,
    };
    draw_line(window, origin_screen, screen_pos, trail_color, 1.0);

    // Projectile dot
    let proj_radius = match proj.source {
        ProjectileSource::Enemy(_) => 3.0,
        _ => 4.0,
    };
    draw_circle(window, screen_pos, proj_radius, color);
}

fn draw_line(window: &mut Window, from: Point<Pixels>, to: Point<Pixels>, color: Hsla, width: f32) {
    let dx = f32::from(to.x) - f32::from(from.x);
    let dy = f32::from(to.y) - f32::from(from.y);
    let len = (dx * dx + dy * dy).sqrt();
    if len < 1.0 {
        return;
    }

    let nx = -dy / len;
    let ny = dx / len;
    let half_w = width / 2.0;

    let points = vec![
        point(from.x + px(nx * half_w), from.y + px(ny * half_w)),
        point(to.x + px(nx * half_w), to.y + px(ny * half_w)),
        point(to.x - px(nx * half_w), to.y - px(ny * half_w)),
        point(from.x - px(nx * half_w), from.y - px(ny * half_w)),
    ];

    let mut path = Path::new(points[0]);
    for p in &points[1..] {
        path.line_to(*p);
    }
    path.line_to(points[0]);
    window.paint_path(path, color);
}

fn draw_hp_bar(window: &mut Window, pos: Point<Pixels>, hp: f32, max_hp: f32, radius: f32) {
    let bar_width = radius * 2.0;
    let bar_height: f32 = 3.0;
    let bar_y = pos.y - px(radius + 10.0);
    let bar_x = pos.x - px(radius);

    // Background
    window.paint_quad(PaintQuad {
        bounds: Bounds {
            origin: point(bar_x, bar_y),
            size: size(px(bar_width), px(bar_height)),
        },
        corner_radii: Corners::all(px(1.0)),
        background: Hsla {
            h: 0.0,
            s: 0.0,
            l: 0.2,
            a: 0.8,
        }
        .into(),
        border_widths: Edges::all(px(0.0)),
        border_color: Hsla::transparent_black(),
        border_style: BorderStyle::default(),
    });

    // Foreground
    let ratio = (hp / max_hp).clamp(0.0, 1.0);
    let hp_color = Hsla {
        h: ratio * 0.33,
        s: 0.8,
        l: 0.5,
        a: 1.0,
    };

    window.paint_quad(PaintQuad {
        bounds: Bounds {
            origin: point(bar_x, bar_y),
            size: size(px(bar_width * ratio), px(bar_height)),
        },
        corner_radii: Corners::all(px(1.0)),
        background: hp_color.into(),
        border_widths: Edges::all(px(0.0)),
        border_color: Hsla::transparent_black(),
        border_style: BorderStyle::default(),
    });
}
