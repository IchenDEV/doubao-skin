//! Versioned theme-package validation and target-specific resolution.
//!
//! Callers deliberately do not see the v1/v2/v3 manifest shapes. They ask
//! whether a validated package supports a host, then resolve one appearance.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::fs;
use std::io::Read;
use std::path::{Component, Path, PathBuf};
use std::sync::OnceLock;

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::theme_css::{self, CssFileScope};

const CURRENT_SCHEMA_VERSION: u32 = 3;
const MAX_MANIFEST_BYTES: u64 = 1024 * 1024;
const MAX_CSS_FILE_BYTES: u64 = 512 * 1024;
const MAX_TARGET_CSS_BYTES: u64 = 2 * 1024 * 1024;
const MAX_SVG_BYTES: u64 = 2 * 1024 * 1024;
const MAX_RESOURCE_BYTES: u64 = 200 * 1024 * 1024;
const V3_SCHEMA: &str = include_str!("../../../design/theme-standard/theme-v3.schema.json");

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ThemeTarget {
    Doubao,
    DoubaoWork,
    WorkBuddy,
}

impl ThemeTarget {
    pub const ALL: [Self; 3] = [Self::Doubao, Self::DoubaoWork, Self::WorkBuddy];

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Doubao => "doubao",
            Self::DoubaoWork => "doubao-work",
            Self::WorkBuddy => "workbuddy",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "doubao" => Some(Self::Doubao),
            "doubao-work" => Some(Self::DoubaoWork),
            "workbuddy" => Some(Self::WorkBuddy),
            _ => None,
        }
    }
}

impl fmt::Display for ThemeTarget {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ThemePackageAppearance {
    Light,
    Dark,
}

impl ThemePackageAppearance {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Light => "light",
            Self::Dark => "dark",
        }
    }
}

