mod game;

use game::{Game, Outcome};
use macroquad::prelude::*;
use std::hash::{BuildHasher, Hasher};

const BG: Color = Color::new(0.035, 0.045, 0.085, 1.);
const PANEL: Color = Color::new(0.085, 0.105, 0.17, 1.);
const MUTED: Color = Color::new(0.57, 0.63, 0.76, 1.);
const COLORS: [Color; 4] = [
    Color::new(1., 0.29, 0.39, 1.),
    Color::new(0.23, 0.60, 1., 1.),
    Color::new(0.23, 0.84, 0.58, 1.),
    Color::new(1., 0.78, 0.25, 1.),
];
const BOARD: Rect = Rect {
    x: 12.,
    y: 375.,
    w: 416.,
    h: 301.,
};

fn conf() -> Conf {
    Conf {
        window_title: "Finger Twister".into(),
        window_width: 440,
        window_height: 820,
        high_dpi: true,
        ..Default::default()
    }
}

fn seed() -> u64 {
    std::collections::hash_map::RandomState::new()
        .build_hasher()
        .finish()
}

fn text(s: &str, x: f32, y: f32, size: f32, color: Color) {
    draw_text(s, x, y, size, color);
}

fn centered(s: &str, y: f32, size: f32, color: Color) {
    let width = measure_text(s, None, size as u16, 1.).width;
    text(s, (440. - width) / 2., y, size, color);
}

fn button(r: Rect, title: &str, selected: bool, click: Option<Vec2>) -> bool {
    draw_rectangle(r.x, r.y, r.w, r.h, if selected { COLORS[1] } else { PANEL });
    let size = 21.;
    let width = measure_text(title, None, size as u16, 1.).width;
    text(
        title,
        r.x + (r.w - width) / 2.,
        r.y + r.h / 2. + 7.,
        size,
        WHITE,
    );
    click.is_some_and(|p| r.contains(p))
}

// Drawing and touch detection use the same circle geometry.
fn circle_geometry(cell: usize, columns: usize, rows: usize, bounds: Rect) -> (Vec2, f32) {
    let w = bounds.w / columns as f32;
    let h = bounds.h / rows as f32;
    (
        vec2(
            bounds.x + (cell % columns) as f32 * w + w / 2.,
            bounds.y + (cell / columns) as f32 * h + h / 2.,
        ),
        w.min(h) * 0.46,
    )
}

fn cell_at(p: Vec2, columns: usize, rows: usize, bounds: Rect) -> Option<usize> {
    if p.x < bounds.x || p.y < bounds.y || p.x >= bounds.right() || p.y >= bounds.bottom() {
        return None;
    }
    let column = ((p.x - bounds.x) / (bounds.w / columns as f32)) as usize;
    let row = ((p.y - bounds.y) / (bounds.h / rows as f32)) as usize;
    let cell = row * columns + column;
    let (center, radius) = circle_geometry(cell, columns, rows, bounds);
    (p.distance_squared(center) <= radius * radius).then_some(cell)
}

