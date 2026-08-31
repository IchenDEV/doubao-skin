//! AST-backed validation for the deliberately narrow v3 theme CSS subset.

use std::collections::BTreeSet;
use std::convert::Infallible;
use std::fs;
use std::path::Path;

use lightningcss::declaration::DeclarationBlock;
use lightningcss::rules::{CssRule, CssRuleList, Location};
use lightningcss::selector::{Component, Selector};
use lightningcss::stylesheet::{ParserOptions, PrinterOptions, StyleSheet};
use lightningcss::traits::ToCss;
use lightningcss::values::url::Url;
use lightningcss::visit_types;
use lightningcss::visitor::{Visit, VisitTypes, Visitor};
use parcel_selectors::attr::{AttrSelectorOperator, ParsedCaseSensitivity};

use crate::theme_package::{ThemePackageError, ThemeTarget};

#[derive(Debug, Clone)]
pub(crate) enum CssFileScope {
    Shared,
    Targets(BTreeSet<ThemeTarget>),
}

pub(crate) fn validate_css_file(
    path: &Path,
    theme_id: &str,
    scope: &CssFileScope,
) -> Result<(), ThemePackageError> {
    let css = fs::read_to_string(path).map_err(|error| {
        ThemePackageError::from_css(
            format!("cannot read CSS as UTF-8: {error}"),
            path,
            None,
            None,
        )
    })?;
    let mut stylesheet = StyleSheet::parse(
        &css,
        ParserOptions {
            filename: path.to_string_lossy().into_owned(),
            error_recovery: false,
            ..ParserOptions::default()
        },
    )
    .map_err(|error| {
        let (line, column) = error
            .loc
            .as_ref()
            .map(|location| (Some(location.line + 1), Some(location.column)))
            .unwrap_or((None, None));
        ThemePackageError::from_css(
            format!("invalid CSS syntax: {}", error.kind),
            path,
            line,
            column,
        )
    })?;

    let mut urls = UrlDetector::default();
    stylesheet
        .visit(&mut urls)
        .map_err(|never| match never {})?;
    if urls.found {
        return Err(ThemePackageError::from_css(
            "url() is not allowed in v3 theme CSS; declare package assets in theme.json",
            path,
            None,
            None,
        ));
    }

    validate_rule_list(path, theme_id, scope, &stylesheet.rules)
}

#[derive(Default)]
struct UrlDetector {
    found: bool,
}

impl<'i> Visitor<'i> for UrlDetector {
    type Error = Infallible;

    fn visit_types(&self) -> VisitTypes {
        visit_types!(URLS)
    }

    fn visit_url(&mut self, _url: &mut Url<'i>) -> Result<(), Self::Error> {
        self.found = true;
        Ok(())
    }
}

fn validate_rule_list(
    path: &Path,
    theme_id: &str,
    scope: &CssFileScope,
    rules: &CssRuleList<'_>,
) -> Result<(), ThemePackageError> {
    for rule in &rules.0 {
        match rule {
            CssRule::Style(style) => {
                for selector in &style.selectors.0 {
                    validate_selector(path, theme_id, scope, selector, style.loc)?;
                }
                validate_declarations(path, &style.declarations, style.loc)?;
                if !style.rules.0.is_empty() {
                    return Err(css_error_at(
                        path,
                        style.loc,
                        "CSS nesting is not allowed in v3 theme CSS",
                    ));
                }
            }
            CssRule::Media(media) => {
                validate_rule_list(path, theme_id, scope, &media.rules)?;
            }
            CssRule::Ignored => {}
            CssRule::Import(rule) => {
                return Err(css_error_at(
                    path,
                    rule.loc,
                    "@import is not allowed in v3 theme CSS",
                ));
            }
            CssRule::FontFace(rule) => {
                return Err(css_error_at(
                    path,
                    rule.loc,
                    "@font-face is not allowed in v3 theme CSS",
                ));
            }
            CssRule::Keyframes(rule) => {
                return Err(css_error_at(
                    path,
                    rule.loc,
                    "@keyframes is not allowed in v3 theme CSS",
                ));
            }
            other => {
                let (name, location) = disallowed_rule(other);
                return Err(css_error_at(
                    path,
                    location,
                    format!("{name} is not allowed in v3 theme CSS"),
                ));
            }
        }
    }
    Ok(())
}

