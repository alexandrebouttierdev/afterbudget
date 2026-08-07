use crate::app::message::Message;
use iced::keyboard::{Key, Modifiers};

pub fn map_keyboard_event(key: &Key, modifiers: Modifiers) -> Option<Message> {
    let ctrl = modifiers.control() || modifiers.logo();

    match key {
        Key::Named(iced::keyboard::key::Named::Escape) => Some(Message::KeyboardEscape),
        _ if ctrl => match key {
            Key::Character(c) => match c.as_str() {
                "d" | "D" => Some(Message::OpenAddExpense),
                "r" | "R" => Some(Message::OpenAddIncome),
                "f" | "F" => Some(Message::FocusSearch),
                "s" | "S" => Some(Message::KeyboardSave),
                _ => None,
            },
            Key::Named(iced::keyboard::key::Named::ArrowLeft) => Some(Message::PreviousMonth),
            Key::Named(iced::keyboard::key::Named::ArrowRight) => Some(Message::NextMonth),
            _ => None,
        },
        _ => None,
    }
}
