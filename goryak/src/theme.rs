//! See https://material-foundation.github.io/material-theme-builder/
//! for the source of the default theme.
//! and https://m3.material.io/styles/color/roles for explanations of the role with more detail

#![allow(dead_code)]

use lazy_static::lazy_static;
use nanoserde::{DeJson, DeJsonErr};
use serde::{Deserialize, Serialize};
use std::ops::Deref;
use std::sync::{RwLock, RwLockReadGuard};
use yakui_core::geometry::Color;

/// Use primary roles for the most prominent components across the UI, such as the FAB
/// high-emphasis buttons, and active states.
pub fn primary() -> Color {
    THEMER.read().unwrap().cur_colors.primary
}

/// Text and icons against primary
pub fn on_primary() -> Color {
    THEMER.read().unwrap().cur_colors.on_primary
}

/// Use secondary roles for less prominent components in the UI such as filter chips.
pub fn secondary() -> Color {
    THEMER.read().unwrap().cur_colors.secondary
}

/// Text and icons against secondary
pub fn on_secondary() -> Color {
    THEMER.read().unwrap().cur_colors.on_secondary
}

/// Use tertiary roles for contrasting accents that balance primary and secondary colors
/// or bring heightened attention to an element such as an input field.
pub fn tertiary() -> Color {
    THEMER.read().unwrap().cur_colors.tertiary
}

/// Text and icons against tertiary
pub fn on_tertiary() -> Color {
    THEMER.read().unwrap().cur_colors.on_tertiary
}

/// Use error roles for components that communicate that an error has occurred.
pub fn error() -> Color {
    THEMER.read().unwrap().cur_colors.error
}

/// Text and icons against error
pub fn on_error() -> Color {
    THEMER.read().unwrap().cur_colors.on_error
}

/// Use background roles for the background color of components such as cards, sheets, and menus.
pub fn background() -> Color {
    THEMER.read().unwrap().cur_colors.background
}

/// Text and icons against background
pub fn on_background() -> Color {
    THEMER.read().unwrap().cur_colors.on_background
}

/// Same color as background. Use surface roles for more neutral backgrounds,
/// and container colors for components like cards, sheets, and dialogs.
pub fn surface() -> Color {
    THEMER.read().unwrap().cur_colors.surface
}

/// Text and icons against surface
pub fn on_surface() -> Color {
    THEMER.read().unwrap().cur_colors.on_surface
}

pub fn surface_variant() -> Color {
    THEMER.read().unwrap().cur_colors.surface_variant
}

pub fn on_surface_variant() -> Color {
    THEMER.read().unwrap().cur_colors.on_surface_variant
}

/// Important boundaries, such as a text field outline
pub fn outline() -> Color {
    THEMER.read().unwrap().cur_colors.outline
}

/// Decorative elements, such as dividers
pub fn outline_variant() -> Color {
    THEMER.read().unwrap().cur_colors.outline_variant
}

/// Shadows for components such as cards, sheets, and menus.
pub fn shadow() -> Color {
    THEMER.read().unwrap().cur_colors.shadow
}

/// Use scrim roles for the color of a scrim, which is a translucent overlay that covers
/// the entire screen and indicates that the UI is temporarily unavailable.
pub fn scrim() -> Color {
    THEMER.read().unwrap().cur_colors.scrim
}

/// High-emphasis fills, texts, and icons against surface
pub fn primary_container() -> Color {
    THEMER.read().unwrap().cur_colors.primary_container
}

/// Text and icons against primary_container
pub fn on_primary_container() -> Color {
    THEMER.read().unwrap().cur_colors.on_primary_container
}

/// Less prominent fill color against surface, for recessive components like tonal buttons
pub fn secondary_container() -> Color {
    THEMER.read().unwrap().cur_colors.secondary_container
}

/// Text and icons against secondary_container
pub fn on_secondary_container() -> Color {
    THEMER.read().unwrap().cur_colors.on_secondary_container
}

/// Complementary container color against surface, for components like input fields
pub fn tertiary_container() -> Color {
    THEMER.read().unwrap().cur_colors.tertiary_container
}

/// Text and icons against tertiary_container
pub fn on_tertiary_container() -> Color {
    THEMER.read().unwrap().cur_colors.on_tertiary_container
}

pub fn colors() -> impl Deref<Target = ParsedSemanticColors> + 'static {
    // doesn't work with a closure for some reason
    fn cur_color_get(g: &Themer) -> &ParsedSemanticColors {
        &g.cur_colors
    }
    let g = THEMER.read().unwrap();
    MappedReadGuard::new(g, cur_color_get)
}

