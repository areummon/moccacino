
use std::time::Duration;

use iced::time::Instant;
use iced::widget::{button, column, container, row, Space};
use iced::{Alignment, Element, Length, Task};

use super::icons::{icon, Icon, Ink};
use super::message::Message;
use super::widgets::text;
use crate::gui::theme::{self, Tone};

pub(crate) const TOAST_TICK: Duration = Duration::from_millis(400);
const TOAST_LIFETIME: Duration = Duration::from_millis(3200);
const TOAST_LIFETIME_LONG: Duration = Duration::from_millis(7000);
const MAX_TOASTS: usize = 4;

#[derive(Debug, Clone)]
pub(crate) struct Toast {
    pub(crate) id: u64,
    pub(crate) title: String,
    pub(crate) body: Option<String>,
    pub(crate) tone: Tone,
    pub(crate) expires_at: Instant,
}

impl super::app::App {
    pub(crate) fn toast(&mut self, tone: Tone, title: impl Into<String>, body: Option<String>) {
        let lifetime = if body.is_some() { TOAST_LIFETIME_LONG } else { TOAST_LIFETIME };
        self.next_toast_id += 1;
        self.toasts.push(Toast {
            id: self.next_toast_id,
            title: title.into(),
            body,
            tone,
            expires_at: Instant::now() + lifetime,
        });
        if self.toasts.len() > MAX_TOASTS {
            self.toasts.remove(0);
        }
    }

    pub(crate) fn toast_tick(&mut self) -> Task<Message> {
        let now = Instant::now();
        self.toasts.retain(|toast| toast.expires_at > now);
        Task::none()
    }

    pub(crate) fn dismiss_toast(&mut self, id: u64) -> Task<Message> {
        self.toasts.retain(|toast| toast.id != id);
        Task::none()
    }

    pub(crate) fn create_toast_layer(&self) -> Element<'_, Message> {
        let mut stack = column![].spacing(8).align_x(Alignment::End);
        for toast in &self.toasts {
            let (glyph, ink) = match toast.tone {
                Tone::Success => (Icon::Check, Ink::Success),
                Tone::Danger => (Icon::Close, Ink::Danger),
                Tone::Warning => (Icon::Warn, Ink::Accent),
                Tone::Accent | Tone::Neutral => (Icon::Check, Ink::Accent),
            };
            let mut content = column![text(toast.title.clone()).size(14).font(theme::SEMIBOLD)].spacing(4);
            if let Some(body) = &toast.body {
                content = content.push(text(body.clone()).size(12).style(theme::text_dim));
            }
            stack = stack.push(
                button(
                    container(
                        row![icon(glyph, 18.0, ink), content]
                            .spacing(10)
                            .align_y(Alignment::Start),
                    )
                    .padding([10, 14])
                    .max_width(380)
                    .style(theme::toast(toast.tone)),
                )
                .padding(0)
                .on_press(Message::DismissToast(toast.id))
                .style(|_theme: &iced::Theme, _status| button::Style::default()),
            );
        }
        container(column![Space::with_height(Length::Fill), stack].align_x(Alignment::End))
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(iced::Padding { top: 0.0, right: 18.0, bottom: 40.0, left: 0.0 })
            .align_x(iced::alignment::Horizontal::Right)
            .into()
    }
}
