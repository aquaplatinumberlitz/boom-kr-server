//! A small renderer for the bundled player guide's Markdown subset. It has no
//! external resources. The binary embeds the same files distributed in player/.
use crate::{Language, Message, Screen, look};
use iced::{
    Element, Length,
    widget::{button, column, container, rich_text, row, scrollable, span, text},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Section {
    Overview,
    Compare,
    Choose,
    Servers,
    Friends,
    Connection,
    Version,
    Firewall,
}

impl Section {
    pub const ALL: [Self; 8] = [
        Self::Overview,
        Self::Compare,
        Self::Choose,
        Self::Servers,
        Self::Friends,
        Self::Connection,
        Self::Version,
        Self::Firewall,
    ];

    fn index(self) -> usize {
        self as usize
    }

    pub fn scroll_id(self) -> &'static str {
        [
            "guide-overview",
            "guide-compare",
            "guide-choose",
            "guide-servers",
            "guide-friends",
            "guide-connection",
            "guide-version",
            "guide-firewall",
        ][self.index()]
    }
}

fn source(language: Language) -> &'static str {
    match language {
        Language::Vi => include_str!("../../../../docs/player/guide.vi.md"),
        Language::En => include_str!("../../../../docs/player/guide.en.md"),
        Language::Ko => include_str!("../../../../docs/player/guide.ko.md"),
    }
}

fn sections(language: Language) -> impl Iterator<Item = (&'static str, &'static str)> {
    source(language)
        .split("\n## ")
        .skip(1)
        .filter_map(|section| {
            let (title, body) = section.split_once('\n')?;
            Some((title, body.split("<!--").next()?.trim()))
        })
}

fn cells(line: &str) -> impl Iterator<Item = &str> {
    line.trim_matches('|').split('|').map(str::trim)
}

fn rendered_table_line(line: &str, head: bool) -> Element<'static, Message> {
    let mut table_row = row![].spacing(1).width(Length::Fill);
    for (i, cell) in cells(line).enumerate() {
        let label = text(cell.to_owned())
            .size(13)
            .font(if head || i == 0 {
                look::BOLD
            } else {
                iced::Font::DEFAULT
            })
            .wrapping(iced::widget::text::Wrapping::WordOrGlyph)
            .color(if head {
                iced::Color::WHITE
            } else {
                look::MUTED
            });
        table_row = table_row.push(
            container(label)
                .padding([7, 8])
                .width(Length::FillPortion(if i == 0 { 2 } else { 4 })),
        );
    }
    container(table_row).style(look::inset).into()
}

pub(crate) fn rich_line(line: &str, size: u32) -> Element<'static, Message> {
    let spans = line
        .split("**")
        .enumerate()
        .map(|(index, part)| {
            let value = span(part.to_owned());
            if index % 2 == 1 {
                value.font(look::BOLD)
            } else {
                value
            }
        })
        .collect::<Vec<_>>();
    rich_text::<(), Message, _, _>(spans)
        .size(size)
        .color(look::MUTED)
        .width(Length::Fill)
        .wrapping(iced::widget::text::Wrapping::WordOrGlyph)
        .into()
}

fn render_markdown(body: &str) -> Element<'static, Message> {
    let mut content = column![].spacing(9).width(Length::Fill);
    let mut table_head = true;
    for line in body.lines().map(str::trim).filter(|line| !line.is_empty()) {
        // The bundled Markdown displays the screenshot in document readers.
        if line.starts_with("![") {
            continue;
        }
        if line.starts_with('|') {
            if !cells(line).all(|cell| cell.chars().all(|ch| matches!(ch, '-' | ':'))) {
                content = content.push(rendered_table_line(line, table_head));
                table_head = false;
            }
        } else {
            table_head = true;
            let line = line
                .strip_prefix("- ")
                .map_or_else(|| line.to_owned(), |item| format!("•  {item}"));
            content = if let Some(title) = line.strip_prefix("### ") {
                content.push(
                    text(title.to_owned())
                        .size(17)
                        .font(look::BOLD)
                        .wrapping(iced::widget::text::Wrapping::WordOrGlyph),
                )
            } else {
                content.push(rich_line(&line, 15))
            };
        }
    }
    content.into()
}

pub(crate) fn view(
    language: Language,
    selected: Section,
    available: f32,
) -> Element<'static, Message> {
    let sections: Vec<_> = sections(language).collect();
    let mut nav = column![].spacing(5).width(160);
    for (section, (title, _)) in Section::ALL.into_iter().zip(&sections) {
        nav = nav.push(
            button(
                text(*title)
                    .size(13)
                    .width(Length::Fill)
                    .wrapping(iced::widget::text::Wrapping::WordOrGlyph),
            )
            .width(Length::Fill)
            .padding([8, 9])
            .style(look::guide_nav(section == selected))
            .on_press(Message::Open(Screen::Help(section))),
        );
    }
    let (title, body) = sections[selected.index()];
    let content = column![
        text(title)
            .size(24)
            .font(look::HEAVY)
            .color(iced::Color::WHITE),
        render_markdown(body),
    ]
    .spacing(12)
    .width(Length::Fill);
    row![
        scrollable(nav).height(Length::Fill),
        scrollable(container(content).padding([0, 10]))
            .id(selected.scroll_id())
            .spacing(7)
            .width(Length::Fill)
            .height(Length::Fill),
    ]
    .spacing(12)
    .height(available)
    .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_locales_provide_every_context_section_and_two_mode_table() {
        for language in Language::ALL {
            let sections: Vec<_> = sections(language).collect();
            assert_eq!(sections.len(), Section::ALL.len());
            for (title, body) in &sections {
                assert!(!title.is_empty() && !body.is_empty());
            }
            let rows: Vec<_> = sections[Section::Compare.index()]
                .1
                .lines()
                .filter(|line| line.starts_with('|'))
                .collect();
            assert_eq!(rows.len(), 9);
            assert!(rows.iter().all(|line| cells(line).count() == 3));
        }
    }
}