fn disallowed_rule(rule: &CssRule<'_>) -> (&'static str, Location) {
    match rule {
        CssRule::Supports(rule) => ("@supports", rule.loc),
        CssRule::FontPaletteValues(rule) => ("@font-palette-values", rule.loc),
        CssRule::FontFeatureValues(rule) => ("@font-feature-values", rule.loc),
        CssRule::Page(rule) => ("@page", rule.loc),
        CssRule::CounterStyle(rule) => ("@counter-style", rule.loc),
        CssRule::Namespace(rule) => ("@namespace", rule.loc),
        CssRule::MozDocument(rule) => ("@-moz-document", rule.loc),
        CssRule::Nesting(rule) => ("@nest", rule.loc),
        CssRule::NestedDeclarations(rule) => ("nested declarations", rule.loc),
        CssRule::Viewport(rule) => ("@viewport", rule.loc),
        CssRule::CustomMedia(rule) => ("@custom-media", rule.loc),
        CssRule::LayerStatement(rule) => ("@layer", rule.loc),
        CssRule::LayerBlock(rule) => ("@layer", rule.loc),
        CssRule::Property(rule) => ("@property", rule.loc),
        CssRule::Container(rule) => ("@container", rule.loc),
        CssRule::Scope(rule) => ("@scope", rule.loc),
        CssRule::StartingStyle(rule) => ("@starting-style", rule.loc),
        CssRule::ViewTransition(rule) => ("@view-transition", rule.loc),
        CssRule::PositionTry(rule) => ("@position-try", rule.loc),
        CssRule::Unknown(rule) => ("unknown at-rule", rule.loc),
        CssRule::Custom(_) => (
            "custom at-rule",
            Location {
                source_index: 0,
                line: 0,
                column: 1,
            },
        ),
        CssRule::Media(rule) => ("@media", rule.loc),
        CssRule::Import(rule) => ("@import", rule.loc),
        CssRule::Style(rule) => ("style rule", rule.loc),
        CssRule::Keyframes(rule) => ("@keyframes", rule.loc),
        CssRule::FontFace(rule) => ("@font-face", rule.loc),
        CssRule::Ignored => (
            "ignored rule",
            Location {
                source_index: 0,
                line: 0,
                column: 1,
            },
        ),
    }
}

fn validate_selector(
    path: &Path,
    theme_id: &str,
    scope: &CssFileScope,
    selector: &Selector<'_>,
    location: Location,
) -> Result<(), ThemePackageError> {
    let serialized = selector
        .to_css_string(PrinterOptions::default())
        .map_err(|error| {
            css_error_at(
                path,
                location,
                format!("cannot serialize selector: {error}"),
            )
        })?;
    let Some(root_target) = direct_root_scope(selector, theme_id) else {
        return Err(css_error_at(
            path,
            location,
            format!("selector is not rooted at html[data-skin=\"{theme_id}\"]: {serialized}"),
        ));
    };

    match scope {
        CssFileScope::Shared => {
            if root_target.is_some() || serialized.contains("data-skin-target") {
                return Err(css_error_at(
                    path,
                    location,
                    format!("shared selector cannot contain data-skin-target: {serialized}"),
                ));
            }
        }
        CssFileScope::Targets(allowed) => {
            let Some(target) = root_target else {
                return Err(css_error_at(
                    path,
                    location,
                    format!("target selector is missing data-skin-target: {serialized}"),
                ));
            };
            if !allowed.contains(&target) {
                return Err(css_error_at(
                    path,
                    location,
                    format!(
                        "selector targets {target}, but this file is not referenced by that target"
                    ),
                ));
            }
            if serialized.matches("data-skin-target").count() != 1 {
                return Err(css_error_at(
                    path,
                    location,
                    format!("target selector must contain exactly one direct target scope: {serialized}"),
                ));
            }
        }
    }
    Ok(())
}

