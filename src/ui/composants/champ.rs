//! Champs de saisie.
//!
//! Un champ est un ensemble « étiquette + contrôle + aide + erreur ». L'erreur
//! est toujours rendue **sous le champ concerné**, avec une icône, et le
//! contour du contrôle bascule en `danger` : jamais un message d'erreur global
//! détaché de sa cause.

use std::borrow::Borrow;

use iced::widget::{column, container, pick_list, row, text, text_input, PickList, TextInput};
use iced::{Alignment, Element, Length};

use crate::ui::composants::icone::{icone, Icone, Taille as TailleIcone};
use crate::ui::theme::espacements::Esp;
use crate::ui::theme::palette::Palette;
use crate::ui::theme::styles;
use crate::ui::theme::typographie::{police, police_chiffres, Role};

/// Identifiants stables des champs pouvant recevoir le focus au clavier.
pub const ID_MONTANT: &str = "champ-montant";
pub const ID_RECHERCHE: &str = "champ-recherche";

/// Contrôle de saisie texte, sans étiquette.
pub fn saisie<'a, Message: Clone + 'a>(
    valeur: &'a str,
    exemple: &'a str,
    au_changement: impl Fn(String) -> Message + 'a,
    en_erreur: bool,
    palette: Palette,
) -> TextInput<'a, Message> {
    text_input(exemple, valeur)
        .on_input(au_changement)
        .size(Role::Corps.taille())
        .font(police(Role::Corps.graisse()))
        .padding([Esp::MD - 2, Esp::MD])
        .style(move |_theme, statut| styles::champ(palette, statut, en_erreur))
}

/// Sélecteur déroulant thémé.
///
/// `options` accepte aussi bien une tranche empruntée qu'un vecteur possédé,
/// ce qui évite de fuir des options statiques à chaque rendu.
pub fn selecteur<'a, T, L, V, Message>(
    options: L,
    selection: Option<V>,
    au_choix: impl Fn(T) -> Message + 'a,
    palette: Palette,
) -> PickList<'a, T, L, V, Message>
where
    T: ToString + PartialEq + Clone + 'a,
    L: Borrow<[T]> + 'a,
    V: Borrow<T> + 'a,
    Message: Clone + 'a,
{
    pick_list(options, selection, au_choix)
        .text_size(Role::Corps.taille())
        .font(police(Role::Corps.graisse()))
        .padding([Esp::MD - 2, Esp::MD])
        .handle(pick_list::Handle::Arrow {
            size: Some(iced::Pixels(Role::Legende.taille() as f32)),
        })
        .style(move |_theme, statut| styles::selecteur(palette, statut))
        .menu_style(move |_theme| styles::menu_selecteur(palette))
}

/// Saisie de montant : chiffres uniquement, chasse fixe, validation à l'entrée.
///
/// Le filtrage est appliqué ici plutôt que dans `update` afin qu'aucun écran ne
/// puisse construire un champ montant acceptant des lettres.
pub fn saisie_montant<'a, Message: Clone + 'a>(
    valeur: &'a str,
    exemple: &'a str,
    au_changement: impl Fn(String) -> Message + 'a,
    a_la_validation: Message,
    en_erreur: bool,
    palette: Palette,
) -> TextInput<'a, Message> {
    text_input(exemple, valeur)
        .on_input(move |saisie| {
            au_changement(crate::domaine::argent::Money::filtrer_saisie(&saisie))
        })
        .on_submit(a_la_validation)
        .size(Role::Corps.taille())
        .font(police_chiffres(Role::Corps.graisse()))
        .padding([Esp::MD - 2, Esp::MD])
        .style(move |_theme, statut| styles::champ(palette, statut, en_erreur))
}

/// Saisie d'un **solde de compte** : comme le montant, mais le signe moins est
/// conservé — on peut être déjà à découvert.
pub fn saisie_solde<'a, Message: Clone + 'a>(
    valeur: &'a str,
    exemple: &'a str,
    au_changement: impl Fn(String) -> Message + 'a,
    a_la_validation: Message,
    en_erreur: bool,
    palette: Palette,
) -> TextInput<'a, Message> {
    text_input(exemple, valeur)
        .on_input(move |saisie| {
            au_changement(crate::domaine::argent::Money::filtrer_saisie_signee(
                &saisie,
            ))
        })
        .on_submit(a_la_validation)
        .size(Role::Corps.taille())
        .font(police_chiffres(Role::Corps.graisse()))
        .padding([Esp::MD - 2, Esp::MD])
        .style(move |_theme, statut| styles::champ(palette, statut, en_erreur))
}

/// Champ complet : étiquette, contrôle, aide facultative, erreur facultative.
pub fn champ<'a, Message: 'a>(
    etiquette: &'a str,
    controle: impl Into<Element<'a, Message>>,
    aide: Option<&'a str>,
    erreur: Option<&'a str>,
    palette: Palette,
) -> Element<'a, Message> {
    let mut bloc = column![
        text(etiquette)
            .size(Role::Libelle.taille())
            .font(police(Role::Libelle.graisse()))
            .color(palette.texte),
        controle.into(),
    ]
    .spacing(Esp::XS + 2)
    .width(Length::Fill);

    if let Some(erreur) = erreur {
        bloc = bloc.push(message_derreur(erreur, palette));
    } else if let Some(aide) = aide {
        bloc = bloc.push(
            text(aide)
                .size(Role::Legende.taille())
                .font(police(Role::Legende.graisse()))
                .color(palette.texte_doux),
        );
    }

    bloc.into()
}

