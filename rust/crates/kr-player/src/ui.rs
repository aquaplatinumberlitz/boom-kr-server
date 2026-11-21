//! Views of the launcher state. Offline remains a selectable preview for O1.
use crate::{
    Copy, Files, Language, Message, Mode, PlayState, Player, Screen, Server, desktop, look,
};
use iced::{
    Alignment, Color, Element, Font, Length,
    widget::{Space, button, canvas, column, container, pick_list, row, stack, text, text_input},
};
use zeroize::Zeroizing;

impl Player {
    fn t(&self, key: Copy) -> &'static str {
        self.language.get(key)
    }

    fn language_menu(&self) -> Element<'_, Message> {
        pick_list(Language::ALL, Some(self.language), Message::Language)
            .text_size(14)
            .padding([9, 12])
            .width(148)
            .style(look::dropdown)
            .into()
    }

    fn link(&self, key: Copy, screen: Screen) -> Element<'_, Message> {
        button(text(self.t(key)).size(13))
            .padding([6, 0])
            .style(look::link)
            .on_press(Message::Open(screen))
            .into()
    }

    fn server_status(&self, server: Server) -> Element<'_, Message> {
        let (key, tint, latency) = match server {
            Server::Ready(ms) => (Copy::ServerReady, look::MINT, Some(ms)),
            Server::Checking => (Copy::ServerChecking, look::SUN, None),
            Server::Offline => (Copy::ServerOffline, look::CORAL, None),
        };
        let mut line = row![
            container(Space::new().width(7).height(7)).style(look::dot(tint)),
            text(self.t(key)).size(13).color(look::MUTED)
        ]
        .align_y(Alignment::Center)
        .spacing(8);
        if let Some(ms) = latency {
            line = line.push(text(format!("{ms} ms")).size(12).color(tint));
        }
        line.into()
    }

    fn server_list(&self) -> Element<'_, Message> {
        let mut items = column![].spacing(5);
        for entry in &self.catalog {
            let selected = self.server_prefs.selected.as_ref() == Some(&entry.id);
            let name = entry
                .name
                .as_deref()
                .filter(|name| !name.trim().is_empty())
                .unwrap_or_else(|| self.t(Copy::BundleServer));
            let status = if selected {
                self.server
            } else {
                Server::Checking
            };
            let check = if selected { "●" } else { "○" };
            let item = row![
                text(check)
                    .size(12)
                    .color(if selected { look::SUN } else { look::MUTED }),
                column![
                    text(name)
                        .size(14)
                        .font(look::BOLD)
                        .wrapping(iced::widget::text::Wrapping::WordOrGlyph),
                    self.server_status(status),
                ]
                .spacing(3)
                .width(Length::Fill),
            ]
            .spacing(9)
            .align_y(Alignment::Center);
            items = items.push(
                button(item)
                    .padding([7, 10])
                    .width(Length::Fill)
                    .style(look::secondary)
                    .on_press_maybe(
                        (!self.starting && self.session.is_none())
                            .then(|| Message::Select(entry.id.clone())),
                    ),
            );
        }
        if self.catalog.is_empty() {
            items = items.push(text(self.t(Copy::NoServers)).size(13).color(look::MUTED));
        }
        let add = button(text(self.t(Copy::AddServer)).size(12))
            .padding([4, 7])
            .style(look::link);
        let soon = container(text(self.t(Copy::Soon)).size(10).color(look::SUN))
            .padding([1, 6])
            .style(look::pill(look::SUN));
        items
            .push(row![add, soon].align_y(Alignment::Center).spacing(5))
            .into()
    }

    fn modes(&self) -> Element<'_, Message> {
        let online = self.mode == Mode::Servers;
        let selector = |mode, selected, label: &'static str, soon| {
            let mut content = column![text(label).size(13).font(look::BOLD).center()]
                .align_x(Alignment::Center)
                .spacing(3);
            if soon {
                let color = if selected { look::DEEP } else { look::SUN };
                let marker = text(self.t(Copy::Soon))
                    .size(10)
                    .font(look::BOLD)
                    .color(color);
                content = content.push(container(marker).padding([1, 7]).style(look::pill(color)));
            }
            button(container(content).center_x(Length::Fill).center_y(45))
                .padding([1, 9])
                .width(if soon {
                    Length::Fill
                } else {
                    Length::Fixed(84.0)
                })
                .style(look::segment(selected))
                .on_press_maybe(
                    (!self.starting && self.session.is_none()).then_some(Message::Mode(mode)),
                )
        };
        container(
            row![
                selector(Mode::Servers, online, self.t(Copy::Servers), false),
                selector(Mode::OfflineFriends, !online, self.t(Copy::Offline), true),
            ]
            .spacing(3),
        )
        .padding(4)
        .style(look::segmented)
        .into()
    }

    fn loading(&self) -> Element<'_, Message> {
        let offset = self.tick % 18;
        let x = offset.min(18 - offset) as f32 * 23.0;
        container(row![
            Space::new().width(x),
            container(Space::new().width(63).height(4)).style(look::progress_fill),
        ])
        .height(4)
        .width(Length::Fill)
        .style(look::progress)
        .into()
    }

    fn play_panel(&self) -> Element<'_, Message> {
        let state = self.play_state();
        let description = match self.mode {
            Mode::Servers => Copy::NativeLogin,
            Mode::OfflineFriends => Copy::Friends,
        };
        let key = match state {
            PlayState::Ready => Copy::Ready,
            PlayState::Soon => Copy::Soon,
            PlayState::Checking => Copy::Checking,
            PlayState::Starting => Copy::Launching,
            PlayState::Running => Copy::Running,
            PlayState::WaitingForServer if self.server == Server::Offline => Copy::Unavailable,
            PlayState::WaitingForServer => Copy::ServerChecking,
            PlayState::Error => Copy::Error,
        };
        let primary = button(
            text(self.t(Copy::Play))
                .font(look::HEAVY)
                .size(22)
                .center()
                .width(Length::Fill),
        )
        .padding([14, 12])
        .width(Length::Fill)
        .style(look::play(false))
        .on_press_maybe((state == PlayState::Ready).then_some(Message::Play));
        let mut body = column![
            text(self.t(Copy::Title))
                .size(32)
                .font(look::HEAVY)
                .color(Color::WHITE),
            text(self.t(Copy::Mode))
                .size(12)
                .font(look::BOLD)
                .color(look::MUTED),
            self.modes(),
        ]
        .spacing(8);
        if self.mode == Mode::Servers {
            body = body.push(self.server_list());
        }
        body = body
            .push(text(self.t(description)).size(14).color(look::MUTED))
            .push(primary)
            .push(
                text(self.t(key))
                    .size(13)
                    .font(look::BOLD)
                    .color(match state {
                        PlayState::Ready => look::MINT,
                        PlayState::Error => look::CORAL,
                        _ => look::MUTED,
                    }),
            );
        if matches!(state, PlayState::Checking | PlayState::Starting) {
            body = body.push(self.loading());
        }
        if self.mode == Mode::Servers {
            if state == PlayState::WaitingForServer && self.server == Server::Offline {
                body = body.push(
                    button(text(self.t(Copy::Retry)).size(13))
                        .style(look::secondary)
                        .padding([6, 9])
                        .on_press_maybe(
                            (self.config.is_some() && !self.probing).then_some(Message::Retry),
                        ),
                );
            }
            if state == PlayState::Error {
                let action = |label, repair| {
                    button(text(self.t(label)).size(13))
                        .style(look::secondary)
                        .padding([6, 9])
                        .on_press_maybe(self.config.is_some().then_some(if repair {
                            Message::RepairFromHome
                        } else {
                            Message::Retry
                        }))
                };
                body = body
                    .push(
                        text(self.t(self.launch_error.unwrap_or(Copy::FilesBad)))
                            .size(13)
                            .color(look::CORAL),
                    )
                    .push(row![action(Copy::Retry, false), action(Copy::Repair, true)].spacing(8));
            }
            body = body.push(
                row![
                    self.link(Copy::Create, Screen::Register),
                    self.link(Copy::Forgot, Screen::Recover),
                    self.link(Copy::Manage, Screen::Menu)
                ]
                .spacing(11),
            );
        }
        if self.preference_error {
            body = body.push(text(self.t(Copy::PrefBad)).size(13).color(look::CORAL));
        }
        body.into()
    }

    fn footer(&self) -> Element<'_, Message> {
        let bundle = self
            .config
            .as_ref()
            .and_then(|c| c.manifest_sha256.get(..8))
            .filter(|prefix| prefix.bytes().all(|b| b.is_ascii_hexdigit()))
            .unwrap_or("—");
        let version = format!(
            "{} v{}  ·  {} {bundle}",
            self.t(Copy::Launcher),
            env!("CARGO_PKG_VERSION"),
            self.t(Copy::Package)
        );
        let info = text(version).size(11).color(look::MUTED);
        let help = self.link(Copy::Help, Screen::Help);
        let summary: Element<'_, Message> = if self
            .config
            .as_ref()
            .and_then(|cfg| cfg.release_url())
            .is_some()
        {
            let download = button(text(self.t(Copy::Download)).size(13))
                .padding([6, 0])
                .style(look::link)
                .on_press(Message::Desktop(desktop::Action::Release));
            column![info, row![help, download].spacing(12)].into()
        } else if self.size.width < 1100.0 {
            column![info, help].into()
        } else {
            row![info, help]
                .spacing(12)
                .align_y(Alignment::Center)
                .into()
        };
        container(summary)
            .padding([2, 10])
            .style(look::panel)
            .into()
    }

    fn modal_body(&self, screen: Screen) -> Element<'_, Message> {
        let wrap = |key| text(self.t(key)).size(14).color(look::MUTED);
        match screen {
            Screen::Settings => {
                let c = self.config.as_ref();
                let action = |key, message, enabled: bool| {
                    button(text(self.t(key)).size(14))
                        .padding([9, 12])
                        .style(look::secondary)
                        .on_press_maybe(enabled.then_some(message))
                };
                let folder = c
                    .map(|c| c.installation.display().to_string())
                    .unwrap_or_else(|| self.t(Copy::Unavailable).into());
                let items = column![
                    row![
                        wrap(Copy::Lang),
                        Space::new().width(Length::Fill),
                        self.language_menu()
                    ]
                    .align_y(Alignment::Center),
                    wrap(Copy::Folder),
                    row![
                        text(folder)
                            .size(14)
                            .wrapping(iced::widget::text::Wrapping::WordOrGlyph)
                            .width(Length::Fill),
                        action(
                            Copy::Open,
                            Message::Desktop(desktop::Action::Game),
                            c.is_some_and(|c| c.installation.is_dir())
                        )
                    ]
                    .align_y(Alignment::Center)
                    .spacing(12),
                    action(
                        Copy::CheckRepair,
                        Message::CheckFiles(true),
                        c.is_some()
                            && self.files != Files::Checking
                            && !self.starting
                            && self.session.is_none()
                    ),
                    action(
                        Copy::Logs,
                        Message::Desktop(desktop::Action::Logs),
                        c.is_some()
                    ),
                ]
                .spacing(14);
                #[cfg(windows)]
                let items = items.push(action(
                    Copy::Shortcut,
                    Message::Desktop(desktop::Action::Shortcut),
                    c.is_some(),
                ));
                if self.files == Files::Checking {
                    items.push(wrap(Copy::Checking)).push(self.loading()).into()
                } else {
                    items.into()
                }
            }
            Screen::Help => wrap(Copy::HelpBody).into(),
            Screen::Menu => column![
                self.link(Copy::Change, Screen::Change),
                self.link(Copy::Delete, Screen::Delete),
            ]
            .spacing(8)
            .into(),
            Screen::AccountResult => {
                if let Some(code) = &self.recovery {
                    column![
                        wrap(Copy::KeepCode),
                        text(format!("{}\n{}", &code[..32], &code[32..]))
                            .size(16)
                            .font(Font::MONOSPACE),
                        button(text(self.t(Copy::Clipboard)).size(14))
                            .style(look::secondary)
                            .padding([8, 12])
                            .on_press(Message::CopyCode),
                    ]
                    .spacing(12)
                    .into()
                } else {
                    Space::new().height(0).into()
                }
            }
            screen => {
                let field = |key: Copy, value, secret, on_input: fn(String) -> Message| {
                    let input = text_input(self.t(key), value)
                        .size(15)
                        .padding([8, 11])
                        .secure(secret)
                        .style(look::field)
                        .on_input_maybe((!self.submitting).then_some(on_input));
                    let input = if matches!(key, Copy::LoginId) {
                        input.id(crate::ACCOUNT_LOGIN)
                    } else {
                        input
                    };
                    column![text(self.t(key)).size(13).color(look::MUTED), input].spacing(4)
                };
                let mut form = column![field(
                    Copy::LoginId,
                    &self.form.login,
                    false,
                    Message::Login
                )]
                .spacing(10);
                let submit = match screen {
                    Screen::Register => {
                        form = form.push(field(
                            Copy::Name,
                            &self.form.display_name,
                            false,
                            Message::DisplayName,
                        ));
                        Copy::Create
                    }
                    Screen::Change => Copy::Change,
                    Screen::Delete => Copy::DeleteConfirm,
                    Screen::Recover => {
                        form = form.push(field(Copy::Code, &self.form.code, true, |s| {
                            Message::Code(Zeroizing::new(s))
                        }));
                        Copy::Forgot
                    }
                    _ => unreachable!(),
                };
                if matches!(screen, Screen::Change | Screen::Delete) {
                    form = form.push(field(Copy::Current, &self.form.current, true, |s| {
                        Message::Current(Zeroizing::new(s))
                    }));
                }
                if screen == Screen::Delete {
                    form = form.push(wrap(Copy::DeleteHint));
                } else {
                    form = form.push(field(
                        if screen == Screen::Register {
                            Copy::Password
                        } else {
                            Copy::New
                        },
                        &self.form.password,
                        true,
                        |s| Message::Password(Zeroizing::new(s)),
                    ));
                }
                let validation = self.form.error(screen);
                let feedback = if self.submitting {
                    Some(Copy::Sending)
                } else if let Some(hint) = validation {
                    Some(hint)
                } else if self.server == Server::Offline {
                    Some(Copy::ServerOffline)
                } else {
                    None
                };
                let feedback: Element<'_, Message> =
                    feedback.map_or_else(|| Space::new().into(), |hint| wrap(hint).into());
                form = form.push(container(feedback).height(27));
                form = form.push(
                    button(text(self.t(submit)).size(15).font(look::BOLD))
                        .style(look::play(screen == Screen::Delete))
                        .padding([10, 15])
                        .on_press_maybe(
                            (!self.submitting
                                && validation.is_none()
                                && matches!(self.server, Server::Ready(_)))
                            .then_some(Message::Submit),
                        ),
                );
                if matches!(screen, Screen::Change | Screen::Delete) {
                    form = form.push(self.link(Copy::Back, Screen::Menu));
                }
                form.into()
            }
        }
    }

    fn modal(&self, screen: Screen, height: f32) -> Element<'_, Message> {
        let title = match screen {
            Screen::Settings => Copy::Settings,
            Screen::Help => Copy::Help,
            Screen::Menu => Copy::Manage,
            Screen::Register => Copy::Create,
            Screen::Change => Copy::Change,
            Screen::Delete => Copy::Delete,
            Screen::Recover => Copy::Forgot,
            Screen::AccountResult if self.recovery.is_some() => Copy::Code,
            Screen::AccountResult => Copy::Manage,
        };
        let titlebar = row![
            text(self.t(title))
                .size(26)
                .font(look::HEAVY)
                .width(Length::Fill),
            button(text(self.t(Copy::Close)).size(13))
                .style(look::secondary)
                .padding([7, 11])
                .on_press_maybe((!self.submitting).then_some(Message::Dismiss)),
        ]
        .align_y(Alignment::Center)
        .spacing(8);
        let mut content = column![self.modal_body(screen)].spacing(14);
        if let Some((notice, error)) = self.modal_notice {
            content = content.push(text(self.t(notice)).size(14).color(if error {
                look::CORAL
            } else {
                look::MINT
            }));
        }
        let dialog = container(column![titlebar, content].spacing(18))
            .max_height(height - 48.0)
            .width(480)
            .padding(24)
            .style(look::modal);
        container(dialog)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .style(look::scrim)
            .into()
    }

    pub(super) fn view(&self) -> Element<'_, Message> {
        let size = self.size;
        let backdrop: Element<'_, Message> = if let Some(art) = &self.art {
            look::art(art, size).into()
        } else {
            canvas::Canvas::new(look::Backdrop)
                .width(Length::Fill)
                .height(Length::Fill)
                .into()
        };
        let toolbar = container(
            row![
                self.language_menu(),
                button(text(format!("⚙  {}", self.t(Copy::Settings))).size(14))
                    .padding([9, 13])
                    .style(look::toolbar)
                    .on_press(Message::Open(Screen::Settings)),
            ]
            .spacing(10)
            .align_y(Alignment::Center),
        )
        .align_right(Length::Fill);
        let card = container(self.play_panel())
            .padding(18)
            .width(374)
            .style(look::panel);
        let controls = container(
            column![
                toolbar,
                card,
                Space::new().height(Length::Fill),
                self.footer(),
            ]
            .spacing(12),
        )
        .padding([18, 24])
        .width(Length::Fill)
        .height(Length::Fill);
        let foreground = self
            .screen
            .map_or_else(|| controls.into(), |screen| self.modal(screen, size.height));
        let layers = stack![
            backdrop,
            container(Space::new().width(Length::Fill).height(Length::Fill)).style(look::shade),
            foreground,
        ]
        .width(Length::Fill)
        .height(Length::Fill);
        container(layers)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(look::window)
            .into()
    }
}