fn direct_root_scope(selector: &Selector<'_>, theme_id: &str) -> Option<Option<ThemeTarget>> {
    let mut is_html = false;
    let mut has_skin = false;
    let mut target = None;

    let finish_compound = |is_html: bool, has_skin: bool, target: Option<ThemeTarget>| {
        (is_html && has_skin).then_some(target)
    };

    for component in selector.iter_raw_match_order() {
        match component {
            Component::Combinator(_) => {
                if let Some(result) = finish_compound(is_html, has_skin, target) {
                    return Some(result);
                }
                is_html = false;
                has_skin = false;
                target = None;
            }
            Component::LocalName(name) if name.lower_name.as_ref() == "html" => is_html = true,
            Component::AttributeInNoNamespace {
                local_name,
                operator,
                value,
                case_sensitivity,
                ..
            } if attribute_is_exact(*operator, *case_sensitivity)
                && local_name.as_ref() == "data-skin" =>
            {
                has_skin = value.as_ref() == theme_id;
            }
            Component::AttributeInNoNamespace {
                local_name,
                operator,
                value,
                case_sensitivity,
                ..
            } if attribute_is_exact(*operator, *case_sensitivity)
                && local_name.as_ref() == "data-skin-target" =>
            {
                target = ThemeTarget::parse(value.as_ref());
            }
            _ => {}
        }
    }
    finish_compound(is_html, has_skin, target)
}

fn attribute_is_exact(
    operator: AttrSelectorOperator,
    case_sensitivity: ParsedCaseSensitivity,
) -> bool {
    operator == AttrSelectorOperator::Equal
        && matches!(
            case_sensitivity,
            ParsedCaseSensitivity::CaseSensitive | ParsedCaseSensitivity::ExplicitCaseSensitive
        )
}

fn validate_declarations(
    path: &Path,
    declarations: &DeclarationBlock<'_>,
    location: Location,
) -> Result<(), ThemePackageError> {
    for property in declarations
        .declarations
        .iter()
        .chain(&declarations.important_declarations)
    {
        let name = property.property_id().name().to_ascii_lowercase();
        if name.starts_with("--") {
            if name.starts_with("--doubao-skin-runtime-") {
                return Err(css_error_at(
                    path,
                    location,
                    format!("reserved runtime custom property cannot be overridden: {name}"),
                ));
            }
            continue;
        }
        if !is_allowed_visual_property(&name) {
            return Err(css_error_at(
                path,
                location,
                format!("property is outside the v3 visual whitelist: {name}"),
            ));
        }
    }
    Ok(())
}

fn is_allowed_visual_property(name: &str) -> bool {
    matches!(
        name,
        "color"
            | "color-scheme"
            | "background"
            | "background-color"
            | "background-image"
            | "background-blend-mode"
            | "box-shadow"
            | "text-shadow"
            | "opacity"
            | "filter"
            | "backdrop-filter"
            | "-webkit-backdrop-filter"
            | "caret-color"
            | "fill"
            | "fill-opacity"
            | "fill-rule"
            | "stroke"
            | "stroke-color"
            | "stroke-opacity"
            | "stroke-width"
            | "stroke-linecap"
            | "stroke-linejoin"
            | "scrollbar-color"
            | "scrollbar-width"
            | "accent-color"
            | "mix-blend-mode"
    ) || name.starts_with("border")
        || name.starts_with("outline")
        || name.starts_with("font")
        || name.starts_with("text")
        || name.starts_with("transition")
        || name.starts_with("-webkit-text-")
}

fn css_error_at(path: &Path, location: Location, message: impl Into<String>) -> ThemePackageError {
    ThemePackageError::from_css(
        message,
        path,
        Some(location.line + 1),
        Some(location.column),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn whitelist_excludes_layout_interaction_and_content() {
        for property in [
            "display",
            "position",
            "width",
            "grid",
            "flex",
            "pointer-events",
            "content",
            "z-index",
        ] {
            assert!(
                !is_allowed_visual_property(property),
                "{property} must stay blocked"
            );
        }
        for property in [
            "color",
            "background-color",
            "border-color",
            "box-shadow",
            "font-weight",
            "text-decoration",
            "transition",
        ] {
            assert!(
                is_allowed_visual_property(property),
                "{property} should be visual-only"
            );
        }
    }
}
