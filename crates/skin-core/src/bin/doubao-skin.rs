use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use serde_json::{json, Value};
use skin_core::authoring::{self, Appearance, CreateOptions};
use skin_core::theme_package::ThemeTarget;
use skin_core::{build, live, theme};

const EXIT_ARGUMENTS: i32 = 2;
const EXIT_INVALID_THEME: i32 = 3;
const EXIT_EXTERNAL: i32 = 4;

#[derive(Debug, Clone, Copy)]
enum ErrorKind {
    Arguments,
    InvalidTheme,
    External,
}

impl ErrorKind {
    fn code(self) -> &'static str {
        match self {
            Self::Arguments => "arguments",
            Self::InvalidTheme => "invalid-theme",
            Self::External => "external-operation",
        }
    }

    fn exit_code(self) -> i32 {
        match self {
            Self::Arguments => EXIT_ARGUMENTS,
            Self::InvalidTheme => EXIT_INVALID_THEME,
            Self::External => EXIT_EXTERNAL,
        }
    }
}

#[derive(Debug)]
struct CliError {
    kind: ErrorKind,
    message: String,
}

impl CliError {
    fn arguments(message: impl Into<String>) -> Self {
        Self {
            kind: ErrorKind::Arguments,
            message: message.into(),
        }
    }

    fn invalid(message: impl Into<String>) -> Self {
        Self {
            kind: ErrorKind::InvalidTheme,
            message: message.into(),
        }
    }

    fn external(message: impl Into<String>) -> Self {
        Self {
            kind: ErrorKind::External,
            message: message.into(),
        }
    }

    fn theme_operation(message: String) -> Self {
        const FILESYSTEM_FAILURES: &[&str] = &[
            "目标位置不是文件夹",
            "输出文件已经存在",
            "同名安装位置不是主题文件夹",
            "无法读取目标文件夹",
            "无法创建主题文件夹",
            "无法生成 theme.json",
            "无法写入 theme.json",
            "无法写入 theme.css",
            "无法读取 theme.json",
            "无法读取 theme.css",
            "无法创建输出文件夹",
            "无法创建主题包",
            "无法读取已生成主题包",
            "无法读取主题资源信息",
            "无法读取主题资源",
            "无法写入主题包",
            "无法完成主题包",
            "无法生成预览图",
            "无法读取主题包",
            "无法创建主题安装目录",
        ];
        const INSTALL_FAILURES: &[&str] = &[
            "无法准备主题安装",
            "无法打开主题包",
            "无法写入主题文件",
            "无法更新已安装主题",
            "无法安装主题",
            "无法读取主题文件夹",
            "无法读取主题文件",
        ];
        if FILESYSTEM_FAILURES
            .iter()
            .chain(INSTALL_FAILURES.iter())
            .any(|prefix| message.starts_with(*prefix))
            || message.starts_with("无法读取 ")
        {
            Self::external(message)
        } else {
            Self::invalid(message)
        }
    }
}

fn main() {
    let mut args = std::env::args().skip(1).collect::<Vec<_>>();
    let json_output = remove_flag(&mut args, "--json");
    if args
        .iter()
        .any(|arg| matches!(arg.as_str(), "-V" | "--version"))
        || args.first().is_some_and(|arg| arg == "version")
    {
        let version = env!("CARGO_PKG_VERSION");
        if json_output {
            println!(
                "{}",
                json!({"ok": true, "command": "version", "result": {"version": version}})
            );
        } else {
            println!("doubao-skin {version}");
        }
        return;
    }
    if args
        .iter()
        .any(|arg| matches!(arg.as_str(), "-h" | "--help"))
        || args.first().is_some_and(|arg| arg == "help")
    {
        if json_output {
            println!(
                "{}",
                json!({"ok": true, "command": "help", "result": {"usage": usage()}})
            );
        } else {
            println!("{}", usage());
        }
        return;
    }
    let command = args.first().map(String::as_str).unwrap_or("help");
    let result = execute(&args);
    match result {
        Ok(value) => {
            if json_output {
                println!(
                    "{}",
                    json!({"ok": true, "command": command, "result": value})
                );
            } else {
                println!("{}", text_result(command, &value));
            }
        }
        Err(error) => {
            if json_output {
                println!(
                    "{}",
                    json!({
                        "ok": false,
                        "command": command,
                        "error": {"code": error.kind.code(), "message": error.message}
                    })
                );
            } else {
                eprintln!("{}", error.message);
            }
            std::process::exit(error.kind.exit_code());
        }
    }
}

