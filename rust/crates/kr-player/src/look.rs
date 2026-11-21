//! Màu, kiểu chữ, kiểu widget và hình vẽ của launcher (chỉ phần nhìn, không có logic).
use iced::theme::Base;
use iced::{
    Background, Border, Color, ContentFit, Font, Gradient, Length, Point, Radians, Rectangle,
    Shadow, Size, Theme, Vector,
    font::Weight,
    gradient::Linear,
    mouse,
    widget::{button, canvas, container, image, pick_list, text_input},
};
use std::f32::consts::FRAC_PI_2;

pub const INK: Color = Color::from_rgb(0.03, 0.10, 0.26);
pub const DEEP: Color = Color::from_rgb(0.03, 0.17, 0.46);
pub const SKY: Color = Color::from_rgb(0.09, 0.56, 0.94);
pub const MUTED: Color = Color::from_rgb(0.80, 0.90, 1.0);
pub const ORANGE: Color = Color::from_rgb(1.0, 0.50, 0.12);
pub const SUN: Color = Color::from_rgb(1.0, 0.82, 0.24);
pub const MINT: Color = Color::from_rgb(0.36, 0.95, 0.75);
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

pub fn surface(theme: &Theme, modal_open: bool, paint: u8) -> iced::theme::Style {
    let mut style = theme.base();
    // Invalidate the full TinySkia surface on modal edits. Its damaged
    // regions otherwise leave stale pieces where the dimmer overlaps controls.
    style.background_color = if modal_open {
        Color::from_rgb(0.03, 0.10, 0.25 + f32::from(paint) * 0.00004)
    } else {
        DEEP
    };
    style
}

pub fn shade(_: &Theme) -> container::Style {
    container::Style::default().background(linear(FRAC_PI_2, alpha(INK, 0.51), alpha(INK, 0.0)))
}

pub fn panel(_: &Theme) -> container::Style {
    container::Style {
        background: Some(alpha(INK, 0.91).into()),
        border: Border {
            color: white(0.30),
            width: 1.0,
            radius: 22.0.into(),
        },
        shadow: glow(alpha(INK, 0.38), 8.0, 24.0),
        ..Default::default()
    }
}

pub fn scrim(_: &Theme) -> container::Style {
    container::Style::default().background(alpha(INK, 0.60))
}

pub fn modal(_: &Theme) -> container::Style {
    container::Style {
        background: Some(Color::from_rgb8(15, 33, 67).into()),
        border: Border {
            color: white(0.32),
            width: 1.0,
            radius: 22.0.into(),
        },
        shadow: glow(alpha(INK, 0.58), 12.0, 36.0),
        ..Default::default()
    }
}

pub fn dropdown(_: &Theme, status: pick_list::Status) -> pick_list::Style {
    pick_list::Style {
        text_color: Color::WHITE,
        placeholder_color: MUTED,
        handle_color: MUTED,
        background: alpha(INK, 0.96).into(),
        border: Border {
            color: if status == pick_list::Status::Active {
                white(0.25)
            } else {
                SUN
            },
            width: 1.0,
            radius: 10.0.into(),
        },
    }
}

pub fn secondary(_: &Theme, status: button::Status) -> button::Style {
    button::Style {
        text_color: if status == button::Status::Disabled {
            white(0.45)
        } else {
            Color::WHITE
        },
        background: Some(
            match status {
                button::Status::Hovered => white(0.20),
                button::Status::Pressed => white(0.09),
                _ => white(0.12),
            }
            .into(),
        ),
        border: Border {
            color: white(0.18),
            width: 1.0,
            radius: 10.0.into(),
        },
        ..Default::default()
    }
}

pub fn toolbar(theme: &Theme, status: button::Status) -> button::Style {
    button::Style {
        background: Some(
            alpha(
                INK,
                if status == button::Status::Hovered {
                    0.78
                } else {
                    0.95
                },
            )
            .into(),
        ),
        ..secondary(theme, status)
    }
}

pub fn link(_: &Theme, status: button::Status) -> button::Style {
    button::Style {
        text_color: match status {
            button::Status::Hovered => SUN,
            button::Status::Pressed => ORANGE,
            button::Status::Disabled => white(0.45),
            button::Status::Active => MUTED,
        },
        ..Default::default()
    }
}

pub fn progress(_: &Theme) -> container::Style {
    container::Style::default()
        .background(white(0.13))
        .border(Border::default().rounded(8))
}

pub fn progress_fill(_: &Theme) -> container::Style {
    container::Style::default()
        .background(SUN)
        .border(Border::default().rounded(8))
}

pub fn art(handle: &image::Handle, view: Size) -> image::Image<image::Handle> {
    let image::Handle::Rgba { width, height, .. } = handle else {
        unreachable!("decode_art creates a RGBA handle")
    };
    image::Image::new(handle.clone())
        .width(Length::Fill)
        .height(Length::Fill)
        .crop(cover_crop(*width, *height, view))
        .content_fit(ContentFit::Cover)
}

fn cover_crop(width: u32, height: u32, view: Size) -> Rectangle<u32> {
    let ratio = view.width.max(1.0) / view.height.max(1.0);
    let crop_width = ((height as f32 * ratio).floor() as u32).clamp(1, width);
    let crop_height = ((width as f32 / ratio).floor() as u32).clamp(1, height);
    Rectangle {
        // At narrow ratios trim mostly the left border; the logo spans the
        // middle and right. At wide ratios keep the entire bottom edge.
        x: (width - crop_width) * 9 / 10,
        y: height - crop_height,
        width: crop_width,
        height: crop_height,
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
        let pressed = status == button::Status::Pressed;
        button::Style {
            background: if selected {
                Some(Color::WHITE.into())
            } else if hovered || pressed {
                Some(white(if pressed { 0.05 } else { 0.12 }).into())
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

#[cfg(test)]
mod tests {
    use super::{cover_crop, decode_art};
    use iced::Size;

    #[test]
    fn missing_or_invalid_art_selects_the_gradient() {
        assert!(decode_art(None).is_none());
        assert!(decode_art(Some(b"not a JPEG".to_vec())).is_none());
    }

    #[test]
    fn cover_crops_top_or_left_while_keeping_the_logo_edge() {
        let wide = cover_crop(960, 718, Size::new(1280.0, 720.0));
        assert_eq!(
            (wide.x, wide.y, wide.width, wide.height),
            (0, 178, 960, 540)
        );
        let tall = cover_crop(960, 718, Size::new(960.0, 900.0));
        assert_eq!((tall.y, tall.height), (0, 718));
        assert!(tall.x > 0);
        assert!(tall.x + tall.width >= 940);
    }
}
