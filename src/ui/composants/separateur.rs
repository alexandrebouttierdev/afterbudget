//! Filets de séparation.

use iced::widget::{container, Space};
use iced::{Element, Length};

use crate::ui::theme::palette::Palette;
use crate::ui::theme::styles;

/// Filet horizontal d'un pixel.
pub fn horizontal<'a, Message: 'a>(palette: Palette) -> Element<'a, Message> {
    container(Space::new(Length::Fill, Length::Fixed(1.0)))
        .width(Length::Fill)
        .height(Length::Fixed(1.0))
        .style(move |_theme| styles::surface_nue(palette, palette.bordure, 0.0))
        .into()
}

/// Filet vertical d'un pixel, pour séparer deux zones d'une même barre.
pub fn vertical<'a, Message: 'a>(hauteur: f32, palette: Palette) -> Element<'a, Message> {
    container(Space::new(Length::Fixed(1.0), Length::Fixed(hauteur)))
        .width(Length::Fixed(1.0))
        .height(Length::Fixed(hauteur))
        .style(move |_theme| styles::surface_nue(palette, palette.bordure, 0.0))
        .into()
}