fn execute(args: &[String]) -> Result<Value, CliError> {
    let Some(command) = args.first().map(String::as_str) else {
        return Err(CliError::arguments(format!("缺少命令。\n\n{}", usage())));
    };
    let rest = &args[1..];
    match command {
        "list" => {
            require_empty(rest, "list")?;
            let themes =
                theme::list_available(&theme::default_themes_dir(), &theme::user_themes_dir())
                    .into_iter()
                    .map(|theme| {
                        json!({
                            "id": theme.id,
                            "name": theme.name,
                            "description": theme.description,
                            "version": theme.version,
                            "author": theme.author,
                            "path": theme.path,
                        })
                    })
                    .collect::<Vec<_>>();
            Ok(json!({"themes": themes, "count": themes.len()}))
        }
        "create" => create_command(rest),
        "check" => {
            let theme_dir = one_path(rest, "check <theme-dir>")?;
            let report = authoring::check(&theme_dir).map_err(CliError::theme_operation)?;
            Ok(report_json(&report))
        }
        "preview" => {
            let theme_dir = one_path(rest, "preview <theme-dir>")?;
            let path = authoring::preview(&theme_dir).map_err(CliError::theme_operation)?;
            Ok(json!({"path": path}))
        }
        "migrate-v3" => migrate_v3_command(rest),
        "pack" => pack_command(rest),
        "install" => {
            let package = one_path(rest, "install <package>")?;
            let installed = theme::install_theme_package(&package, &theme::user_themes_dir())
                .map_err(CliError::theme_operation)?;
            Ok(theme_json(&installed))
        }
        "apply" => apply_command(rest),
        "restore" => restore_command(rest),
        "build" => {
            let input = one_value(rest, "build <theme>")?;
            let selected = resolve_theme(input)?;
            let path =
                build::apply(&selected, |line| eprintln!("{line}")).map_err(CliError::external)?;
            Ok(json!({"id": selected.id, "path": path}))
        }
        "remove-build" => {
            require_empty(rest, "remove-build")?;
            build::remove(|line| eprintln!("{line}")).map_err(CliError::external)?;
            let path = build::skin_app().map_err(CliError::external)?;
            Ok(json!({"path": path}))
        }
        _ => Err(CliError::arguments(format!(
            "未知命令：{command}。运行 doubao-skin --help 查看可用命令"
        ))),
    }
}

fn migrate_v3_command(args: &[String]) -> Result<Value, CliError> {
    let theme_dir = args
        .first()
        .filter(|value| !value.starts_with('-'))
        .map(PathBuf::from)
        .ok_or_else(|| CliError::arguments("用法：migrate-v3 <theme-dir> [--write]"))?;
    let write = match &args[1..] {
        [] => false,
        [flag] if flag == "--write" => true,
        _ => {
            return Err(CliError::arguments(
                "用法：migrate-v3 <theme-dir> [--write]",
            ))
        }
    };
    let report = authoring::migrate_v3(&theme_dir, write).map_err(CliError::theme_operation)?;
    serde_json::to_value(report)
        .map_err(|error| CliError::external(format!("无法生成迁移报告：{error}")))
}

fn create_command(args: &[String]) -> Result<Value, CliError> {
    let theme_dir = args
        .first()
        .filter(|value| !value.starts_with('-'))
        .map(PathBuf::from)
        .ok_or_else(|| CliError::arguments("用法：create <theme-dir> --name <名称> [选项]"))?;
    let mut name = None;
    let mut description = None;
    let mut author = "本地用户".to_string();
    let mut accent = "#3370eb".to_string();
    let mut appearance = Appearance::Both;
    let mut targets = None;
    let mut index = 1;
    while index < args.len() {
        match args[index].as_str() {
            "--name" => name = Some(option_value(args, &mut index, "--name")?.to_string()),
            "--description" => {
                description = Some(option_value(args, &mut index, "--description")?.to_string())
            }
            "--author" => author = option_value(args, &mut index, "--author")?.to_string(),
            "--accent" => accent = option_value(args, &mut index, "--accent")?.to_string(),
            "--appearance" => {
                appearance = Appearance::parse(option_value(args, &mut index, "--appearance")?)
                    .map_err(CliError::arguments)?
            }
            "--targets" => {
                targets = Some(parse_targets(option_value(args, &mut index, "--targets")?)?)
            }
            unknown => return Err(CliError::arguments(format!("create 不支持参数 {unknown}"))),
        }
        index += 1;
    }
    let name = name.ok_or_else(|| CliError::arguments("create 需要 --name <名称>"))?;
    let description = description.unwrap_or_else(|| format!("{name}主题"));
    let targets = targets.ok_or_else(|| {
        CliError::arguments(
            "create 需要 --targets <doubao,doubao-work,workbuddy>，请显式声明支持范围",
        )
    })?;
    let report = authoring::create(
        &theme_dir,
        &CreateOptions {
            name,
            description,
            author,
            accent,
            appearance,
            targets,
        },
    )
    .map_err(CliError::theme_operation)?;
    Ok(report_json(&report))
}

