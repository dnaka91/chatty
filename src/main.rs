#![allow(dead_code)]

use std::{io::Cursor, iter, sync::LazyLock};

use anyhow::Result;
use iced::{
    application, border,
    daemon::DefaultStyle,
    font,
    widget::{
        self, button, column, container, horizontal_space, markdown, row, scrollable, text,
        text_input, Column, Row, Text,
    },
    window, Center, Color, Element, Fill, Font, Task, Theme,
};
use jiff::ToSpan;

mod icons;
mod twitch;

const THEME: Theme = Theme::CatppuccinFrappe;

const ICON_FONT_NAME: &str = "Material Symbols Outlined";
const ICON_FONT_DATA: &[u8] = include_bytes!("../fonts/MaterialSymbolsOutlined-Regular.ttf");

const WINDOW_ICON: &[u8] = include_bytes!("../logo/logo.png");

static TAB_SCROLL: LazyLock<scrollable::Id> = LazyLock::new(scrollable::Id::unique);
static CHAT_SCROLL: LazyLock<scrollable::Id> = LazyLock::new(scrollable::Id::unique);

fn main() -> Result<()> {
    let overlay = overlay_mode();
    let decorations = !overlay || with_decorations();

    let icon = {
        let mut reader = image::ImageReader::new(Cursor::new(WINDOW_ICON));
        reader.set_format(image::ImageFormat::Png);

        let image = reader.decode()?.into_rgba8();
        let width = image.width();
        let height = image.height();

        window::icon::from_rgba(image.into_raw(), width, height)?
    };

    iced::application("Chatty", App::update, App::view)
        .font(ICON_FONT_DATA)
        .theme(|_| THEME)
        .style(App::style)
        .window(window::Settings {
            decorations,
            transparent: overlay,
            icon: Some(icon),
            ..window::Settings::default()
        })
        .run_with(move || App::new(overlay))?;

    Ok(())
}

fn overlay_mode() -> bool {
    std::env::args_os().any(|arg| arg == "--overlay")
}

fn with_decorations() -> bool {
    std::env::args_os().any(|arg| arg == "--with-decorations")
}

struct App {
    input: String,
    items: Vec<TwitchMessage>,
    tabs: Vec<u64>,
    active_tab: Option<u64>,
    overlay: bool,
}

struct TwitchMessage {
    sent: jiff::Zoned,
    user: TwitchUser,
    content: Vec<markdown::Item>,
}

struct TwitchUser {
    name: String,
    color: Color,
}

#[derive(Clone, Debug)]
enum Message {
    Input(String),
    Send(String),
    LinkClicked(markdown::Url),
    OpenSettings,
    OpenAccount,
    OpenEmojiPanel,
    SwitchTab(u64),
    AddTab,
    CloseTab(u64),
}

impl App {
    fn new(overlay: bool) -> (Self, Task<Message>) {
        (
            Self {
                input: String::new(),
                items: vec![
                    TwitchMessage {
                        sent: jiff::Zoned::now(),
                        user: TwitchUser {
                            name: "Bob".to_owned(),
                            color: Color::from_rgb8(230, 100, 60),
                        },
                        content: markdown::parse("This is **awesome**!").collect(),
                    },
                    TwitchMessage {
                        sent: jiff::Zoned::now().checked_add(2.minutes()).unwrap(),
                        user: TwitchUser {
                            name: "Alice".to_owned(),
                            color: Color::from_rgb8(50, 140, 255),
                        },
                        content: markdown::parse("_Twitch_ chat with [Markdown](https://daringfireball.net/projects/markdown/), yeah").collect(),
                    },
                ],
                tabs: vec![1, 2],
                active_tab: Some(1),
                overlay,
            },
            widget::focus_next(),
        )
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Input(input) => self.input = input,
            Message::Send(message) => {
                if message.trim().is_empty() {
                    return Task::none();
                }

                self.items.push(TwitchMessage {
                    sent: jiff::Zoned::now(),
                    user: TwitchUser {
                        name: "Me".to_owned(),
                        color: Color::from_rgb8(255, 255, 255),
                    },
                    content: markdown::parse(&message).collect(),
                });
                self.input = String::new();

                return scrollable::snap_to(CHAT_SCROLL.clone(), scrollable::RelativeOffset::END);
            }
            Message::LinkClicked(url) => {
                let _ = open::that_in_background(url.to_string());
            }
            Message::OpenSettings => {
                eprintln!("TODO: open settings");
            }
            Message::OpenAccount => {
                eprintln!("TODO: open account");
            }
            Message::OpenEmojiPanel => {
                eprintln!("TODO: open emoji panel");
            }
            Message::SwitchTab(tab) => {
                self.active_tab = Some(tab);
            }
            Message::AddTab => {
                self.tabs
                    .push(self.tabs.iter().copied().max().unwrap_or(0) + 1);

                return scrollable::snap_to(TAB_SCROLL.clone(), scrollable::RelativeOffset::END);
            }
            Message::CloseTab(tab) => {
                self.tabs.retain(|&id| id != tab);
                self.active_tab.take_if(|&mut id| id == tab);
            }
        }