/// Champ montant : traitement à part, car c'est la donnée que l'utilisateur
/// vient réellement saisir. Chiffres à chasse fixe, taille supérieure, devise
/// affichée en suffixe plutôt que laissée à la charge de la saisie.
pub fn champ_montant<'a, Message: Clone + 'a>(
    etiquette: &'a str,
    valeur: &'a str,
    devise: &'a str,
    au_changement: impl Fn(String) -> Message + 'a,
    a_la_validation: Message,
    erreur: Option<&'a str>,
    palette: Palette,
) -> Element<'a, Message> {
    let en_erreur = erreur.is_some();

    let saisie = text_input("0,00", valeur)
        .id(text_input::Id::new(ID_MONTANT))
        // Chiffres et séparateur décimal uniquement : une lettre ne peut pas
        // atteindre le champ, donc aucune erreur à afficher pour elle.
        .on_input(move |saisie| {
            au_changement(crate::domaine::argent::Money::filtrer_saisie(&saisie))
        })
        .on_submit(a_la_validation)
        .size(Role::MontantFort.taille())
        .font(police_chiffres(Role::MontantFort.graisse()))
        .padding([Esp::MD, Esp::MD])
        .style(move |_theme, statut| styles::champ(palette, statut, en_erreur))
        .width(Length::Fill);

    let unite = container(
        text(devise)
            .size(Role::TitreSection.taille())
            .font(police(Role::TitreSection.graisse()))
            .color(palette.texte_doux),
    )
    .padding([0, Esp::SM]);

    let ligne = row![saisie, unite]
        .align_y(Alignment::Center)
        .spacing(Esp::XS);

    champ(etiquette, ligne, None, erreur, palette)
}

/// Champ de recherche : icône intégrée, largeur fixe pour ne pas déséquilibrer
/// la barre de filtres.
pub fn champ_recherche<'a, Message: Clone + 'a>(
    valeur: &'a str,
    au_changement: impl Fn(String) -> Message + 'a,
    palette: Palette,
) -> Element<'a, Message> {
    text_input("Rechercher une transaction…", valeur)
        .id(text_input::Id::new(ID_RECHERCHE))
        .on_input(au_changement)
        .icon(text_input::Icon {
            font: iced::Font::DEFAULT,
            code_point: '\u{1F50D}',
            size: None,
            spacing: Esp::SM as f32,
            side: text_input::Side::Left,
        })
        .size(Role::Corps.taille())
        .font(police(Role::Corps.graisse()))
        .padding([Esp::SM + 1, Esp::MD])
        .style(move |_theme, statut| styles::champ(palette, statut, false))
        .width(Length::Fixed(240.0))
        .into()
}

/// Message d'erreur associé à un champ.
pub fn message_derreur<'a, Message: 'a>(erreur: &'a str, palette: Palette) -> Element<'a, Message> {
    row![
        icone(Icone::Alerte, TailleIcone::Petite, palette.danger),
        text(erreur)
            .size(Role::Legende.taille())
            .font(police(Role::Legende.graisse()))
            .color(palette.danger),
    ]
    .spacing(Esp::XS + 1)
    .align_y(Alignment::Center)
    .into()
}

/// Interrupteur binaire : une case et son libellé, sur une ligne cliquable.
///
/// Iced fournit `checkbox`, mais son style ne suit pas le design system ; celui
/// -ci reprend exactement le contrôle utilisé pour le statut d'une transaction,
/// afin que la même forme signifie partout la même chose.
pub fn interrupteur<'a, Message: Clone + 'a>(
    intitule: &'a str,
    aide: &'a str,
    actif: bool,
    au_changement: impl Fn(bool) -> Message + 'a,
    palette: Palette,
) -> Element<'a, Message> {
    let case = container(if actif {
        crate::ui::composants::icone::icone(
            crate::ui::composants::icone::Icone::Coche,
            crate::ui::composants::icone::Taille::Petite,
            palette.accent_contraste,
        )
    } else {
        iced::widget::Space::new(Length::Fixed(0.0), Length::Fixed(0.0)).into()
    })
    .width(Length::Fixed(18.0))
    .height(Length::Fixed(18.0))
    .align_x(Alignment::Center)
    .align_y(Alignment::Center)
    .style(move |_theme| iced::widget::container::Style {
        background: Some(iced::Background::Color(if actif {
            palette.accent
        } else {
            palette.surface_basse
        })),
        border: iced::Border {
            color: if actif {
                palette.accent
            } else {
                palette.bordure_forte
            },
            width: 1.0,
            radius: crate::ui::theme::espacements::Rayon::XS.into(),
        },
        ..iced::widget::container::Style::default()
    });

    iced::widget::button(
        row![
            case,
            column![
                text(intitule)
                    .size(Role::CorpsFort.taille())
                    .font(police(Role::CorpsFort.graisse()))
                    .color(palette.texte_fort),
                text(aide)
                    .size(Role::Legende.taille())
                    .font(police(Role::Legende.graisse()))
                    .color(palette.texte_doux),
            ]
            .spacing(Esp::XXS),
        ]
        .spacing(Esp::SM)
        .align_y(Alignment::Center),
    )
    .padding(Esp::SM)
    .width(Length::Fill)
    .on_press(au_changement(!actif))
    .style(move |_theme, statut| styles::bouton_fantome(palette, palette.texte, statut))
    .into()
}
