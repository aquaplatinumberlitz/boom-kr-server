//! Màu, kiểu chữ, kiểu widget và hình vẽ của launcher (chỉ phần nhìn, không có logic).
use iced::{
    Background, Border, Color, Font, Gradient, Point, Radians, Rectangle, Shadow, Size, Theme,
    Vector,
    font::Weight,
    gradient::Linear,
    mouse,
    widget::{button, canvas, container, image, text_input},
};
use std::f32::consts::{FRAC_PI_2, PI};

pub const INK: Color = Color::from_rgb(0.03, 0.10, 0.26);
pub const DEEP: Color = Color::from_rgb(0.03, 0.17, 0.46);
pub const SKY: Color = Color::from_rgb(0.09, 0.56, 0.94);
pub const MUTED: Color = Color::from_rgb(0.80, 0.90, 1.0);
pub const ORANGE: Color = Color::from_rgb(1.0, 0.50, 0.12);
pub const PINK: Color = Color::from_rgb(1.0, 0.25, 0.52);
pub const SUN: Color = Color::from_rgb(1.0, 0.82, 0.24);
pub const MINT: Color = Color::from_rgb(0.36, 0.95, 0.75);
pub const VIOLET: Color = Color::from_rgb(0.62, 0.52, 1.0);
pub const CORAL: Color = Color::from_rgb(1.0, 0.47, 0.47);

pub const BOLD: Font = Font {
    weight: Weight::Bold,
    ..Font::DEFAULT
};
pub const HEAVY: Font = Font {
    weight: Weight::Black,
    ..Font::DEFAULT
};

const fn white(alpha: f32) -> Color {
    Color::from_rgba(1.0, 1.0, 1.0, alpha)
}
const fn alpha(c: Color, a: f32) -> Color {
    Color::from_rgba(c.r, c.g, c.b, a)
}
fn linear(angle: f32, from: Color, to: Color) -> Background {
    Background::Gradient(Gradient::Linear(
        Linear::new(Radians(angle))
            .add_stop(0.0, from)
            .add_stop(1.0, to),
    ))
}
fn glow(color: Color, y: f32, blur: f32) -> Shadow {
    Shadow {
        color,
        offset: Vector::new(0.0, y),
        blur_radius: blur,
    }
}

pub fn window(_: &Theme) -> container::Style {
    container::Style::default().background(linear(2.3, DEEP, SKY))
}

pub fn art_matte(_: &Theme) -> container::Style {
    container::Style::default().background(INK)
}

pub fn art_panel(_: &Theme) -> container::Style {
    container::Style {
        background: Some(linear(PI, alpha(INK, 0.87), alpha(INK, 0.95))),
        border: Border {
            color: white(0.36),
            width: 1.0,
            radius: 20.0.into(),
        },
        ..Default::default()
    }
}

pub fn decode_art(bytes: Option<Vec<u8>>) -> Option<image::Handle> {
    let mut reader =
        ::image::ImageReader::with_format(std::io::Cursor::new(bytes?), ::image::ImageFormat::Jpeg);
    let mut limits = ::image::Limits::default();
    limits.max_image_width = Some(1920);
    limits.max_image_height = Some(1920);
    limits.max_alloc = Some(24 * 1024 * 1024);
    reader.limits(limits);
    let rgba = reader.decode().ok()?.into_rgba8();
    Some(image::Handle::from_rgba(
        rgba.width(),
        rgba.height(),
        rgba.into_raw(),
    ))
}

pub fn pill(tint: Color) -> impl Fn(&Theme) -> container::Style {
    move |_| container::Style {
        background: Some(alpha(tint, 0.12).into()),
        border: Border {
            color: alpha(tint, 0.30),
            width: 1.0,
            radius: 999.0.into(),
        },
        ..Default::default()
    }
}

pub fn dot(color: Color) -> impl Fn(&Theme) -> container::Style {
    move |_| container::Style {
        background: Some(color.into()),
        border: Border::default().rounded(999),
        shadow: glow(alpha(color, 0.8), 0.0, 8.0),
        ..Default::default()
    }
}

pub fn brand_tag(_: &Theme) -> container::Style {
    container::Style {
        background: Some(linear(FRAC_PI_2, SUN, ORANGE)),
        border: Border::default().rounded(6),
        text_color: Some(Color::WHITE),
        ..Default::default()
    }
}

pub fn segmented(_: &Theme) -> container::Style {
    container::Style {
        background: Some(white(0.06).into()),
        border: Border {
            color: white(0.10),
            width: 1.0,
            radius: 999.0.into(),
        },
        ..Default::default()
    }
}