impl fmt::Display for ThemePackageAppearance {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SupportLevel {
    Unsupported,
    Shared,
    Tailored,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SupportDeclaration {
    Explicit,
    LegacyInferred,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TargetSupport {
    pub level: SupportLevel,
    pub declaration: SupportDeclaration,
}

impl TargetSupport {
    pub const fn is_supported(self) -> bool {
        !matches!(self.level, SupportLevel::Unsupported)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ThemePackageErrorCategory {
    Io,
    Json,
    Manifest,
    UnsupportedSchema,
    UnsupportedTarget,
    UnsupportedAppearance,
    Path,
    MissingResource,
    Resource,
    Semantic,
    Css,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThemePackageError {
    pub category: ThemePackageErrorCategory,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pointer: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<ThemeTarget>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub appearance: Option<ThemePackageAppearance>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub column: Option<u32>,
}

impl ThemePackageError {
    fn new(category: ThemePackageErrorCategory, message: impl Into<String>) -> Self {
        Self {
            category,
            message: message.into(),
            pointer: None,
            target: None,
            appearance: None,
            file: None,
            line: None,
            column: None,
        }
    }

    fn at_pointer(mut self, pointer: impl Into<String>) -> Self {
        self.pointer = Some(pointer.into());
        self
    }

    fn for_resolution(mut self, target: ThemeTarget, appearance: ThemePackageAppearance) -> Self {
        self.target = Some(target);
        self.appearance = Some(appearance);
        self
    }

    pub(crate) fn from_css(
        message: impl Into<String>,
        file: &Path,
        line: Option<u32>,
        column: Option<u32>,
    ) -> Self {
        Self {
            category: ThemePackageErrorCategory::Css,
            message: message.into(),
            pointer: None,
            target: None,
            appearance: None,
            file: Some(file.to_string_lossy().into_owned()),
            line,
            column,
        }
    }
}

impl fmt::Display for ThemePackageError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.message)?;
        if let Some(pointer) = &self.pointer {
            write!(formatter, " ({pointer})")?;
        }
        if let Some(target) = self.target {
            write!(formatter, " [{target}")?;
            if let Some(appearance) = self.appearance {
                write!(formatter, "/{appearance}")?;
            }
            write!(formatter, "]")?;
        }
        if let Some(file) = &self.file {
            write!(formatter, " in {file}")?;
            if let Some(line) = self.line {
                write!(formatter, ":{line}")?;
                if let Some(column) = self.column {
                    write!(formatter, ":{column}")?;
                }
            }
        }
        Ok(())
    }
}

impl std::error::Error for ThemePackageError {}

#[derive(Debug, Clone)]
pub struct ResolvedTheme {
    pub schema_version: u32,
    pub id: String,
    pub name: String,
    pub description: String,
    pub version: String,
    pub author: String,
    pub target: ThemeTarget,
    pub appearance: ThemePackageAppearance,
    pub support: TargetSupport,
    pub preview: Option<PathBuf>,
    pub css_files: Vec<PathBuf>,
    pub resource_files: Vec<PathBuf>,
    pub(crate) visual: Value,
}

impl ResolvedTheme {
    pub fn visual(&self) -> &Value {
        &self.visual
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TargetValidationReport {
    pub support_level: SupportLevel,
    pub declaration: SupportDeclaration,
    pub appearances: Vec<ThemePackageAppearance>,
    pub css: BTreeMap<ThemePackageAppearance, Vec<String>>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidationReport {
    pub schema_version: u32,
    pub id: String,
    pub targets: BTreeMap<String, TargetValidationReport>,
    pub resources: Vec<String>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ValidatedThemePackage {
    root: PathBuf,
    manifest: Value,
    schema_version: u32,
    id: String,
    name: String,
    description: String,
    version: String,
    author: String,
    resources: BTreeSet<String>,
    warnings: Vec<String>,
}

impl ValidatedThemePackage {
    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn schema_version(&self) -> u32 {
        self.schema_version
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn version(&self) -> &str {
        &self.version
    }

    pub fn author(&self) -> &str {
        &self.author
    }

    pub(crate) fn manifest(&self) -> &Value {
        &self.manifest
    }

    pub fn support(&self, target: ThemeTarget) -> TargetSupport {
        if self.schema_version < 3 {
            return match (self.schema_version, target) {
                (1, ThemeTarget::WorkBuddy) => TargetSupport {
                    level: SupportLevel::Unsupported,
                    declaration: SupportDeclaration::LegacyInferred,
                },
                (2, ThemeTarget::WorkBuddy) => TargetSupport {
                    level: SupportLevel::Shared,
                    declaration: SupportDeclaration::LegacyInferred,
                },
                _ => TargetSupport {
                    level: SupportLevel::Tailored,
                    declaration: SupportDeclaration::LegacyInferred,
                },
            };
        }

        let Some(target_layer) = self.manifest["targets"].get(target.as_str()) else {
            return TargetSupport {
                level: SupportLevel::Unsupported,
                declaration: SupportDeclaration::Explicit,
            };
        };
        let tailored = target_has_substantive_delta(&self.manifest["shared"], target_layer);
        TargetSupport {
            level: if tailored {
                SupportLevel::Tailored
            } else {
                SupportLevel::Shared
            },
            declaration: SupportDeclaration::Explicit,
        }
    }

    pub fn appearances(&self, target: ThemeTarget) -> Vec<ThemePackageAppearance> {
        if !self.support(target).is_supported() {
            return Vec::new();
        }
        let appearance = if self.schema_version == 3 {
            self.manifest["targets"][target.as_str()]
                .get("appearance")
                .and_then(Value::as_str)
                .or_else(|| self.manifest["shared"]["appearance"].as_str())
                .unwrap_or("dark-only")
        } else {
            self.manifest
                .get("appearance")
                .and_then(Value::as_str)
                .unwrap_or_else(|| match self.manifest.get("mode").and_then(Value::as_str) {
                    Some("light") => "light-only",
                    Some("auto") => "both",
                    _ => "dark-only",
                })
        };
        match appearance {
            "light-only" => vec![ThemePackageAppearance::Light],
            "both" => vec![ThemePackageAppearance::Light, ThemePackageAppearance::Dark],
            _ => vec![ThemePackageAppearance::Dark],
        }
    }

    pub fn resolve(
        &self,
        target: ThemeTarget,
        appearance: ThemePackageAppearance,
    ) -> Result<ResolvedTheme, ThemePackageError> {
        let support = self.support(target);
        if !support.is_supported() {
            return Err(ThemePackageError::new(
                ThemePackageErrorCategory::UnsupportedTarget,
                format!("theme {} does not support {target}", self.id),
            )
            .for_resolution(target, appearance));
        }
        if !self.appearances(target).contains(&appearance) {
            return Err(ThemePackageError::new(
                ThemePackageErrorCategory::UnsupportedAppearance,
                format!(
                    "theme {} does not support {appearance} on {target}",
                    self.id
                ),
            )
            .for_resolution(target, appearance));
        }

        if self.schema_version < 3 {
            return self.resolve_legacy(target, appearance, support);
        }

        let shared = &self.manifest["shared"];
        let target_layer = &self.manifest["targets"][target.as_str()];
        let mut visual = visual_fields(shared);
        if let Some(variant) = appearance_variant(shared, appearance) {
            merge_value(&mut visual, visual_fields(variant));
        }
        merge_value(&mut visual, visual_fields(target_layer));
        if let Some(variant) = appearance_variant(target_layer, appearance) {
            merge_value(&mut visual, visual_fields(variant));
        }
        validate_minimum_semantics(&visual)
            .map_err(|error| error.for_resolution(target, appearance))?;

        let css_relative = effective_css_paths(shared, target_layer, appearance)?;
        let mut seen = BTreeSet::new();
        let mut css_files = Vec::with_capacity(css_relative.len());
        let mut css_bytes = 0_u64;
        for relative in css_relative {
            if !seen.insert(relative.clone()) {
                return Err(ThemePackageError::new(
                    ThemePackageErrorCategory::Css,
                    format!("CSS file is repeated in the effective load chain: {relative}"),
                )
                .for_resolution(target, appearance));
            }
            let path = resolve_package_file(&self.root, &relative, "CSS")?;
            let size = fs::metadata(&path)
                .map_err(|error| io_error(&path, error))?
                .len();
            if size > MAX_CSS_FILE_BYTES {
                return Err(ThemePackageError::new(
                    ThemePackageErrorCategory::Css,
                    format!("CSS file exceeds 512 KiB: {relative}"),
                )
                .for_resolution(target, appearance));
            }
            css_bytes = css_bytes.saturating_add(size);
            css_files.push(path);
        }
        if css_bytes > MAX_TARGET_CSS_BYTES {
            return Err(ThemePackageError::new(
                ThemePackageErrorCategory::Css,
                format!("resolved CSS exceeds 2 MiB for {target}/{appearance}"),
            )
            .for_resolution(target, appearance));
        }

        let preview_relative = target_layer
            .get("preview")
            .and_then(|value| value.get("image"))
            .and_then(Value::as_str)
            .or_else(|| self.manifest["preview"]["image"].as_str());
        let preview = preview_relative
            .map(|relative| resolve_package_file(&self.root, relative, "preview"))
            .transpose()?;
        if let Some(preview) = &preview {
            validate_resource_file(preview, ResourceKind::Image)?;
        }
        validate_visual_resource_types(&self.root, &visual)?;
        let mut resource_files = collect_visual_resource_paths(&visual)
            .into_iter()
            .map(|relative| resolve_package_file(&self.root, &relative, "resource"))
            .collect::<Result<Vec<_>, _>>()?;
        resource_files.extend(css_files.iter().cloned());
        if let Some(preview) = &preview {
            resource_files.push(preview.clone());
        }
        resource_files.sort();
        resource_files.dedup();

        Ok(ResolvedTheme {
            schema_version: self.schema_version,
            id: self.id.clone(),
            name: self.name.clone(),
            description: self.description.clone(),
            version: self.version.clone(),
            author: self.author.clone(),
            target,
            appearance,
            support,
            preview,
            css_files,
            resource_files,
            visual,
        })
    }

    fn resolve_legacy(
        &self,
        target: ThemeTarget,
        appearance: ThemePackageAppearance,
        support: TargetSupport,
    ) -> Result<ResolvedTheme, ThemePackageError> {
        let css_files = if target == ThemeTarget::WorkBuddy {
            Vec::new()
        } else {
            vec![resolve_package_file(&self.root, "theme.css", "legacy CSS")?]
        };
        let preview = self
            .manifest
            .get("preview")
            .and_then(|value| value.get("image"))
            .and_then(Value::as_str)
            .map(|relative| resolve_package_file(&self.root, relative, "preview"))
            .transpose()?;
        Ok(ResolvedTheme {
            schema_version: self.schema_version,
            id: self.id.clone(),
            name: self.name.clone(),
            description: self.description.clone(),
            version: self.version.clone(),
            author: self.author.clone(),
            target,
            appearance,
            support,
            preview,
            css_files,
            resource_files: Vec::new(),
            visual: self.manifest.clone(),
        })
    }

    pub fn report(&self) -> Result<ValidationReport, ThemePackageError> {
        let mut targets = BTreeMap::new();
        for target in ThemeTarget::ALL {
            let support = self.support(target);
            let appearances = self.appearances(target);
            let mut css = BTreeMap::new();
            for appearance in appearances.iter().copied() {
                let resolved = self.resolve(target, appearance)?;
                css.insert(
                    appearance,
                    resolved
                        .css_files
                        .iter()
                        .map(|path| package_relative(&self.root, path))
                        .collect(),
                );
            }
            targets.insert(
                target.as_str().to_string(),
                TargetValidationReport {
                    support_level: support.level,
                    declaration: support.declaration,
                    appearances,
                    css,
                },
            );
        }
        Ok(ValidationReport {
            schema_version: self.schema_version,
            id: self.id.clone(),
            targets,
            resources: self.resources.iter().cloned().collect(),
            warnings: self.warnings.clone(),
        })
    }
}

pub fn validate_theme_package(
    theme_dir: &Path,
) -> Result<ValidatedThemePackage, ThemePackageError> {
    let root = theme_dir
        .canonicalize()
        .map_err(|error| io_error(theme_dir, error))?;
    if !root.is_dir() {
        return Err(ThemePackageError::new(
            ThemePackageErrorCategory::Io,
            format!("theme path is not a directory: {}", root.display()),
        ));
    }
    let manifest_path = root.join("theme.json");
    let metadata = fs::metadata(&manifest_path).map_err(|error| io_error(&manifest_path, error))?;
    if metadata.len() > MAX_MANIFEST_BYTES {
        return Err(ThemePackageError::new(
            ThemePackageErrorCategory::Manifest,
            "theme.json exceeds 1 MiB",
        ));
    }
    let manifest_text =
        fs::read_to_string(&manifest_path).map_err(|error| io_error(&manifest_path, error))?;
    let manifest: Value =
        serde_json::from_str(&manifest_text).map_err(|error| ThemePackageError {
            category: ThemePackageErrorCategory::Json,
            message: format!("invalid theme.json: {error}"),
            pointer: None,
            target: None,
            appearance: None,
            file: Some("theme.json".into()),
            line: Some(error.line() as u32),
            column: Some(error.column() as u32),
        })?;
    let schema_version = manifest
        .get("schemaVersion")
        .and_then(Value::as_u64)
        .unwrap_or(1) as u32;
    if schema_version > CURRENT_SCHEMA_VERSION {
        return Err(ThemePackageError::new(
            ThemePackageErrorCategory::UnsupportedSchema,
            format!(
                "theme schema v{schema_version} is newer than supported v{CURRENT_SCHEMA_VERSION}"
            ),
        )
        .at_pointer("/schemaVersion"));
    }
    if schema_version == 3 {
        validate_v3_schema(&manifest)?;
    }
    let id = required_string(&manifest, "id")?.to_string();
    if root.file_name().and_then(|value| value.to_str()) != Some(id.as_str()) {
        return Err(ThemePackageError::new(
            ThemePackageErrorCategory::Manifest,
            format!("theme id must match its directory name: {id}"),
        )
        .at_pointer("/id"));
    }
    let name = manifest
        .get("name")
        .and_then(Value::as_str)
        .unwrap_or(&id)
        .to_string();
    let description = manifest
        .get("description")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    let version = manifest
        .get("version")
        .and_then(Value::as_str)
        .unwrap_or("1.0.0")
        .to_string();
    let author = manifest
        .get("author")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();

    if schema_version < 3 {
        resolve_package_file(&root, "theme.css", "legacy CSS")?;
    }

    let resources = if schema_version == 3 {
        collect_manifest_resources(&manifest)
    } else {
        BTreeSet::new()
    };
    for relative in &resources {
        resolve_package_file(&root, relative, "resource")?;
    }
    if schema_version == 3 {
        validate_declared_previews(&root, &manifest)?;
    }

    let mut package = ValidatedThemePackage {
        root,
        manifest,
        schema_version,
        id,
        name,
        description,
        version,
        author,
        resources,
        warnings: Vec::new(),
    };

    if schema_version == 3 {
        validate_all_v3_resolutions(&package)?;
        validate_v3_css(&package)?;
        for target in ThemeTarget::ALL {
            let layer = package.manifest["targets"].get(target.as_str());
            if layer.is_some_and(|value| {
                value.as_object().is_some_and(|object| !object.is_empty())
                    && package.support(target).level == SupportLevel::Shared
            }) {
                package.warnings.push(format!(
                    "targets.{} repeats shared values without changing the resolved theme",
                    target.as_str()
                ));
            }
        }
    }
    Ok(package)
}

fn v3_validator() -> &'static jsonschema::Validator {
    static VALIDATOR: OnceLock<jsonschema::Validator> = OnceLock::new();
    VALIDATOR.get_or_init(|| {
        let schema: Value =
            serde_json::from_str(V3_SCHEMA).expect("embedded theme v3 schema is JSON");
        jsonschema::draft202012::new(&schema).expect("embedded theme v3 schema compiles")
    })
}

fn validate_v3_schema(manifest: &Value) -> Result<(), ThemePackageError> {
    if let Some(error) = v3_validator().iter_errors(manifest).next() {
        let pointer = error.instance_path().to_string();
        return Err(ThemePackageError::new(
            ThemePackageErrorCategory::Manifest,
            format!("theme.json does not satisfy schema v3: {error}"),
        )
        .at_pointer(if pointer.is_empty() {
            "/".into()
        } else {
            pointer
        }));
    }
    Ok(())
}

fn validate_all_v3_resolutions(package: &ValidatedThemePackage) -> Result<(), ThemePackageError> {
    for target in ThemeTarget::ALL {
        for appearance in package.appearances(target) {
            let resolved = package.resolve(target, appearance)?;
            if target == ThemeTarget::WorkBuddy
                && resolved
                    .visual()
                    .get("icons")
                    .and_then(Value::as_object)
                    .is_some_and(|icons| !icons.is_empty())
            {
                return Err(ThemePackageError::new(
                    ThemePackageErrorCategory::Manifest,
                    "WorkBuddy does not support theme icon replacement; set targets.workbuddy.icons to null",
                )
                .at_pointer("/targets/workbuddy/icons")
                .for_resolution(target, appearance));
            }
        }
    }
    Ok(())
}

#[derive(Default)]
struct CssReference {
    shared: bool,
    targets: BTreeSet<ThemeTarget>,
}

fn validate_v3_css(package: &ValidatedThemePackage) -> Result<(), ThemePackageError> {
    let mut references: BTreeMap<String, CssReference> = BTreeMap::new();
    collect_shared_css_references(&package.manifest["shared"], &mut references)?;
    for target in ThemeTarget::ALL {
        if let Some(layer) = package.manifest["targets"].get(target.as_str()) {
            collect_target_css_references(layer, target, &mut references)?;
        }
    }
    for (relative, reference) in references {
        if reference.shared && !reference.targets.is_empty() {
            return Err(ThemePackageError::new(
                ThemePackageErrorCategory::Css,
                format!("CSS file cannot be both shared and target-scoped: {relative}"),
            ));
        }
        let path = resolve_package_file(&package.root, &relative, "CSS")?;
        let scope = if reference.shared {
            CssFileScope::Shared
        } else {
            CssFileScope::Targets(reference.targets)
        };
        theme_css::validate_css_file(&path, &package.id, &scope)?;
    }
    Ok(())
}

fn collect_shared_css_references(
    layer: &Value,
    references: &mut BTreeMap<String, CssReference>,
) -> Result<(), ThemePackageError> {
    for relative in css_paths(layer)? {
        references.entry(relative).or_default().shared = true;
    }
    for appearance in [ThemePackageAppearance::Light, ThemePackageAppearance::Dark] {
        if let Some(variant) = appearance_variant(layer, appearance) {
            for relative in css_paths(variant)? {
                references.entry(relative).or_default().shared = true;
            }
        }
    }
    Ok(())
}

fn collect_target_css_references(
    layer: &Value,
    target: ThemeTarget,
    references: &mut BTreeMap<String, CssReference>,
) -> Result<(), ThemePackageError> {
    for relative in css_paths(layer)? {
        references
            .entry(relative)
            .or_default()
            .targets
            .insert(target);
    }
    for appearance in [ThemePackageAppearance::Light, ThemePackageAppearance::Dark] {
        if let Some(variant) = appearance_variant(layer, appearance) {
            for relative in css_paths(variant)? {
                references
                    .entry(relative)
                    .or_default()
                    .targets
                    .insert(target);
            }
        }
    }
    Ok(())
}

fn effective_css_paths(
    shared: &Value,
    target: &Value,
    appearance: ThemePackageAppearance,
) -> Result<Vec<String>, ThemePackageError> {
    let mut paths = css_paths(shared)?;
    if let Some(variant) = appearance_variant(shared, appearance) {
        paths.extend(css_paths(variant)?);
    }
    paths.extend(css_paths(target)?);
    if let Some(variant) = appearance_variant(target, appearance) {
        paths.extend(css_paths(variant)?);
    }
    Ok(paths)
}

fn css_paths(layer: &Value) -> Result<Vec<String>, ThemePackageError> {
    let Some(css) = layer.get("css") else {
        return Ok(Vec::new());
    };
    let array = css.as_array().ok_or_else(|| {
        ThemePackageError::new(ThemePackageErrorCategory::Manifest, "css must be an array")
    })?;
    Ok(array
        .iter()
        .filter_map(Value::as_str)
        .map(str::to_string)
        .collect())
}

fn appearance_variant(layer: &Value, appearance: ThemePackageAppearance) -> Option<&Value> {
    layer
        .get("variants")
        .and_then(|variants| variants.get(appearance.as_str()))
}

fn visual_fields(layer: &Value) -> Value {
    const CONTROL_FIELDS: [&str; 4] = ["appearance", "css", "variants", "preview"];
    let mut visual = Map::new();
    if let Some(object) = layer.as_object() {
        for (key, value) in object {
            if !CONTROL_FIELDS.contains(&key.as_str()) {
                visual.insert(key.clone(), value.clone());
            }
        }
    }
    Value::Object(visual)
}

fn merge_value(base: &mut Value, overlay: Value) {
    match (base, overlay) {
        (Value::Object(base), Value::Object(overlay)) => {
            for (key, value) in overlay {
                if value.is_null() {
                    base.remove(&key);
                } else if let Some(current) = base.get_mut(&key) {
                    merge_value(current, value);
                } else {
                    base.insert(key, value);
                }
            }
        }
        (base, overlay) => *base = overlay,
    }
}

fn target_has_substantive_delta(shared: &Value, target: &Value) -> bool {
    if target.get("preview").is_some() || !css_paths(target).unwrap_or_default().is_empty() {
        return true;
    }
    if target.get("appearance").and_then(Value::as_str)
        != shared.get("appearance").and_then(Value::as_str)
        && target.get("appearance").is_some()
    {
        return true;
    }
    for appearance in [ThemePackageAppearance::Light, ThemePackageAppearance::Dark] {
        let mut shared_visual = visual_fields(shared);
        if let Some(variant) = appearance_variant(shared, appearance) {
            merge_value(&mut shared_visual, visual_fields(variant));
        }
        let mut resolved = shared_visual.clone();
        merge_value(&mut resolved, visual_fields(target));
        if let Some(variant) = appearance_variant(target, appearance) {
            merge_value(&mut resolved, visual_fields(variant));
        }
        if resolved != shared_visual {
            return true;
        }
    }
    false
}

fn validate_minimum_semantics(visual: &Value) -> Result<(), ThemePackageError> {
    const STRING_FIELDS: [&str; 18] = [
        "/composer/background",
        "/composer/border",
        "/composer/textColor",
        "/composer/placeholderColor",
        "/composer/caretColor",
        "/composer/iconColor",
        "/composer/sendButtonBackground",
        "/composer/sendButtonIconColor",
        "/content/chatBackground",
        "/content/userMessageBackground",
        "/content/userMessageText",
        "/content/assistantMessageBackground",
        "/content/assistantMessageText",
        "/content/codeBackground",
        "/content/codeHeaderBackground",
        "/content/selectionColor",
        "/content/scrollbarColor",
        "/content/scrollbarHoverColor",
    ];
    for pointer in STRING_FIELDS {
        if !visual
            .pointer(pointer)
            .and_then(Value::as_str)
            .is_some_and(|value| !value.trim().is_empty())
        {
            return Err(ThemePackageError::new(
                ThemePackageErrorCategory::Semantic,
                format!("resolved visual is missing required field {pointer}"),
            )
            .at_pointer(pointer));
        }
    }
    if !visual
        .pointer("/composer/radius")
        .and_then(Value::as_f64)
        .is_some_and(f64::is_finite)
    {
        return Err(ThemePackageError::new(
            ThemePackageErrorCategory::Semantic,
            "resolved visual is missing required field /composer/radius",
        )
        .at_pointer("/composer/radius"));
    }
    Ok(())
}

fn collect_manifest_resources(manifest: &Value) -> BTreeSet<String> {
    let mut resources = BTreeSet::new();
    if let Some(preview) = manifest["preview"]["image"].as_str() {
        resources.insert(preview.to_string());
    }
    collect_layer_resources(&manifest["shared"], &mut resources);
    if let Some(targets) = manifest["targets"].as_object() {
        for target in targets.values() {
            if let Some(preview) = target
                .get("preview")
                .and_then(|preview| preview.get("image"))
                .and_then(Value::as_str)
            {
                resources.insert(preview.to_string());
            }
            collect_layer_resources(target, &mut resources);
        }
    }
    resources
}

fn collect_layer_resources(layer: &Value, resources: &mut BTreeSet<String>) {
    resources.extend(collect_visual_resource_paths(&visual_fields(layer)));
    resources.extend(css_paths(layer).unwrap_or_default());
    for appearance in [ThemePackageAppearance::Light, ThemePackageAppearance::Dark] {
        if let Some(variant) = appearance_variant(layer, appearance) {
            resources.extend(collect_visual_resource_paths(&visual_fields(variant)));
            resources.extend(css_paths(variant).unwrap_or_default());
        }
    }
}

fn collect_visual_resource_paths(visual: &Value) -> BTreeSet<String> {
    let mut resources = BTreeSet::new();
    for pointer in ["/background/src", "/background/poster"] {
        if let Some(path) = visual.pointer(pointer).and_then(Value::as_str) {
            resources.insert(path.to_string());
        }
    }
    if let Some(assets) = visual
        .pointer("/typography/assets")
        .and_then(Value::as_array)
    {
        for asset in assets {
            if let Some(path) = asset.get("src").and_then(Value::as_str) {
                resources.insert(path.to_string());
            }
        }
    }
    if let Some(icons) = visual.get("icons").and_then(Value::as_object) {
        for path in icons.values().filter_map(Value::as_str) {
            resources.insert(path.to_string());
        }
    }
    resources
}

#[derive(Debug, Clone, Copy)]
enum ResourceKind {
    Image,
    Video,
    Font,
}

impl ResourceKind {
    const fn label(self) -> &'static str {
        match self {
            Self::Image => "image",
            Self::Video => "video",
            Self::Font => "font",
        }
    }
}

fn validate_declared_previews(root: &Path, manifest: &Value) -> Result<(), ThemePackageError> {
    if let Some(path) = manifest.pointer("/preview/image").and_then(Value::as_str) {
        let path = resolve_package_file(root, path, "preview")?;
        validate_resource_file(&path, ResourceKind::Image)?;
    }
    if let Some(targets) = manifest.get("targets").and_then(Value::as_object) {
        for target in targets.values() {
            if let Some(path) = target.pointer("/preview/image").and_then(Value::as_str) {
                let path = resolve_package_file(root, path, "target preview")?;
                validate_resource_file(&path, ResourceKind::Image)?;
            }
        }
    }
    Ok(())
}

fn validate_visual_resource_types(root: &Path, visual: &Value) -> Result<(), ThemePackageError> {
    if let Some(background) = visual.get("background") {
        match background.get("type").and_then(Value::as_str) {
            Some("image") => validate_visual_path(root, background, "src", ResourceKind::Image)?,
            Some("video") => {
                validate_visual_path(root, background, "src", ResourceKind::Video)?;
                validate_visual_path(root, background, "poster", ResourceKind::Image)?;
            }
            _ => {}
        }
    }
    if let Some(assets) = visual
        .pointer("/typography/assets")
        .and_then(Value::as_array)
    {
        for asset in assets {
            validate_visual_path(root, asset, "src", ResourceKind::Font)?;
        }
    }
    if let Some(icons) = visual.get("icons").and_then(Value::as_object) {
        for relative in icons.values().filter_map(Value::as_str) {
            let path = resolve_package_file(root, relative, "icon")?;
            validate_resource_file(&path, ResourceKind::Image)?;
        }
    }
    Ok(())
}

fn validate_visual_path(
    root: &Path,
    object: &Value,
    key: &str,
    kind: ResourceKind,
) -> Result<(), ThemePackageError> {
    let Some(relative) = object.get(key).and_then(Value::as_str) else {
        return Ok(());
    };
    let path = resolve_package_file(root, relative, kind.label())?;
    validate_resource_file(&path, kind)
}

fn validate_resource_file(path: &Path, kind: ResourceKind) -> Result<(), ThemePackageError> {
    let metadata = fs::metadata(path).map_err(|error| io_error(path, error))?;
    if metadata.len() == 0 || metadata.len() > MAX_RESOURCE_BYTES {
        return Err(resource_error(
            path,
            format!(
                "{} resource must be between 1 byte and 200 MiB",
                kind.label()
            ),
        ));
    }
    let extension = path
        .extension()
        .and_then(|value| value.to_str())
        .map(str::to_ascii_lowercase)
        .unwrap_or_default();
    match kind {
        ResourceKind::Image => match extension.as_str() {
            "png" | "jpg" | "jpeg" => validate_decoded_image(path, &extension),
            "gif" => validate_magic(path, &[b"GIF87a", b"GIF89a"], "GIF image"),
            "webp" => validate_webp(path),
            "svg" => validate_safe_svg(path, metadata.len()),
            _ => Err(resource_error(
                path,
                format!("unsupported image extension: .{extension}"),
            )),
        },
        ResourceKind::Video => match extension.as_str() {
            "mp4" | "m4v" => validate_mp4(path),
            "webm" => validate_magic(path, &[&[0x1a, 0x45, 0xdf, 0xa3]], "WebM video"),
            _ => Err(resource_error(
                path,
                format!("unsupported video extension: .{extension}"),
            )),
        },
        ResourceKind::Font => match extension.as_str() {
            "woff" => validate_magic(path, &[b"wOFF"], "WOFF font"),
            "woff2" => validate_magic(path, &[b"wOF2"], "WOFF2 font"),
            "ttf" => validate_magic(path, &[&[0x00, 0x01, 0x00, 0x00], b"true"], "TrueType font"),
            "otf" => validate_magic(path, &[b"OTTO"], "OpenType font"),
            _ => Err(resource_error(
                path,
                format!("unsupported font extension: .{extension}"),
            )),
        },
    }
}

fn validate_decoded_image(path: &Path, extension: &str) -> Result<(), ThemePackageError> {
    let expected = if extension == "png" {
        image::ImageFormat::Png
    } else {
        image::ImageFormat::Jpeg
    };
    let reader = image::ImageReader::open(path)
        .map_err(|error| resource_error(path, format!("cannot read image: {error}")))?
        .with_guessed_format()
        .map_err(|error| resource_error(path, format!("cannot identify image: {error}")))?;
    if reader.format() != Some(expected) {
        return Err(resource_error(
            path,
            format!("file contents do not match .{extension}"),
        ));
    }
    reader
        .into_dimensions()
        .map_err(|error| resource_error(path, format!("invalid image data: {error}")))?;
    Ok(())
}

fn validate_magic(path: &Path, signatures: &[&[u8]], label: &str) -> Result<(), ThemePackageError> {
    let max = signatures
        .iter()
        .map(|value| value.len())
        .max()
        .unwrap_or(0);
    let mut prefix = vec![0_u8; max];
    let mut file = fs::File::open(path).map_err(|error| io_error(path, error))?;
    let read = file
        .read(&mut prefix)
        .map_err(|error| io_error(path, error))?;
    prefix.truncate(read);
    if signatures
        .iter()
        .any(|signature| prefix.starts_with(signature))
    {
        Ok(())
    } else {
        Err(resource_error(path, format!("file is not a valid {label}")))
    }
}

fn validate_webp(path: &Path) -> Result<(), ThemePackageError> {
    let mut prefix = [0_u8; 12];
    fs::File::open(path)
        .and_then(|mut file| file.read_exact(&mut prefix))
        .map_err(|error| io_error(path, error))?;
    if &prefix[..4] == b"RIFF" && &prefix[8..] == b"WEBP" {
        Ok(())
    } else {
        Err(resource_error(path, "file is not a valid WebP image"))
    }
}

fn validate_mp4(path: &Path) -> Result<(), ThemePackageError> {
    let mut prefix = [0_u8; 12];
    fs::File::open(path)
        .and_then(|mut file| file.read_exact(&mut prefix))
        .map_err(|error| io_error(path, error))?;
    if &prefix[4..8] == b"ftyp" {
        Ok(())
    } else {
        Err(resource_error(path, "file is not a valid MP4 video"))
    }
}

fn validate_safe_svg(path: &Path, size: u64) -> Result<(), ThemePackageError> {
    if size > MAX_SVG_BYTES {
        return Err(resource_error(path, "SVG resource exceeds 2 MiB"));
    }
    let source = fs::read_to_string(path)
        .map_err(|error| resource_error(path, format!("SVG must be valid UTF-8: {error}")))?;
    if source.to_ascii_lowercase().contains("<!doctype") {
        return Err(resource_error(path, "SVG document types are not allowed"));
    }
    let document = roxmltree::Document::parse(&source)
        .map_err(|error| resource_error(path, format!("invalid SVG XML: {error}")))?;
    let root = document.root_element();
    if !root.tag_name().name().eq_ignore_ascii_case("svg") {
        return Err(resource_error(path, "SVG root element must be <svg>"));
    }
    for node in document.descendants().filter(roxmltree::Node::is_element) {
        let tag = node.tag_name().name();
        if matches_ignore_ascii_case(
            tag,
            &[
                "script",
                "foreignObject",
                "iframe",
                "object",
                "embed",
                "image",
                "style",
            ],
        ) {
            return Err(resource_error(path, format!("unsafe SVG element <{tag}>")));
        }
        for attribute in node.attributes() {
            let name = attribute.name();
            let value = attribute.value().trim();
            if name.to_ascii_lowercase().starts_with("on") {
                return Err(resource_error(
                    path,
                    format!("unsafe SVG event attribute {name}"),
                ));
            }
            if matches_ignore_ascii_case(name, &["href", "src"]) && !value.starts_with('#') {
                return Err(resource_error(
                    path,
                    format!("external SVG reference in {name}"),
                ));
            }
            let lower = value.to_ascii_lowercase();
            if lower.contains("javascript:")
                || lower.contains("data:")
                || (lower.contains("url(") && !all_svg_urls_are_fragments(&lower))
            {
                return Err(resource_error(path, format!("unsafe SVG value in {name}")));
            }
        }
    }
    Ok(())
}

fn matches_ignore_ascii_case(value: &str, candidates: &[&str]) -> bool {
    candidates
        .iter()
        .any(|candidate| value.eq_ignore_ascii_case(candidate))
}

fn all_svg_urls_are_fragments(value: &str) -> bool {
    let mut remaining = value;
    while let Some(index) = remaining.find("url(") {
        remaining = &remaining[index + 4..];
        let Some(end) = remaining.find(')') else {
            return false;
        };
        let target = remaining[..end].trim().trim_matches(['\'', '"']);
        if !target.starts_with('#') {
            return false;
        }
        remaining = &remaining[end + 1..];
    }
    true
}

fn resource_error(path: &Path, message: impl Into<String>) -> ThemePackageError {
    ThemePackageError {
        category: ThemePackageErrorCategory::Resource,
        message: message.into(),
        pointer: None,
        target: None,
        appearance: None,
        file: Some(path.to_string_lossy().into_owned()),
        line: None,
        column: None,
    }
}

fn resolve_package_file(
    root: &Path,
    relative: &str,
    label: &str,
) -> Result<PathBuf, ThemePackageError> {
    validate_relative_path(relative)?;
    let mut current = root.to_path_buf();
    for segment in relative.split('/') {
        let exact = fs::read_dir(&current)
            .map_err(|error| io_error(&current, error))?
            .filter_map(Result::ok)
            .find(|entry| entry.file_name().to_str() == Some(segment))
            .ok_or_else(|| {
                ThemePackageError::new(
                    ThemePackageErrorCategory::MissingResource,
                    format!("{label} file does not exist with exact case: {relative}"),
                )
            })?;
        let metadata =
            fs::symlink_metadata(exact.path()).map_err(|error| io_error(&exact.path(), error))?;
        if metadata.file_type().is_symlink() {
            return Err(ThemePackageError::new(
                ThemePackageErrorCategory::Path,
                format!("package paths cannot traverse symlinks: {relative}"),
            ));
        }
        current = exact.path();
    }
    if !current.is_file() {
        return Err(ThemePackageError::new(
            ThemePackageErrorCategory::MissingResource,
            format!("{label} path is not a file: {relative}"),
        ));
    }
    let canonical = current
        .canonicalize()
        .map_err(|error| io_error(&current, error))?;
    if !canonical.starts_with(root) {
        return Err(ThemePackageError::new(
            ThemePackageErrorCategory::Path,
            format!("package path escapes the theme directory: {relative}"),
        ));
    }
    Ok(canonical)
}

fn validate_relative_path(relative: &str) -> Result<(), ThemePackageError> {
    if relative.is_empty()
        || relative.contains('\\')
        || relative.contains('?')
        || relative.contains('#')
        || Path::new(relative).is_absolute()
        || relative.split('/').any(|segment| segment.is_empty())
        || Path::new(relative)
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(ThemePackageError::new(
            ThemePackageErrorCategory::Path,
            format!("invalid package-relative path: {relative}"),
        ));
    }
    Ok(())
}

fn required_string<'a>(manifest: &'a Value, key: &str) -> Result<&'a str, ThemePackageError> {
    manifest.get(key).and_then(Value::as_str).ok_or_else(|| {
        ThemePackageError::new(
            ThemePackageErrorCategory::Manifest,
            format!("theme.json is missing string field {key}"),
        )
        .at_pointer(format!("/{key}"))
    })
}

fn io_error(path: &Path, error: std::io::Error) -> ThemePackageError {
    ThemePackageError {
        category: if error.kind() == std::io::ErrorKind::NotFound {
            ThemePackageErrorCategory::MissingResource
        } else {
            ThemePackageErrorCategory::Io
        },
        message: format!("cannot access {}: {error}", path.display()),
        pointer: None,
        target: None,
        appearance: None,
        file: Some(path.to_string_lossy().into_owned()),
        line: None,
        column: None,
    }
}

fn package_relative(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn merge_recurses_replaces_arrays_and_deletes_nulls() {
        let mut base = serde_json::json!({
            "composer": { "background": "white", "radius": 18 },
            "icons": { "main": "main.svg", "send": "send.svg" },
            "list": [1, 2]
        });
        merge_value(
            &mut base,
            serde_json::json!({
                "composer": { "background": "black" },
                "icons": { "main": null },
                "list": [3]
            }),
        );
        assert_eq!(
            base,
            serde_json::json!({
                "composer": { "background": "black", "radius": 18 },
                "icons": { "send": "send.svg" },
                "list": [3]
            })
        );
    }

    #[test]
    fn invalid_relative_paths_fail_closed() {
        for path in [
            "",
            "/a.css",
            "../a.css",
            "a/../b.css",
            "a\\b.css",
            "a.css?x",
            "a.css#x",
        ] {
            assert!(
                validate_relative_path(path).is_err(),
                "{path:?} should fail"
            );
        }
        assert!(validate_relative_path("styles/doubao-family.css").is_ok());
    }
}