fn pack_command(args: &[String]) -> Result<Value, CliError> {
    if !(1..=2).contains(&args.len()) || args[0].starts_with('-') {
        return Err(CliError::arguments(
            "用法：pack <theme-dir> [output.doubao-skin.zip]",
        ));
    }
    let theme_dir = PathBuf::from(&args[0]);
    let id = theme_dir
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| CliError::arguments("无法从主题目录确定输出文件名"))?;
    let output = args.get(1).map(PathBuf::from).unwrap_or_else(|| {
        theme_dir
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .join(format!("{id}.doubao-skin.zip"))
    });
    let report = authoring::pack(&theme_dir, &output).map_err(CliError::theme_operation)?;
    Ok(json!({"id": report.id, "path": output, "files": report.files}))
}

fn apply_command(args: &[String]) -> Result<Value, CliError> {
    let input = args
        .first()
        .filter(|value| !value.starts_with('-'))
        .ok_or_else(|| {
            CliError::arguments("用法：apply <theme> [--target doubao-work] [--watch]")
        })?;
    let mut target = live::TargetApp::DoubaoWork;
    let mut watch = false;
    let mut index = 1;
    while index < args.len() {
        match args[index].as_str() {
            "--target" => {
                let value = option_value(args, &mut index, "--target")?;
                target = live::TargetApp::from_id(value).ok_or_else(|| {
                    CliError::arguments("--target 必须是 doubao、doubao-work 或 workbuddy")
                })?;
            }
            "--watch" => watch = true,
            unknown => return Err(CliError::arguments(format!("apply 不支持参数 {unknown}"))),
        }
        index += 1;
    }
    let selected = resolve_theme(input)?;
    if !selected.supports_target(target) {
        return Err(CliError::invalid(format!(
            "主题 {} 不支持{}",
            selected.name,
            target.display_name()
        )));
    }
    let stop = Arc::new(AtomicBool::new(false));
    let mut injected = 0usize;
    live::run(&selected, target, !watch, stop, |line| {
        if line.trim_start().starts_with("injected:") {
            injected += 1;
        }
        eprintln!("{line}");
    })
    .map_err(CliError::external)?;
    if !watch && injected == 0 {
        return Err(CliError::external(format!(
            "未找到可应用的{}页面，请打开应用后重试",
            target.display_name()
        )));
    }
    Ok(json!({"id": selected.id, "target": target.id(), "pages": injected, "watch": watch}))
}

fn restore_command(args: &[String]) -> Result<Value, CliError> {
    let mut target = live::TargetApp::DoubaoWork;
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--target" => {
                let value = option_value(args, &mut index, "--target")?;
                target = live::TargetApp::from_id(value).ok_or_else(|| {
                    CliError::arguments("--target 必须是 doubao、doubao-work 或 workbuddy")
                })?;
            }
            unknown => return Err(CliError::arguments(format!("restore 不支持参数 {unknown}"))),
        }
        index += 1;
    }
    let restored = live::restore(target, |line| eprintln!("{line}")).map_err(CliError::external)?;
    if restored == 0 {
        return Err(CliError::external(format!(
            "没有清理到{}页面；请确认应用已打开且没有仍在运行的 --watch 进程",
            target.display_name()
        )));
    }
    Ok(json!({"target": target.id(), "pages": restored}))
}

fn resolve_theme(input: &str) -> Result<theme::Theme, CliError> {
    if Path::new(input).is_dir() {
        return theme::load(&theme::default_themes_dir(), input).map_err(CliError::invalid);
    }
    let installed = theme::user_themes_dir();
    if installed.join(input).is_dir() {
        return theme::load(&installed, input).map_err(CliError::invalid);
    }
    let bundled = theme::default_themes_dir();
    if bundled.join(input).is_dir() {
        return theme::load(&bundled, input).map_err(CliError::invalid);
    }
    Err(CliError::invalid(format!(
        "找不到主题 {input}，请先运行 doubao-skin list"
    )))
}

fn one_path(args: &[String], usage: &str) -> Result<PathBuf, CliError> {
    one_value(args, usage).map(PathBuf::from)
}

fn one_value<'a>(args: &'a [String], usage: &str) -> Result<&'a str, CliError> {
    if args.len() != 1 || args[0].starts_with('-') {
        return Err(CliError::arguments(format!("用法：{usage}")));
    }
    Ok(&args[0])
}