pub fn segment(selected: bool) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_, status| {
        let hovered = status == button::Status::Hovered;
        button::Style {
            background: if selected {
                Some(Color::WHITE.into())
            } else if hovered {
                Some(white(0.08).into())
            } else {
                None
            },
            text_color: if selected {
                INK
            } else if hovered {
                Color::WHITE
            } else {
                MUTED
            },
            border: Border::default().rounded(999),
            shadow: if selected {
                glow(Color::from_rgba(0.0, 0.0, 0.0, 0.35), 2.0, 10.0)
            } else {
                Shadow::default()
            },
            ..Default::default()
        }
    }
}

pub fn card(_: &Theme) -> container::Style {
    container::Style {
        background: Some(linear(PI, white(0.10), white(0.045))),
        border: Border {
            color: white(0.14),
            width: 1.0,
            radius: 26.0.into(),
        },
        shadow: glow(Color::from_rgba(0.0, 0.05, 0.18, 0.45), 22.0, 60.0),
        ..Default::default()
    }
}

pub fn field(_: &Theme, status: text_input::Status) -> text_input::Style {
    text_input::Style {
        background: Color::from_rgba(0.02, 0.08, 0.22, 0.45).into(),
        border: Border {
            color: match status {
                text_input::Status::Focused { .. } => ORANGE,
                text_input::Status::Hovered => white(0.24),
                _ => white(0.12),
            },
            width: if matches!(status, text_input::Status::Focused { .. }) {
                1.5
            } else {
                1.0
            },
            radius: 12.0.into(),
        },
        icon: MUTED,
        placeholder: alpha(MUTED, 0.55),
        value: Color::WHITE,
        selection: alpha(ORANGE, 0.35),
    }
}

/// Nút chính: cam khi vào chơi, đỏ khi dừng trò chơi, mờ khi chưa bấm được.
pub fn play(stop: bool) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_, status| {
        let (from, to) = if stop {
            (CORAL, Color::from_rgb(0.86, 0.22, 0.36))
        } else {
            (
                Color::from_rgb(1.0, 0.70, 0.10),
                Color::from_rgb(1.0, 0.40, 0.05),
            )
        };
        match status {
            button::Status::Disabled => button::Style {
                background: Some(white(0.07).into()),
                text_color: white(0.64),
                border: Border {
                    color: white(0.08),
                    width: 1.0,
                    radius: 14.0.into(),
                },
                ..Default::default()
            },
            _ => {
                let hovered = status == button::Status::Hovered;
                let pressed = status == button::Status::Pressed;
                button::Style {
                    background: Some(linear(FRAC_PI_2, if hovered { SUN } else { from }, to)),
                    text_color: INK,
                    border: Border::default().rounded(14),
                    shadow: if pressed {
                        Shadow::default()
                    } else {
                        glow(alpha(to, if hovered { 0.6 } else { 0.42 }), 10.0, 28.0)
                    },
                    ..Default::default()
                }
            }
        }
    }
}

fn circle(frame: &mut canvas::Frame, x: f32, y: f32, r: f32, color: Color) {
    frame.fill(&canvas::Path::circle(Point::new(x, y), r), color);
}

fn halo(frame: &mut canvas::Frame, x: f32, y: f32, r: f32, color: Color, strength: f32) {
    let steps = (r / 4.0).clamp(12.0, 48.0) as usize;
    for i in 0..steps {
        let t = 1.0 - i as f32 / steps as f32;
        circle(frame, x, y, r * t, alpha(color, strength / steps as f32));
    }
}

fn sparkle(frame: &mut canvas::Frame, x: f32, y: f32, s: f32, color: Color) {
    let c = Point::new(x, y);
    let path = canvas::Path::new(|p| {
        p.move_to(Point::new(x, y - s));
        p.quadratic_curve_to(c, Point::new(x + s, y));
        p.quadratic_curve_to(c, Point::new(x, y + s));
        p.quadratic_curve_to(c, Point::new(x - s, y));
        p.quadratic_curve_to(c, Point::new(x, y - s));
    });
    frame.fill(&path, color);
}

fn ellipse(frame: &mut canvas::Frame, x: f32, y: f32, rx: f32, ry: f32, color: Color) {
    let path = canvas::Path::new(|p| {
        p.ellipse(canvas::path::arc::Elliptical {
            center: Point::new(x, y),
            radii: Vector::new(rx, ry),
            rotation: Radians(0.0),
            start_angle: Radians(0.0),
            end_angle: Radians(2.0 * PI),
        })
    });
    frame.fill(&path, color);
}