fn board(g: &Game, bounds: Rect, hint: Option<f32>, feedback: Option<(usize, f64)>) {
    let feedback = feedback.and_then(|(cell, time)| {
        let progress = ((get_time() - time) / 0.65) as f32;
        (0. ..1.).contains(&progress).then_some((cell, progress))
    });
    for (i, &color) in g.cells.iter().enumerate() {
        let (center, radius) = circle_geometry(i, g.columns, g.rows, bounds);
        let held = g.occupied(i);
        let target = hint.is_some() && color == g.target && !held;
        let c = COLORS[color];
        let brightness = if hint.is_some() && !target && !held {
            0.5
        } else {
            1.
        };
        draw_circle(
            center.x,
            center.y,
            radius,
            Color::new(c.r * brightness, c.g * brightness, c.b * brightness, 1.),
        );
        let fs = (radius * 0.48).clamp(6., 32.);
        let mark = ["I", "II", "III", "IV"][color];
        let mw = measure_text(mark, None, fs as u16, 1.).width;
        text(mark, center.x - mw / 2., center.y + fs * 0.35, fs, BG);
        if target {
            let pulse = 0.65 + 0.35 * (get_time() as f32 * 5.).sin().abs();
            draw_circle_lines(
                center.x,
                center.y,
                radius - 3.,
                3.,
                Color::new(1., 1., 1., pulse),
            );
            let bar_width = radius * 1.1;
            draw_rectangle(
                center.x - bar_width / 2.,
                center.y + radius * 0.55,
                bar_width * hint.unwrap(),
                4.,
                WHITE,
            );
        }
        if held {
            let finger = g.fingers.iter().find(|f| f.cell == i).unwrap();
            if finger.removable {
                // A white disk and dark lift symbol remain distinct from target rings.
                let pulse = 0.5 + 0.5 * (get_time() as f32 * 4.).sin();
                draw_circle(center.x, center.y, radius * 0.93, WHITE);
                draw_circle_lines(center.x, center.y, radius - 2., 3., BG);
                draw_circle_lines(center.x, center.y, radius * (0.78 + pulse * 0.08), 2., BG);
                let label_size = radius * 0.48;
                let width = measure_text("PUST", None, label_size as u16, 1.).width;
                text(
                    "PUST",
                    center.x - width / 2.,
                    center.y + radius * 0.5,
                    label_size,
                    BG,
                );
                let arrow_y = center.y - radius * (0.28 + pulse * 0.08);
                let stroke = (radius * 0.07).max(1.5);
                draw_line(
                    center.x,
                    center.y + radius * 0.05,
                    center.x,
                    arrow_y,
                    stroke,
                    BG,
                );
                draw_line(
                    center.x,
                    arrow_y,
                    center.x - radius * 0.19,
                    arrow_y + radius * 0.19,
                    stroke,
                    BG,
                );
                draw_line(
                    center.x,
                    arrow_y,
                    center.x + radius * 0.19,
                    arrow_y + radius * 0.19,
                    stroke,
                    BG,
                );
            } else {
                draw_circle(center.x, center.y - radius * 0.52, fs * 0.7, BG);
                let label = finger.order.to_string();
                let mw = measure_text(&label, None, fs as u16, 1.).width;
                text(
                    &label,
                    center.x - mw / 2.,
                    center.y - radius * 0.52 + fs * 0.35,
                    fs,
                    WHITE,
                );
            }
        }
        if let Some((accepted, progress)) = feedback {
            let flash = Color::new(1., 1., 1., 1. - progress);
            if accepted == i {
                draw_circle_lines(
                    center.x,
                    center.y,
                    radius * (0.55 + progress * 0.43),
                    (radius * 0.1).max(2.),
                    flash,
                );
                draw_circle_lines(center.x, center.y, radius - 2., 4., flash);
            }
            if target {
                // All next targets visibly acknowledge a new round, even if its color repeats.
                draw_circle_lines(
                    center.x,
                    center.y,
                    radius * (0.72 + progress * 0.25),
                    4.,
                    flash,
                );
            }
        }
    }
    if let Some((_, progress)) = feedback {
        let thickness = bounds.w.min(bounds.h) * 0.012;
        draw_rectangle_lines(
            bounds.x + thickness / 2.,
            bounds.y + thickness / 2.,
            bounds.w - thickness,
            bounds.h - thickness,
            thickness,
            Color::new(0.3, 1., 0.65, (1. - progress) * 0.9),
        );
    }
}