pub struct MappedReadGuard<T: 'static, F> {
    inner: RwLockReadGuard<'static, T>,
    map: F,
}

impl<T, F> MappedReadGuard<T, F> {
    pub fn new(inner: RwLockReadGuard<'static, T>, map: F) -> Self {
        Self { inner, map }
    }
}

impl<T, F, U> Deref for MappedReadGuard<T, F>
where
    F: for<'a> Fn(&'a T) -> &'a U,
{
    type Target = U;

    fn deref(&self) -> &Self::Target {
        (self.map)(&self.inner)
    }
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub enum Theme {
    Light,
    LightMediumContrast,
    LightHighContrast,
    Dark,
    DarkMediumContrast,
    DarkHighContrast,
}

pub fn set_theme(theme: Theme) {
    let mut themer = THEMER.write().unwrap();
    themer.cur_theme = theme;
    themer.cur_colors = match theme {
        Theme::Light => themer.schemes.light.clone(),
        Theme::LightMediumContrast => themer.schemes.light_medium_contrast.clone(),
        Theme::LightHighContrast => themer.schemes.light_high_contrast.clone(),
        Theme::Dark => themer.schemes.dark.clone(),
        Theme::DarkMediumContrast => themer.schemes.dark_medium_contrast.clone(),
        Theme::DarkHighContrast => themer.schemes.dark_high_contrast.clone(),
    };
}

pub fn update_material_colors(json: &str) -> Result<(), DeJsonErr> {
    let root: Root = DeJson::deserialize_json(json)?;
    let cur_theme = THEMER.read().unwrap().cur_theme;
    *THEMER.write().unwrap() = Themer::new(root);
    set_theme(cur_theme);
    Ok(())
}

const DEFAULT_THEME_JSON: &str = include_str!("material-theme.json");

struct Themer {
    cur_colors: ParsedSemanticColors,
    cur_theme: Theme,
    palettes: ParsedPalettes,
    schemes: ParsedSchemes,
}

lazy_static! {
    static ref THEMER: RwLock<Themer> =
        RwLock::new(Themer::new(parse_json(DEFAULT_THEME_JSON).unwrap()));
}

impl Themer {
    fn new(root: Root) -> Self {
        let parsed_palettes: ParsedPalettes = root.palettes.into();
        let parsed_schemes: ParsedSchemes = root.schemes.into();

        Self {
            cur_colors: parsed_schemes.dark.clone(),
            cur_theme: Theme::Dark,
            palettes: parsed_palettes,
            schemes: parsed_schemes,
        }
    }
}

fn parse_json(json: &str) -> Result<Root, DeJsonErr> {
    DeJson::deserialize_json(json)
}

#[derive(Clone)]
/// See https://m3.material.io/styles/color/roles for explanation
pub struct ParsedSemanticColors {
    pub primary: Color,
    pub on_primary: Color,
    pub primary_container: Color,
    pub on_primary_container: Color,

    pub secondary: Color,
    pub on_secondary: Color,
    pub secondary_container: Color,
    pub on_secondary_container: Color,

    pub tertiary: Color,
    pub on_tertiary: Color,
    pub tertiary_container: Color,
    pub on_tertiary_container: Color,

    pub error: Color,
    pub on_error: Color,
    pub error_container: Color,
    pub on_error_container: Color,

    pub background: Color,
    pub on_background: Color,

    pub surface: Color,
    pub surface_tint: Color,
    pub on_surface: Color,
    pub surface_variant: Color,
    pub on_surface_variant: Color,

    pub outline: Color,
    pub outline_variant: Color,

    pub shadow: Color,
    pub scrim: Color,

    // When you need that extra bit of control
    pub surface_dim: Color,
    pub surface_bright: Color,
    pub surface_container_lowest: Color,
    pub surface_container_low: Color,
    pub surface_container: Color,
    pub surface_container_high: Color,
    pub surface_container_highest: Color,

    // Mostly useless but here for completeness
    pub inverse_surface: Color,
    pub inverse_on_surface: Color,
    pub inverse_primary: Color,

    pub primary_fixed: Color,
    pub on_primary_fixed: Color,
    pub primary_fixed_dim: Color,
    pub on_primary_fixed_variant: Color,

    pub secondary_fixed: Color,
    pub on_secondary_fixed: Color,
    pub secondary_fixed_dim: Color,
    pub on_secondary_fixed_variant: Color,

    pub tertiary_fixed: Color,
    pub on_tertiary_fixed: Color,
    pub tertiary_fixed_dim: Color,
    pub on_tertiary_fixed_variant: Color,
}

#[derive(DeJson)]
struct Palettes {
    pub primary: Palette,
    pub secondary: Palette,
    pub tertiary: Palette,
    pub neutral: Palette,
    #[nserde(rename = "neutral-variant")]
    pub neutral_variant: Palette,
}

struct ParsedPalettes {
    pub primary: [Color; 18],
    pub secondary: [Color; 18],
    pub tertiary: [Color; 18],
    pub neutral: [Color; 18],
    pub neutral_variant: [Color; 18],
}

impl From<Palettes> for ParsedPalettes {
    fn from(value: Palettes) -> Self {
        fn parse_palette(palette: Palette) -> [Color; 18] {
            [
                parse_hex(&palette._0),
                parse_hex(&palette._5),
                parse_hex(&palette._10),
                parse_hex(&palette._15),
                parse_hex(&palette._20),
                parse_hex(&palette._25),
                parse_hex(&palette._30),
                parse_hex(&palette._35),
                parse_hex(&palette._40),
                parse_hex(&palette._50),
                parse_hex(&palette._60),
                parse_hex(&palette._70),
                parse_hex(&palette._80),
                parse_hex(&palette._90),
                parse_hex(&palette._95),
                parse_hex(&palette._98),
                parse_hex(&palette._99),
                parse_hex(&palette._100),
            ]
        }

        Self {
            primary: parse_palette(value.primary),
            secondary: parse_palette(value.secondary),
            tertiary: parse_palette(value.tertiary),
            neutral: parse_palette(value.neutral),
            neutral_variant: parse_palette(value.neutral_variant),
        }
    }
}

#[derive(DeJson)]
#[allow(non_snake_case)]
struct SemanticColors {
    pub primary: String,
    pub surfaceTint: String,
    pub onPrimary: String,
    pub primaryContainer: String,
    pub onPrimaryContainer: String,
    pub secondary: String,
    pub onSecondary: String,
    pub secondaryContainer: String,
    pub onSecondaryContainer: String,
    pub tertiary: String,
    pub onTertiary: String,
    pub tertiaryContainer: String,
    pub onTertiaryContainer: String,
    pub error: String,
    pub onError: String,
    pub errorContainer: String,
    pub onErrorContainer: String,
    pub background: String,
    pub onBackground: String,
    pub surface: String,
    pub onSurface: String,
    pub surfaceVariant: String,
    pub onSurfaceVariant: String,
    pub outline: String,
    pub outlineVariant: String,
    pub shadow: String,
    pub scrim: String,
    pub inverseSurface: String,
    pub inverseOnSurface: String,
    pub inversePrimary: String,
    pub primaryFixed: String,
    pub onPrimaryFixed: String,
    pub primaryFixedDim: String,
    pub onPrimaryFixedVariant: String,
    pub secondaryFixed: String,
    pub onSecondaryFixed: String,
    pub secondaryFixedDim: String,
    pub onSecondaryFixedVariant: String,
    pub tertiaryFixed: String,
    pub onTertiaryFixed: String,
    pub tertiaryFixedDim: String,
    pub onTertiaryFixedVariant: String,
    pub surfaceDim: String,
    pub surfaceBright: String,
    pub surfaceContainerLowest: String,
    pub surfaceContainerLow: String,
    pub surfaceContainer: String,
    pub surfaceContainerHigh: String,
    pub surfaceContainerHighest: String,
}

#[derive(DeJson)]
struct Schemes {
    pub light: SemanticColors,
    #[nserde(rename = "light-medium-contrast")]
    pub light_medium_contrast: SemanticColors,
    #[nserde(rename = "light-high-contrast")]
    pub light_high_contrast: SemanticColors,
    pub dark: SemanticColors,
    #[nserde(rename = "dark-medium-contrast")]
    pub dark_medium_contrast: SemanticColors,
    #[nserde(rename = "dark-high-contrast")]
    pub dark_high_contrast: SemanticColors,
}

struct ParsedSchemes {
    pub light: ParsedSemanticColors,
    pub light_medium_contrast: ParsedSemanticColors,
    pub light_high_contrast: ParsedSemanticColors,
    pub dark: ParsedSemanticColors,
    pub dark_medium_contrast: ParsedSemanticColors,
    pub dark_high_contrast: ParsedSemanticColors,
}

impl From<Schemes> for ParsedSchemes {
    fn from(value: Schemes) -> Self {
        Self {
            light: value.light.into(),
            light_medium_contrast: value.light_medium_contrast.into(),
            light_high_contrast: value.light_high_contrast.into(),
            dark: value.dark.into(),
            dark_medium_contrast: value.dark_medium_contrast.into(),
            dark_high_contrast: value.dark_high_contrast.into(),
        }
    }
}

#[derive(DeJson)]
struct Root {
    pub schemes: Schemes,
    pub palettes: Palettes,
}

fn parse_hex(v: &str) -> Color {
    Color::hex(u32::from_str_radix(v.trim_start_matches('#'), 16).unwrap())
}

impl From<SemanticColors> for ParsedSemanticColors {
    fn from(value: SemanticColors) -> Self {
        Self {
            primary: parse_hex(&value.primary),
            surface_tint: parse_hex(&value.surfaceTint),
            on_primary: parse_hex(&value.onPrimary),
            primary_container: parse_hex(&value.primaryContainer),
            on_primary_container: parse_hex(&value.onPrimaryContainer),
            secondary: parse_hex(&value.secondary),
            on_secondary: parse_hex(&value.onSecondary),
            secondary_container: parse_hex(&value.secondaryContainer),
            on_secondary_container: parse_hex(&value.onSecondaryContainer),
            tertiary: parse_hex(&value.tertiary),
            on_tertiary: parse_hex(&value.onTertiary),
            tertiary_container: parse_hex(&value.tertiaryContainer),
            on_tertiary_container: parse_hex(&value.onTertiaryContainer),
            error: parse_hex(&value.error),
            on_error: parse_hex(&value.onError),
            error_container: parse_hex(&value.errorContainer),
            on_error_container: parse_hex(&value.onErrorContainer),
            background: parse_hex(&value.background),
            on_background: parse_hex(&value.onBackground),
            surface: parse_hex(&value.surface),
            on_surface: parse_hex(&value.onSurface),
            surface_variant: parse_hex(&value.surfaceVariant),
            on_surface_variant: parse_hex(&value.onSurfaceVariant),
            outline: parse_hex(&value.outline),
            outline_variant: parse_hex(&value.outlineVariant),
            shadow: parse_hex(&value.shadow),
            scrim: parse_hex(&value.scrim),
            inverse_surface: parse_hex(&value.inverseSurface),
            inverse_on_surface: parse_hex(&value.inverseOnSurface),
            inverse_primary: parse_hex(&value.inversePrimary),
            primary_fixed: parse_hex(&value.primaryFixed),
            on_primary_fixed: parse_hex(&value.onPrimaryFixed),
            primary_fixed_dim: parse_hex(&value.primaryFixedDim),
            on_primary_fixed_variant: parse_hex(&value.onPrimaryFixedVariant),
            secondary_fixed: parse_hex(&value.secondaryFixed),
            on_secondary_fixed: parse_hex(&value.onSecondaryFixed),
            secondary_fixed_dim: parse_hex(&value.secondaryFixedDim),
            on_secondary_fixed_variant: parse_hex(&value.onSecondaryFixedVariant),
            tertiary_fixed: parse_hex(&value.tertiaryFixed),
            on_tertiary_fixed: parse_hex(&value.onTertiaryFixed),
            tertiary_fixed_dim: parse_hex(&value.tertiaryFixedDim),
            on_tertiary_fixed_variant: parse_hex(&value.onTertiaryFixedVariant),
            surface_dim: parse_hex(&value.surfaceDim),
            surface_bright: parse_hex(&value.surfaceBright),
            surface_container_lowest: parse_hex(&value.surfaceContainerLowest),
            surface_container_low: parse_hex(&value.surfaceContainerLow),
            surface_container: parse_hex(&value.surfaceContainer),
            surface_container_high: parse_hex(&value.surfaceContainerHigh),
            surface_container_highest: parse_hex(&value.surfaceContainerHighest),
        }
    }
}

#[derive(DeJson)]
struct Palette {
    #[nserde(rename = "0")]
    pub _0: String,

    #[nserde(rename = "5")]
    pub _5: String,

    #[nserde(rename = "10")]
    pub _10: String,

    #[nserde(rename = "15")]
    pub _15: String,

    #[nserde(rename = "20")]
    pub _20: String,

    #[nserde(rename = "25")]
    pub _25: String,

    #[nserde(rename = "30")]
    pub _30: String,

    #[nserde(rename = "35")]
    pub _35: String,

    #[nserde(rename = "40")]
    pub _40: String,

    #[nserde(rename = "50")]
    pub _50: String,

    #[nserde(rename = "60")]
    pub _60: String,

    #[nserde(rename = "70")]
    pub _70: String,

    #[nserde(rename = "80")]
    pub _80: String,

    #[nserde(rename = "90")]
    pub _90: String,

    #[nserde(rename = "95")]
    pub _95: String,

    #[nserde(rename = "98")]
    pub _98: String,

    #[nserde(rename = "99")]
    pub _99: String,

    #[nserde(rename = "100")]
    pub _100: String,
}