fn bomb(frame: &mut canvas::Frame, x: f32, y: f32, r: f32, body: Color) {
    let stroke = |c: Color, w: f32| {
        canvas::Stroke::default()
            .with_color(c)
            .with_width(w)
            .with_line_cap(canvas::LineCap::Round)
    };
    let top = Point::new(x + r * 0.42, y - r * 0.9);
    let end = Point::new(x + r * 0.98, y - r * 1.32);
    let fuse = canvas::Path::new(|p| {
        p.move_to(top);
        p.bezier_curve_to(
            Point::new(x + r * 0.55, y - r * 1.35),
            Point::new(x + r * 0.85, y - r * 1.05),
            end,
        );
    });
    frame.stroke(&fuse, stroke(Color::from_rgb(0.85, 0.66, 0.42), r * 0.09));
    circle(frame, x, y, r, body);
    frame.stroke(
        &canvas::Path::circle(Point::new(x, y), r),
        stroke(white(0.22), r * 0.04),
    );
    let rim = canvas::Path::new(|p| {
        p.arc(canvas::path::Arc {
            center: Point::new(x, y),
            radius: r * 0.86,
            start_angle: Radians(-0.2),
            end_angle: Radians(1.5),
        })
    });
    frame.stroke(&rim, stroke(alpha(ORANGE, 0.55), r * 0.07));
    ellipse(
        frame,
        x - r * 0.38,
        y - r * 0.42,
        r * 0.26,
        r * 0.15,
        white(0.32),
    );
    circle(frame, x - r * 0.08, y - r * 0.62, r * 0.06, white(0.4));
    let cap = canvas::Path::rounded_rectangle(
        Point::new(top.x - r * 0.2, top.y - r * 0.1),
        Size::new(r * 0.4, r * 0.3),
        (r * 0.08).into(),
    );
    frame.fill(&cap, Color::from_rgb(0.30, 0.27, 0.44));
}

fn spark(frame: &mut canvas::Frame, x: f32, y: f32, r: f32, lit: bool) {
    let end = Point::new(x + r * 0.98, y - r * 1.32);
    let stroke = |c: Color, w: f32| {
        canvas::Stroke::default()
            .with_color(c)
            .with_width(w)
            .with_line_cap(canvas::LineCap::Round)
    };
    let spark = if lit { 1.0 } else { 0.78 };
    halo(frame, end.x, end.y, r * 0.75 * spark, ORANGE, 0.6);
    for i in 0..8 {
        let a = i as f32 * PI / 4.0 + if lit { 0.0 } else { PI / 8.0 };
        let len = r * if i % 2 == 0 { 0.42 } else { 0.26 } * spark;
        frame.stroke(
            &canvas::Path::line(
                Point::new(end.x + a.cos() * r * 0.1, end.y + a.sin() * r * 0.1),
                Point::new(end.x + a.cos() * len, end.y + a.sin() * len),
            ),
            stroke(SUN, r * 0.05),
        );
    }
    circle(frame, end.x, end.y, r * 0.11, Color::WHITE);
}

fn cloud(frame: &mut canvas::Frame, x: f32, y: f32, r: f32) {
    // Một path nhiều hình tròn: phủ một lần nên chỗ chồng nhau không đậm hơn.
    let path = canvas::Path::new(|p| {
        for (dx, dy, k) in [
            (-1.1, 0.25, 0.7),
            (0.0, 0.0, 1.0),
            (1.0, 0.2, 0.75),
            (0.45, 0.35, 0.6),
        ] {
            p.circle(Point::new(x + dx * r, y + dy * r), r * k);
        }
    });
    frame.fill(&path, white(0.08));
}

fn buddy(frame: &mut canvas::Frame, x: f32, y: f32, r: f32, color: Color, look: f32) {
    circle(frame, x, y, r, color);
    ellipse(
        frame,
        x - r * 0.35,
        y - r * 0.5,
        r * 0.22,
        r * 0.13,
        white(0.45),
    );
    for side in [-1.0, 1.0] {
        let ex = x + side * r * 0.3 + look * r * 0.08;
        let ey = y - r * 0.02;
        ellipse(frame, ex, ey, r * 0.16, r * 0.2, Color::WHITE);
        circle(frame, ex + look * r * 0.06, ey + r * 0.03, r * 0.09, INK);
        circle(
            frame,
            ex + look * r * 0.06 - r * 0.03,
            ey - r * 0.03,
            r * 0.03,
            Color::WHITE,
        );
        circle(
            frame,
            x + side * r * 0.55,
            y + r * 0.3,
            r * 0.11,
            alpha(PINK, 0.45),
        );
    }
    let mouth = canvas::Path::new(|p| {
        p.move_to(Point::new(x - r * 0.1 + look * r * 0.08, y + r * 0.3));
        p.quadratic_curve_to(
            Point::new(x + look * r * 0.08, y + r * 0.44),
            Point::new(x + r * 0.1 + look * r * 0.08, y + r * 0.3),
        );
    });
    frame.stroke(
        &mouth,
        canvas::Stroke::default()
            .with_color(INK)
            .with_width(r * 0.06)
            .with_line_cap(canvas::LineCap::Round),
    );
}