#[macroquad::main(conf)]
async fn main() {
    simulate_mouse_with_touch(false);
    let mut columns = 4;
    let mut rows = 4;
    let mut seconds = 5.;
    let mut release_after = 5;
    let mut preview = Game::new(columns, rows, seconds, seed(), 0.);
    let mut game: Option<Game> = None;
    let mut armed = false;
    let mut practice = false;
    let mut virtual_contacts: Vec<(u64, Option<usize>)> = Vec::new();
    let mut next_id = 1u64;
    let mut feedback: Option<(usize, f64)> = None;
    loop {
        let playing = game.as_ref().is_some_and(|g| g.outcome == Outcome::Playing);
        let scale = if playing {
            1.
        } else {
            (screen_width() / 440.).min(screen_height() / 820.)
        };
        let origin = if playing {
            Vec2::ZERO
        } else {
            vec2(
                (screen_width() - 440. * scale) / 2.,
                (screen_height() - 820. * scale) / 2.,
            )
        };
        let local = |p: Vec2| (p - origin) / scale;
        set_camera(&Camera2D {
            zoom: vec2(2. * scale / screen_width(), 2. * scale / screen_height()),
            target: if playing {
                vec2(screen_width() / 2., screen_height() / 2.)
            } else {
                vec2(220., 410.)
            },
            ..Default::default()
        });
        clear_background(BG);
        let touches = touches();
        let active: Vec<_> = touches
            .iter()
            .filter(|t| !matches!(t.phase, TouchPhase::Ended | TouchPhase::Cancelled))
            .collect();
        let touch_click = touches
            .iter()
            .find(|t| t.phase == TouchPhase::Started)
            .map(|t| local(t.position / macroquad::miniquad::window::dpi_scale()));
        let mouse_click =
            is_mouse_button_pressed(MouseButton::Left).then(|| local(Vec2::from(mouse_position())));
        let click = touch_click.or(mouse_click);
        let released = active.is_empty() && !is_mouse_button_down(MouseButton::Left);

        if game.is_none() {
            centered("FINGER / TWISTER", 43., 32., WHITE);
            centered("Kolko prstov udrzis v hre?", 74., 19., MUTED);
            for (label, y, value) in [
                ("POCET STLPCOV / SIRKA", 104., &mut columns),
                ("POCET RIADKOV / VYSKA", 173., &mut rows),
            ] {
                text(label, 18., y, 17., MUTED);
                if button(Rect::new(16., y + 10., 80., 36.), "-", false, click) {
                    *value = value.saturating_sub(1).max(3);
                }
                button(
                    Rect::new(104., y + 10., 232., 36.),
                    &value.to_string(),
                    true,
                    None,
                );
                if button(Rect::new(344., y + 10., 80., 36.), "+", false, click) {
                    *value = (*value + 1).min(12);
                }
            }

            if preview.columns != columns || preview.rows != rows {
                preview = Game::new(columns, rows, seconds, seed(), 0.);
            }
            text("CAS NA NOVY PRST", 18., 242., 17., MUTED);
            for (index, value) in [3., 5., 8.].into_iter().enumerate() {
                if button(
                    Rect::new(16. + index as f32 * 138., 252., 130., 36.),
                    &format!("{value} sekund"),
                    seconds == value,
                    click,
                ) {
                    seconds = value;
                }
            }
            text("UVOLNOVANIE PRSTOV", 18., 314., 17., MUTED);
            for (index, limit) in [5, 10].into_iter().enumerate() {
                if button(
                    Rect::new(16. + index as f32 * 208., 326., 200., 38.),
                    &format!("Po {limit} prstoch"),
                    release_after == limit,
                    click,
                ) {
                    release_after = limit;
                }
            }
            board(&preview, BOARD, None, None);
            centered(
                "Pridaj prst na farbu. PUST = mozes zdvihnut.",
                703.,
                18.,
                MUTED,
            );
            if !cfg!(target_os = "android")
                && button(
                    Rect::new(16., 718., 408., 34.),
                    if practice {
                        "MYS: TRENING ZAPNUTY"
                    } else {
                        "MYS: ZAPNUT TRENING"
                    },
                    practice,
                    click,
                )
            {
                practice = !practice;
            }
            let fits = columns * rows >= release_after;
            if button(
                Rect::new(16., 764., 408., 44.),
                if fits {
                    "HRAT"
                } else {
                    "ZVACSI MRIEZKU PRE 10 PRSTOV"
                },
                fits,
                click,
            ) && fits
            {
                let mut next = Game::new(columns, rows, seconds, seed(), get_time());
                next.release_after = release_after;
                game = Some(next);
                armed = false;
                virtual_contacts.clear();
                feedback = None;
            }
        } else if let Some(g) = game.as_mut() {
            let bounds = Rect::new(0., 0., screen_width(), screen_height());
            if !armed && released {
                armed = true;
                g.deadline = get_time() + g.seconds;
            }
            if armed && g.outcome == Outcome::Playing {
                let mut contacts: Vec<_> = active
                    .iter()
                    .map(|t| {
                        (
                            t.id,
                            cell_at(
                                local(t.position / macroquad::miniquad::window::dpi_scale()),
                                g.columns,
                                g.rows,
                                bounds,
                            ),
                        )
                    })
                    .collect();
                if practice {
                    if let Some(p) = mouse_click {
                        let cell = cell_at(p, g.columns, g.rows, bounds);
                        if let Some(index) = virtual_contacts.iter().position(|(_, c)| *c == cell) {
                            virtual_contacts.remove(index);
                        } else {
                            virtual_contacts.push((next_id, cell));
                            next_id += 1;
                        }
                    }
                    contacts.extend(virtual_contacts.iter().copied());
                } else if active.is_empty() && is_mouse_button_down(MouseButton::Left) {
                    contacts.push((
                        u64::MAX,
                        cell_at(
                            local(Vec2::from(mouse_position())),
                            g.columns,
                            g.rows,
                            bounds,
                        ),
                    ));
                }
                let before = g.placed;
                g.update(&contacts, get_time());
                if g.placed > before {
                    if let Some(finger) = g.fingers.iter().find(|f| f.order == g.placed) {
                        feedback = Some((finger.cell, get_time()));
                    }
                }
            }
            if playing {
                let remaining = if armed {
                    (g.deadline - get_time()).max(0.)
                } else {
                    seconds
                };
                board(
                    g,
                    bounds,
                    Some((remaining / seconds).clamp(0., 1.) as f32),
                    feedback,
                );
            } else {
                centered("FINGER / TWISTER", 43., 32., WHITE);
                centered(
                    if g.outcome == Outcome::Won {
                        "VYBORNE!"
                    } else {
                        "KONIEC HRY"
                    },
                    159.,
                    34.,
                    WHITE,
                );
                if let Outcome::Lost(reason) = g.outcome {
                    centered(reason, 197., 18., COLORS[0]);
                }
                centered(&format!("Polozene prsty: {}", g.placed), 234., 23., MUTED);
                board(g, BOARD, None, None);
            }
            let terminal = g.outcome != Outcome::Playing;
            // Restart only after the losing contacts have been released.
            if terminal && released {
                armed = false;
            }
            let back = !playing
                && button(
                    Rect::new(16., 764., 408., 44.),
                    "NOVA HRA / NASTAVENIA",
                    true,
                    if armed { None } else { click },
                );
            if back {
                game = None;
            }
        }
        next_frame().await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn circular_hit_testing_matches_rectangular_grids() {
        for bounds in [
            BOARD,
            Rect::new(0., 0., 1080., 2400.),
            Rect::new(0., 0., 2400., 1080.),
        ] {
            for columns in 3..=6 {
                for rows in 3..=6 {
                    for cell in 0..columns * rows {
                        let (center, radius) = circle_geometry(cell, columns, rows, bounds);
                        assert_eq!(cell_at(center, columns, rows, bounds), Some(cell));
                        assert_eq!(
                            cell_at(center + vec2(radius * 0.99, 0.), columns, rows, bounds),
                            Some(cell)
                        );
                        assert_eq!(
                            cell_at(center + vec2(radius * 1.01, 0.), columns, rows, bounds),
                            None
                        );
                    }
                    assert_eq!(
                        cell_at(vec2(bounds.x, bounds.y), columns, rows, bounds),
                        None
                    );
                    assert_eq!(
                        cell_at(vec2(bounds.right(), bounds.bottom()), columns, rows, bounds),
                        None
                    );
                }
            }
        }
    }
}