        Task::none()
    }

    fn view(&self) -> Element<Message> {
        let settings = button(icon(icons::SETTINGS)).on_press(Message::OpenSettings);
        let account = button(row![icon(icons::PERSON), text("username")].align_y(Center))
            .on_press(Message::OpenAccount);

        let tab_add = button(icon(icons::ADD))
            .style(tab_add_style)
            .on_press(Message::AddTab);

        let tabs = self
            .tabs
            .iter()
            .map(|&id| {
                row![
                    button(text(format!("Tab {id}")))
                        .style(tab_style)
                        .padding([10.18, 30.0])
                        .on_press(Message::SwitchTab(id)),
                    button(icon(icons::CLOSE))
                        .style(tab_close_style)
                        .padding([5, 0])
                        .on_press(Message::CloseTab(id))
                ]
                .into()
            })
            .chain(iter::once(tab_add.into()))
            .collect::<Row<_>>()
            .spacing(4)
            .align_y(Center);

        let tab_bar = scrollable(tabs)
            .direction(scrollable::Direction::Horizontal(
                scrollable::Scrollbar::default().width(4).scroller_width(4),
            ))
            .spacing(2);

        let chat = Column::with_children(self.items.iter().map(|msg| {
            row![
                text(msg.sent.time().strftime("%H:%M").to_string()),
                text(format!("{}:", msg.user.name))
                    .color(msg.user.color)
                    .font(Font {
                        weight: font::Weight::Bold,
                        ..Font::DEFAULT
                    }),
                markdown(
                    &msg.content,
                    markdown::Settings::default(),
                    markdown::Style::from_palette(THEME.palette())
                )
                .map(Message::LinkClicked)
            ]
            .spacing(3)
            .into()
        }))
        .width(Fill)
        .spacing(5);

        let input = text_input("Type a message...", &self.input)
            .on_input(Message::Input)
            .on_submit(Message::Send(self.input.clone()))
            .padding(10);

        let send = button(icon(icons::SEND))
            .on_press(Message::Send(self.input.clone()))
            .padding([5, 10]);

        let emojis = button(icon(icons::MOOD))
            .on_press(Message::OpenEmojiPanel)
            .padding([5, 10]);

        column![
            row![
                container(row![tab_bar, horizontal_space()])
                    .padding(4)
                    .style(container::dark),
                container(row![settings, account].spacing(4))
                    .padding(4)
                    .style(container::dark)
            ]
            .align_y(Center)
            .spacing(4),
            scrollable(chat)
                .id(CHAT_SCROLL.clone())
                .spacing(10)
                .height(Fill),
            row![input, send, emojis].spacing(4),
        ]
        .spacing(4)
        .padding(4)
        .into()
    }

    fn style(&self, theme: &Theme) -> application::Appearance {
        let default = Theme::default_style(theme);

        if self.overlay {
            application::Appearance {
                background_color: Color::TRANSPARENT,
                ..default
            }
        } else {
            default
        }
    }
}

fn icon(unicode: char) -> Text<'static> {
    text(unicode)
        .font(Font::with_name(ICON_FONT_NAME))
        .size(24)
        .align_x(Center)
}

fn tab_style(theme: &Theme, status: button::Status) -> button::Style {
    let style = button::secondary(theme, status);
    button::Style {
        border: border::rounded(border::left(2)),
        ..style
    }
}

fn tab_close_style(theme: &Theme, status: button::Status) -> button::Style {
    let style = button::secondary(theme, status);
    button::Style {
        border: border::rounded(border::right(2)),
        ..style
    }
}

fn tab_add_style(theme: &Theme, status: button::Status) -> button::Style {
    let style = button::secondary(theme, status);
    button::Style {
        background: if status == button::Status::Active {
            None
        } else {
            style.background
        },
        ..style
    }
}