/// Ánh sáng nền phía sau toàn cửa sổ.
pub struct Backdrop;
impl<M> canvas::Program<M> for Backdrop {
    type State = canvas::Cache;
    fn draw(
        &self,
        cache: &canvas::Cache,
        renderer: &iced::Renderer,
        _: &Theme,
        bounds: Rectangle,
        _: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        vec![cache.draw(renderer, bounds.size(), |frame| {
            let (w, h) = (bounds.width, bounds.height);
            halo(
                frame,
                w * 0.80,
                h * 0.35,
                380.0,
                Color::from_rgb(0.55, 0.90, 1.0),
                0.30,
            );
            for (cx, cy, k) in [(0.10, 0.20, 1.0), (0.50, 0.10, 0.7), (0.92, 0.80, 0.9)] {
                cloud(frame, w * cx, h * cy, 34.0 * k);
            }
            halo(
                frame,
                w * 0.18,
                h * 1.0,
                340.0,
                Color::from_rgb(0.45, 0.95, 1.0),
                0.22,
            );
            halo(frame, w * 0.45, -40.0, 260.0, Color::WHITE, 0.16);
            for (x, y, s, a) in [
                (0.06, 0.22, 5.0, 0.5),
                (0.52, 0.14, 4.0, 0.4),
                (0.57, 0.62, 3.0, 0.35),
                (0.95, 0.88, 5.0, 0.4),
                (0.62, 0.92, 3.5, 0.3),
                (0.35, 0.08, 3.0, 0.3),
            ] {
                sparkle(frame, w * x, h * y, s, white(a));
            }
        })]
    }
}

/// Logo nhỏ cạnh chữ BOOM.
pub struct Mark;
impl<M> canvas::Program<M> for Mark {
    type State = ();
    fn draw(
        &self,
        _: &(),
        renderer: &iced::Renderer,
        _: &Theme,
        bounds: Rectangle,
        _: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let mut frame = canvas::Frame::new(renderer, bounds.size());
        bomb(&mut frame, 15.0, 22.0, 12.0, ORANGE);
        spark(&mut frame, 15.0, 22.0, 12.0, true);
        vec![frame.into_geometry()]
    }
}

/// Tranh minh hoạ: quả bom đang cháy ngòi giữa nhóm bạn trên sàn ô vuông.
pub struct Arcade {
    pub tick: usize,
}
impl<M> canvas::Program<M> for Arcade {
    type State = canvas::Cache;
    fn draw(
        &self,
        cache: &canvas::Cache,
        renderer: &iced::Renderer,
        _: &Theme,
        bounds: Rectangle,
        _: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let scene = cache.draw(renderer, bounds.size(), |frame| {
            let floor = 168.0;
            for x in 0..10 {
                for y in 0..2 {
                    let tile = canvas::Path::rounded_rectangle(
                        Point::new(x as f32 * 48.0 + 4.0, floor + y as f32 * 34.0),
                        Size::new(44.0, 30.0),
                        7.0.into(),
                    );
                    frame.fill(&tile, white(if (x + y) % 2 == 0 { 0.075 } else { 0.04 }));
                }
            }
            halo(frame, 240.0, 132.0, 112.0, ORANGE, 0.32);
            for (x, y, r) in [
                (240.0, 186.0, 56.0),
                (82.0, 190.0, 50.0),
                (398.0, 190.0, 40.0),
            ] {
                ellipse(
                    frame,
                    x,
                    y,
                    r,
                    r * 0.22,
                    Color::from_rgba(0.0, 0.0, 0.05, 0.4),
                );
            }
            bomb(frame, 240.0, 128.0, 56.0, Color::from_rgb(0.10, 0.15, 0.34));
            buddy(frame, 82.0, 142.0, 50.0, MINT, 1.0);
            buddy(frame, 398.0, 152.0, 40.0, VIOLET, -1.0);
            buddy(frame, 452.0, 52.0, 22.0, SUN, -1.0);
            for (x, y, r, c) in [
                (30.0, 40.0, 4.0, SUN),
                (150.0, 30.0, 3.0, PINK),
                (330.0, 40.0, 5.0, MINT),
                (360.0, 100.0, 3.0, SUN),
                (16.0, 110.0, 3.0, VIOLET),
            ] {
                circle(frame, x, y, r, c);
            }
            sparkle(frame, 120.0, 70.0, 7.0, white(0.8));
            sparkle(frame, 330.0, 76.0, 5.0, white(0.6));
        });
        let mut live = canvas::Frame::new(renderer, bounds.size());
        spark(&mut live, 240.0, 128.0, 56.0, self.tick.is_multiple_of(2));
        vec![scene, live.into_geometry()]
    }
}

#[cfg(test)]
mod tests {
    use super::decode_art;

    #[test]
    fn missing_or_invalid_art_selects_the_gradient() {
        assert!(decode_art(None).is_none());
        assert!(decode_art(Some(b"not a JPEG".to_vec())).is_none());
    }
}
