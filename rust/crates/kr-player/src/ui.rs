//! Views of the launcher state.
use crate::guide::{self, Section};
use crate::{
    Copy, Files, Language, Message, Mode, PlayState, Player, Screen, Server, desktop, look,
};
use iced::{
    Alignment, Color, Element, Font, Length,
    widget::{
        Space, button, canvas, column, container, image, pick_list, row, scrollable, stack, text,
        text_input,
    },
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
                .unwrap_or_else(|| self.t(Copy::BundleServer));
            let mut label = column![
                text(name)
                    .size(14)
                    .font(look::BOLD)
                    .wrapping(iced::widget::text::Wrapping::WordOrGlyph)
            ]
            .spacing(3);
            if selected {
                label = label.push(self.server_status(self.server));
            }
            let item = button(
                row![
                    text(if selected { "●" } else { "○" })
                        .size(12)
                        .color(if selected { look::SUN } else { look::MUTED }),
                    label
                ]
                .spacing(9)
                .align_y(Alignment::Center),
            )
            .padding([6, 9])
            .width(Length::Fill)
            .style(look::secondary)
            .on_press_maybe(
                self.server_editable()
                    .then(|| Message::Select(entry.id.clone())),
            );
            let mut line = row![item].spacing(5).align_y(Alignment::Center);
            if entry.invite.is_some() {
                line = line.push(
                    button(text(self.t(Copy::RemoveServer)).size(12))
                        .padding([6, 8])
                        .style(look::secondary)
                        .on_press_maybe(
                            self.server_editable()
                                .then(|| Message::RemoveServer(entry.id.clone())),
                        ),
                );
            }
            items = items.push(line);
        }
        if self.catalog.is_empty() {
            items = items.push(text(self.t(Copy::NoServers)).size(13).color(look::MUTED));
        }
        if self.catalog.len() > 1 {
            scrollable(items).height(72).into()
        } else {
            items.into()
        }
    }

    fn header(&self) -> Element<'_, Message> {
        column![
            row![
                image(look::logo()).width(34).height(34),
                text(self.t(Copy::Title))
                    .size(32)
                    .font(look::HEAVY)
                    .color(Color::WHITE)
            ]
            .spacing(9)
            .align_y(Alignment::Center),
            text(self.t(Copy::Mode))
                .size(12)
                .font(look::BOLD)
                .color(look::MUTED),
            row![
                self.modes(),
                button(text("?").font(look::BOLD).size(16))
                    .style(look::secondary)
                    .padding([8, 10])
                    .on_press(Message::Open(Screen::Help(Section::Compare)))
            ]
            .spacing(7)
            .align_y(Alignment::Center),
        ]
        .spacing(6)
        .into()
    }

    fn modes(&self) -> Element<'_, Message> {
        let online = self.mode == Mode::Servers;
        let selector = |mode, selected, label: &'static str| {
            let content = text(label).size(13).font(look::BOLD).center();
            button(container(content).center_x(Length::Fill).center_y(34))
                .padding([1, 9])
                .width(if mode == Mode::OfflineFriends {
                    Length::Fill
                } else {
                    Length::Fixed(84.0)
                })
                .style(look::segment(selected))
                .on_press_maybe(
                    (!self.starting && self.session.is_none() && !self.server_connecting)
                        .then_some(Message::Mode(mode)),
                )
        };
        container(
            row![
                selector(Mode::Servers, online, self.t(Copy::Servers)),
                selector(Mode::OfflineFriends, !online, self.t(Copy::PlayFriends),),
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

    fn friends_panel(&self) -> Element<'_, Message> {
        let enabled = self.can_friends();
        let hosting = self.config.as_ref().is_some_and(|c| c.friends.is_some());
        let running = self.session.is_some();
        let label = |key| text(self.t(key)).size(13).font(look::BOLD);
        let note = |key| text(self.t(key)).size(13).color(look::MUTED);
        let name = text_input("", &self.friends_name)
            .padding(6)
            .size(14)
            .style(look::field)
            .on_input_maybe(enabled.then_some(Message::FriendsName));
        let mut code = text_input("O1-…", &self.friends_code)
            .padding(6)
            .size(14)
            .style(look::field)
            .on_input_maybe(enabled.then_some(Message::FriendsCode));
        if enabled {
            code = code.on_submit(Message::FriendsJoin);
        }
        let action = |key, message: Option<Message>| {
            button(text(self.t(key)).size(13))
                .padding([6, 8])
                .style(look::secondary)
                .on_press_maybe(message)
        };
        let host_label = if running {
            if self.friends_host.is_some() {
                Copy::FriendsStop
            } else {
                Copy::StopGame
            }
        } else {
            Copy::FriendsHost
        };
        let primary = button(
            text(self.t(host_label))
                .size(15)
                .font(look::BOLD)
                .center()
                .width(Length::Fill),
        )
        .padding([9, 10])
        .width(Length::Fill)
        .style(look::play(running))
        .on_press_maybe(if running {
            Some(Message::Stop)
        } else {
            (enabled && hosting).then_some(Message::FriendsHost)
        });
        let mut host = column![primary].spacing(6);
        if let Some(room) = &self.friends_host {
            host = host.push(
                container(
                    column![
                        row![
                            label(Copy::FriendsCode),
                            Space::new().width(Length::Fill),
                            action(
                                if matches!(self.friends_notice, Some(Copy::Copied)) {
                                    Copy::Copied
                                } else {
                                    Copy::Clipboard
                                },
                                Some(Message::FriendsCopy)
                            )
                        ]
                        .align_y(Alignment::Center),
                        text(room.code())
                            .size(13)
                            .font(Font::MONOSPACE)
                            .color(look::MINT)
                            .wrapping(iced::widget::text::Wrapping::WordOrGlyph),
                        note(Copy::FriendsShare),
                    ]
                    .spacing(3),
                )
                .padding([6, 10])
                .width(Length::Fill)
                .style(look::inset),
            );
        }
        let mut join = column![
            label(Copy::FriendsJoin),
            row![
                code,
                action(Copy::FriendsJoin, enabled.then_some(Message::FriendsJoin))
            ]
            .spacing(6)
            .align_y(Alignment::Center),
            row![
                label(Copy::FriendsLan).width(Length::Fill),
                action(
                    Copy::FriendsFind,
                    (enabled && !self.friends_finding).then_some(Message::FriendsFind)
                )
            ]
            .spacing(6)
            .align_y(Alignment::Center),
        ]
        .spacing(5);
        if enabled && !self.friends_list.is_empty() {
            let mut rooms = column![].spacing(4);
            for room in &self.friends_list {
                let code = kr_p2p::Invite::room(room.route.node);
                let title = room
                    .host
                    .as_ref()
                    .map_or_else(|| code.to_string(), |host| format!("{host} · {code}"));
                rooms = rooms.push(
                    button(
                        text(title)
                            .size(12)
                            .wrapping(iced::widget::text::Wrapping::WordOrGlyph),
                    )
                    .width(Length::Fill)
                    .style(look::secondary)
                    .padding([5, 8])
                    .on_press(Message::FriendsSelect(room.route.clone())),
                );
            }
            join = join.push(scrollable(rooms).height(54));
        }
        let mut body = column![
            row![label(Copy::FriendsName).width(112), name]
                .spacing(6)
                .align_y(Alignment::Center),
            host,
        ]
        .spacing(7);
        if running {
            body = body.push(label(if self.friends_host.is_some() {
                Copy::FriendsRunning
            } else {
                Copy::Running
            }));
            let instruction = self.t(Copy::FriendsTrial);
            body = body.push(guide::rich_line(instruction, 13));
        } else {
            body = body.push(note(Copy::FriendsHostHint)).push(join);
        }
        if self.starting || self.files == Files::Checking {
            body = body.push(note(if self.starting {
                Copy::Launching
            } else {
                Copy::Checking
            }));
            body = body.push(self.loading());
        }
        if self.friends_finding {
            body = body.push(note(Copy::FriendsSearching));
        }
        if self.files == Files::Error {
            body = body.push(action(
                Copy::Repair,
                self.config.is_some().then_some(Message::RepairFromHome),
            ));
        }
        if let Some(notice) = self
            .friends_notice
            .or((!hosting).then_some(Copy::FriendsNoHost))
            && !matches!(notice, Copy::Copied)
        {
            body = body.push(text(self.t(notice)).size(13).color(
                if matches!(notice, Copy::FriendsNone) {
                    look::MUTED
                } else {
                    look::CORAL
                },
            ));
            if matches!(notice, Copy::FriendsVersion)
                && self.config.as_ref().and_then(|c| c.release_url()).is_some()
            {
                body = body.push(action(
                    Copy::Download,
                    Some(Message::Desktop(desktop::Action::Release)),
                ));
            }
        }
        if self.preference_error {
            body = body.push(text(self.t(Copy::PrefBad)).size(13).color(look::CORAL));
        }
        body.into()
    }

    fn play_panel(&self) -> Element<'_, Message> {
        if self.mode == Mode::OfflineFriends {
            return self.friends_panel();
        }
        let state = self.play_state();
        let key = match state {
            PlayState::Ready => Copy::Ready,
            PlayState::Inactive => Copy::Unavailable,
            PlayState::Checking => Copy::Checking,
            PlayState::Starting => Copy::Launching,
            PlayState::Running => Copy::Running,
            PlayState::WaitingForServer if self.server == Server::Offline => Copy::Unavailable,
            PlayState::WaitingForServer => Copy::ServerChecking,
            PlayState::Error => Copy::Error,
        };
        let primary = button(
            text(self.t(if state == PlayState::Running {
                Copy::StopGame
            } else {
                Copy::Play
            }))
            .font(look::HEAVY)
            .size(22)
            .center()
            .width(Length::Fill),
        )
        .padding([9, 12])
        .width(Length::Fill)
        .style(look::play(state == PlayState::Running))
        .on_press_maybe(match state {
            PlayState::Running => Some(Message::Stop),
            PlayState::Ready if !self.server_connecting => Some(Message::Play),
            _ => None,
        });
        let editable = self.server_editable();
        let mut invite = text_input(self.t(Copy::ServerInvite), &self.server_invite)
            .padding(6)
            .size(13)
            .style(look::field)
            .on_input_maybe(editable.then_some(Message::ServerInvite));
        if editable {
            invite = invite.on_submit(Message::AddServer);
        }
        let mut body = column![
            self.server_list(),
            row![
                invite,
                button(text(self.t(Copy::AddServer)).size(13))
                    .padding([6, 8])
                    .style(look::secondary)
                    .on_press_maybe(editable.then_some(Message::AddServer))
            ]
            .spacing(7)
            .align_y(Alignment::Center),
        ]
        .spacing(7);
        if self.server_connecting {
            body = body.push(
                text(self.t(Copy::ServerChecking))
                    .size(13)
                    .color(look::MUTED),
            );
        } else if let Some(notice) = self.server_notice {
            body = body.push(text(self.t(notice)).size(13).color(look::CORAL));
        }
        body = body
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
                body =
                    body.push(self.link(Copy::ConnectionGuide, Screen::Help(Section::Connection)));
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
                    .push(
                        row![
                            action(Copy::Retry, false),
                            action(Copy::Repair, true),
                            self.link(Copy::VersionGuide, Screen::Help(Section::Version))
                        ]
                        .spacing(8),
                    );
            }
            body = body
                .push(
                    row![
                        self.link(Copy::Create, Screen::Register),
                        self.link(Copy::Forgot, Screen::Recover),
                    ]
                    .spacing(11),
                )
                .push(
                    row![
                        self.link(Copy::Change, Screen::Change),
                        self.link(Copy::Delete, Screen::Delete)
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
        let help = self.link(Copy::QuickHelp, Screen::Help(Section::Overview));
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
        } else {
            row![info, help]
                .spacing(12)
                .align_y(Alignment::Center)
                .into()
        };
        container(summary)
            .padding([6, 12])
            .style(look::inset)
            .into()
    }

    fn modal_body(&self, screen: Screen, height: f32) -> Element<'_, Message> {
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
                let items = column![
                    row![
                        wrap(Copy::Lang),
                        Space::new().width(Length::Fill),
                        self.language_menu()
                    ]
                    .align_y(Alignment::Center),
                    action(
                        Copy::Folder,
                        Message::Desktop(desktop::Action::Game),
                        c.is_some_and(|c| c.installation.is_dir())
                    ),
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
                let items = items
                    .push(action(
                        Copy::Shortcut,
                        Message::Desktop(desktop::Action::Shortcut),
                        c.is_some(),
                    ))
                    .push(action(
                        Copy::Firewall,
                        Message::Desktop(desktop::Action::Firewall),
                        c.is_some_and(|c| c.friends.is_some()) && !self.firewall_busy,
                    ));
                if self.files == Files::Checking {
                    items.push(wrap(Copy::Checking)).push(self.loading()).into()
                } else {
                    items.into()
                }
            }
            Screen::Help(section) => guide::view(self.language, section, height),
            Screen::AccountResult => {
                if let Some(code) = &self.recovery {
                    column![
                        wrap(Copy::KeepCode),
                        text(format!("{}\n{}", &code[..32], &code[32..]))
                            .size(16)
                            .font(Font::MONOSPACE),
                        row![
                            button(text(self.t(Copy::Clipboard)).size(14))
                                .style(look::secondary)
                                .padding([8, 12])
                                .on_press(Message::CopyCode),
                            button(text(self.t(Copy::SaveCode)).size(14))
                                .style(look::secondary)
                                .padding([8, 12])
                                .on_press(Message::SaveCode)
                        ]
                        .spacing(8),
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
                        .secure(secret && (matches!(key, Copy::Code) || !self.form.reveal))
                        .style(look::field)
                        .on_input_maybe((!self.submitting).then_some(on_input));
                    let input = if !self.submitting {
                        input.on_submit(Message::Submit)
                    } else {
                        input
                    };
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
                form = form.push(
                    button(
                        text(self.t(if self.form.reveal {
                            Copy::HidePassword
                        } else {
                            Copy::ShowPassword
                        }))
                        .size(13),
                    )
                    .style(look::secondary)
                    .padding([5, 8])
                    .on_press_maybe((!self.submitting).then_some(Message::TogglePassword)),
                );
                let validation = self.form.error(screen);
                let feedback = if self.submitting {
                    Some(Copy::Sending)
                } else if let Some(hint) =
                    validation.filter(|_| self.form.edited && self.modal_notice.is_none())
                {
                    Some(hint)
                } else if self.server == Server::Offline {
                    Some(Copy::ServerOffline)
                } else {
                    None
                };
                let feedback: Element<'_, Message> =
                    feedback.map_or_else(|| Space::new().into(), |hint| wrap(hint).into());
                form = form.push(container(feedback));
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
                form.into()
            }
        }
    }

    fn modal(&self, screen: Screen, height: f32) -> Element<'_, Message> {
        let title = match screen {
            Screen::Settings => Copy::Settings,
            Screen::Help(_) => Copy::Help,
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
        let body_height = (height
            - 48.0
            - 48.0
            - 38.0
            - 18.0
            - if self.modal_notice.is_some() {
                62.0
            } else {
                0.0
            })
        .max(80.0);
        let body = self.modal_body(screen, body_height);
        let body: Element<_> = if matches!(screen, Screen::Help(_)) {
            body
        } else {
            scrollable(body).height(Length::Shrink).into()
        };
        let mut content = column![body].spacing(14);
        if let Some((notice, error)) = self.modal_notice {
            content = content.push(text(self.t(notice)).size(14).color(if error {
                look::CORAL
            } else {
                look::MINT
            }));
        }
        let dialog = container(column![titlebar, content].spacing(18))
            .max_height(height - 48.0)
            .width(
                (self.size.width - 48.0).min(if matches!(screen, Screen::Help(_)) {
                    820.0
                } else {
                    480.0
                }),
            )
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
                button(text(self.t(Copy::Settings)).size(14))
                    .padding([9, 13])
                    .style(look::toolbar)
                    .on_press(Message::Open(Screen::Settings)),
            ]
            .spacing(10)
            .align_y(Alignment::Center),
        )
        .align_right(Length::Fill);
        let footer = self.footer();
        let reserve = if self.config.as_ref().and_then(|c| c.release_url()).is_some() {
            174.0
        } else {
            155.0
        };
        let card = container(
            column![
                self.header(),
                scrollable(self.play_panel())
                    .height(Length::Shrink)
                    .spacing(5)
                    .direction(scrollable::Direction::Vertical(
                        scrollable::Scrollbar::new().width(4).scroller_width(4)
                    ))
            ]
            .spacing(10),
        )
        .padding(15)
        .max_height((size.height - reserve).max(240.0))
        .width(374)
        .style(look::panel);
        let controls = container(
            column![toolbar, card, Space::new().height(Length::Fill), footer,].spacing(12),
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