fn option_value<'a>(
    args: &'a [String],
    index: &mut usize,
    option: &str,
) -> Result<&'a str, CliError> {
    *index += 1;
    args.get(*index)
        .map(String::as_str)
        .ok_or_else(|| CliError::arguments(format!("{option} 缺少值")))
}

fn require_empty(args: &[String], usage: &str) -> Result<(), CliError> {
    if args.is_empty() {
        Ok(())
    } else {
        Err(CliError::arguments(format!("用法：{usage}")))
    }
}

fn remove_flag(args: &mut Vec<String>, flag: &str) -> bool {
    let found = args.iter().any(|arg| arg == flag);
    args.retain(|arg| arg != flag);
    found
}

fn report_json(report: &authoring::CheckReport) -> Value {
    json!({
        "id": report.id,
        "files": report.files,
        "validation": report.validation
    })
}

fn parse_targets(value: &str) -> Result<std::collections::BTreeSet<ThemeTarget>, CliError> {
    let mut targets = std::collections::BTreeSet::new();
    for value in value.split(',') {
        let target = ThemeTarget::parse(value.trim()).ok_or_else(|| {
            CliError::arguments("--targets 只支持 doubao、doubao-work、workbuddy")
        })?;
        targets.insert(target);
    }
    if targets.is_empty() {
        return Err(CliError::arguments("--targets 不能为空"));
    }
    Ok(targets)
}

fn theme_json(theme: &theme::Theme) -> Value {
    json!({
        "id": theme.id,
        "name": theme.name,
        "version": theme.version,
        "path": theme.path,
    })
}

fn text_result(command: &str, value: &Value) -> String {
    match command {
        "list" => {
            let rows = value["themes"]
                .as_array()
                .into_iter()
                .flatten()
                .map(|theme| {
                    format!(
                        "- {} ({})",
                        theme["name"].as_str().unwrap_or("未命名主题"),
                        theme["id"].as_str().unwrap_or("unknown")
                    )
                })
                .collect::<Vec<_>>();
            if rows.is_empty() {
                "没有找到可用主题".into()
            } else {
                format!("共找到 {} 个主题\n{}", rows.len(), rows.join("\n"))
            }
        }
        "create" => format!("主题已创建：{}", text_field(value, "id")),
        "check" => format!("检查通过：{}", text_field(value, "id")),
        "preview" => format!("预览已生成：{}", text_field(value, "path")),
        "migrate-v3" => {
            if value["written"].as_bool() == Some(true) {
                format!("主题已迁移到 v3：{}", text_field(value, "id"))
            } else {
                format!("v3 迁移预检通过：{}", text_field(value, "id"))
            }
        }
        "pack" => format!("主题包已生成：{}", text_field(value, "path")),
        "install" => format!("主题已安装：{}", text_field(value, "id")),
        "apply" => format!("主题已应用：{}", text_field(value, "id")),
        "restore" => "已恢复默认".into(),
        "build" => format!("主题版已生成：{}", text_field(value, "path")),
        "remove-build" => "已移除主题版".into(),
        _ => value.to_string(),
    }
}

fn text_field<'a>(value: &'a Value, key: &str) -> &'a str {
    value[key].as_str().unwrap_or("未知")
}

fn usage() -> &'static str {
    concat!(
        "doubao-skin ",
        env!("CARGO_PKG_VERSION"),
        " — 豆皮命令行工具\n\n\
用法：\n\
  doubao-skin list [--json]\n\
  doubao-skin create <theme-dir> --name <名称> --targets <doubao,doubao-work,workbuddy> [--description <描述>] [--accent <#RRGGBB>] [--appearance light|dark|both] [--author <作者>]\n\
  doubao-skin check <theme-dir>\n\
  doubao-skin preview <theme-dir>\n\
  doubao-skin migrate-v3 <theme-dir> [--write]\n\
  doubao-skin pack <theme-dir> [output.doubao-skin.zip]\n\
  doubao-skin install <package>\n\
  doubao-skin apply <theme> [--target doubao|doubao-work|workbuddy] [--watch]  # WorkBuddy 仅 macOS\n\
  doubao-skin restore [--target doubao|doubao-work|workbuddy]                  # WorkBuddy 仅 macOS\n\
  doubao-skin build <theme>                                        # 仅 macOS\n\
  doubao-skin remove-build                                         # 仅 macOS\n\
  doubao-skin --version\n\n\
退出码：0 成功，2 参数错误，3 主题无效，4 外部操作失败",
    )
}
