pub struct Locale {
    pub app_name: &'static str,

    // Menu
    pub menu_about: &'static str,
    pub menu_services: &'static str,
    pub menu_hide: &'static str,
    pub menu_hide_others: &'static str,
    pub menu_show_all: &'static str,
    pub menu_quit: &'static str,
    pub target_doubao: &'static str,
    pub target_doubao_greeting: &'static str,
    pub target_doubao_work: &'static str,
    pub target_doubao_work_greeting: &'static str,

    // Source views
    pub source_library: &'static str,
    pub source_store: &'static str,

    // Theme actions
    pub action_applied: &'static str,
    pub action_restored: &'static str,
    pub action_apply_failed: &'static str,
    pub action_applying: &'static str,
    pub action_restoring: &'static str,
    pub action_apply_theme: &'static str,
    pub action_in_use: &'static str,
    pub action_restore_default: &'static str,

    // Automatic theme lifecycle
    pub auto_theme_keep_title: &'static str,
    pub auto_theme_keep_description: &'static str,
    pub auto_theme_login_title: &'static str,
    pub auto_theme_apply_first: &'static str,
    pub auto_theme_unsupported: &'static str,
    pub auto_theme_enabling: &'static str,
    pub auto_theme_disabling: &'static str,
    pub auto_theme_enabled: &'static str,
    pub auto_theme_disabled: &'static str,
    pub auto_theme_cleanup_pending: &'static str,
    pub auto_theme_approval_required: &'static str,
    pub auto_theme_not_ready: &'static str,
    pub auto_theme_missing_service: &'static str,
    pub auto_theme_open_settings: &'static str,
    pub auto_theme_login_enabled: &'static str,
    pub auto_theme_login_disabled: &'static str,
    pub auto_theme_save_failed: &'static str,
    pub auto_theme_rollback_failed: &'static str,
    pub auto_theme_restore_cleanup_failed: &'static str,

    // Install
    pub install_installing: &'static str,
    pub install_one_done: &'static str,
    pub install_prompt_title: &'static str,
    pub install_drop_hint: &'static str,
    pub install_choose_file: &'static str,
    pub install_button: &'static str,
    pub install_button_done: &'static str,
    pub install_button_busy: &'static str,
    pub install_package_fallback_name: &'static str,

    // Search
    pub search_placeholder: &'static str,
    pub search_clear_label: &'static str,
    pub search_no_match: &'static str,

    // Store
    pub store_title: &'static str,
    pub store_refresh: &'static str,
    pub store_refresh_label: &'static str,
    pub store_connecting: &'static str,
    pub store_connect_failed: &'static str,
    pub store_connect_full_failed: &'static str,
    pub store_no_themes: &'static str,
    pub store_select_hint: &'static str,
    pub store_loading: &'static str,

    // Store categories
    pub cat_pure: &'static str,
    pub cat_atmosphere: &'static str,
    pub cat_gallery: &'static str,
    pub cat_codex: &'static str,
    pub cat_brand: &'static str,
    pub cat_default: &'static str,

    // Misc
    pub aria_all_themes: &'static str,
    pub aria_app: &'static str,
    pub aria_store_themes: &'static str,
    pub opacity_label: &'static str,
    pub drop_label: &'static str,
    pub empty_library: &'static str,
    pub not_installed_target: &'static str,
    pub theme_unavailable: &'static str,
    pub section_pinned: &'static str,
    pub section_projects: &'static str,
    pub section_recommended: &'static str,
    pub error_keyword: &'static str,

    // Preview navigation labels
    pub nav_new_task: &'static str,
    pub nav_scheduled: &'static str,
    pub nav_skills: &'static str,
    pub nav_cloud: &'static str,
    pub nav_remote: &'static str,
    pub nav_main_conversation: &'static str,
    pub nav_look: &'static str,
    pub nav_work_header: &'static str,
    pub nav_daily_work: &'static str,
    pub nav_content_creation: &'static str,
    pub nav_research: &'static str,
    pub nav_design: &'static str,
    pub nav_composer_placeholder: &'static str,
}

impl Locale {
    pub fn format_install_fail(&self, error: &str) -> String {
        format!("安装失败：{error}")
    }
    pub fn format_install_partial(&self, n: usize, error: &str) -> String {
        format!("已安装 {n} 个主题；{error}")
    }
    pub fn format_install_count(&self, n: usize) -> String {
        format!("已安装 {n} 个主题")
    }
    pub fn format_not_installed(&self, name: &str) -> String {
        format!("尚未安装{name}")
    }
    pub fn format_please_install(&self, name: &str) -> String {
        format!("请先安装{name}")
    }
    pub fn format_searching_theme(&self, id: &str) -> String {
        format!("正在查找主题「{id}」…")
    }
    pub fn format_target_label(&self, name: &str, missing: bool) -> String {
        if missing {
            format!("{name} · 未安装")
        } else {
            name.to_string()
        }
    }
    pub fn format_target_aria(
        &self,
        name: &str,
        installed: bool,
        selected: bool,
        shortcut: &str,
    ) -> String {
        let state = if !installed {
            "未安装"
        } else if selected {
            "已选中"
        } else {
            "可选"
        };
        format!("{name}，{state}，{shortcut}")
    }
    pub fn format_opacity_aria(&self, percent: u32) -> String {
        format!("{} {percent}%", self.opacity_label)
    }
    pub fn format_auto_theme_login_description(&self, name: &str) -> String {
        format!("登录电脑后自动打开{name}")
    }
    pub fn format_switch_aria(&self, title: &str, enabled: bool) -> String {
        if enabled {
            title.to_string()
        } else {
            format!("{title}，不可用")
        }
    }
    pub fn format_store_item_aria(&self, name: &str, installed: bool) -> String {
        if installed {
            format!("{name} 已安装")
        } else {
            format!("安装 {name}")
        }
    }
}

pub static ZH_CN: Locale = Locale {
    app_name: "豆皮",

    menu_about: "关于豆皮",
    menu_services: "服务",
    menu_hide: "隐藏豆皮",
    menu_hide_others: "隐藏其他",
    menu_show_all: "全部显示",
    menu_quit: "退出豆皮",

    target_doubao: "豆包",
    target_doubao_greeting: "有什么我能帮你的？",
    target_doubao_work: "豆包工作",
    target_doubao_work_greeting: "今天有什么工作要处理？",

    source_library: "我的主题",
    source_store: "主题商店",

    action_applied: "已应用",
    action_restored: "已恢复默认",
    action_apply_failed: "应用失败，请再试一次",
    action_applying: "正在应用…",
    action_restoring: "正在恢复…",
    action_apply_theme: "应用主题",
    action_in_use: "正在使用",
    action_restore_default: "恢复默认",

    auto_theme_keep_title: "自动保持上次主题",
    auto_theme_keep_description: "关闭豆皮后，下次打开仍会恢复当前主题",
    auto_theme_login_title: "登录时打开豆包",
    auto_theme_apply_first: "请先成功应用一个主题",
    auto_theme_unsupported: "自动保持主题需要 macOS 13 或更高版本",
    auto_theme_enabling: "正在开启自动保持主题…",
    auto_theme_disabling: "正在关闭自动保持主题…",
    auto_theme_enabled: "豆皮后台服务已注册",
    auto_theme_disabled: "自动保持主题已关闭",
    auto_theme_cleanup_pending: "后台启动项尚未移除，再点一次开关重试",
    auto_theme_approval_required: "需要在系统设置中允许豆皮后台运行",
    auto_theme_not_ready: "豆皮后台服务尚未启用",
    auto_theme_missing_service: "当前安装包不包含豆皮后台服务",
    auto_theme_open_settings: "打开系统设置",
    auto_theme_login_enabled: "登录时将自动打开豆包",
    auto_theme_login_disabled: "登录时不会自动打开豆包",
    auto_theme_save_failed: "主题已应用，但无法保存自动恢复设置",
    auto_theme_rollback_failed: "后台服务启用失败，自动恢复设置也未能回滚",
    auto_theme_restore_cleanup_failed: "已恢复默认，但自动恢复设置未能清除",

    install_installing: "正在安装主题…",
    install_one_done: "主题已安装",
    install_prompt_title: "安装主题",
    install_drop_hint: "拖入主题包即可安装",
    install_choose_file: "选择文件…",
    install_button: "安装",
    install_button_done: "已安装",
    install_button_busy: "正在安装…",
    install_package_fallback_name: "主题包",

    search_placeholder: "搜索主题",
    search_clear_label: "清除搜索",
    search_no_match: "没有匹配的主题",

    store_title: "主题商店",
    store_refresh: "刷新",
    store_refresh_label: "刷新主题商店",
    store_connecting: "正在连接…",
    store_connect_failed: "暂时无法连接",
    store_connect_full_failed: "暂时无法打开主题商店",
    store_no_themes: "暂时没有可用主题",
    store_select_hint: "选择一个主题以查看详情",
    store_loading: "正在连接主题商店…",

    cat_pure: "纯色",
    cat_atmosphere: "氛围背景",
    cat_gallery: "热门灵感",
    cat_codex: "编辑器配色",
    cat_brand: "品牌灵感",
    cat_default: "主题",

    aria_all_themes: "全部主题",
    aria_app: "豆皮",
    aria_store_themes: "商店主题",
    opacity_label: "界面不透明度",
    drop_label: "拖入主题包即可安装，或选择文件",
    empty_library: "还没有可用主题",
    not_installed_target: "尚未安装目标应用",
    theme_unavailable: "这个主题暂时不可用",
    section_pinned: "置顶",
    section_projects: "项目",
    section_recommended: "为你推荐",
    error_keyword: "失败",

    nav_new_task: "新工作任务",
    nav_scheduled: "定时任务",
    nav_skills: "技能 · 连接器 · 伙伴",
    nav_cloud: "云盘",
    nav_remote: "手机遥控电脑",
    nav_main_conversation: "主对话",
    nav_look: "看看",
    nav_work_header: "豆包 工作",
    nav_daily_work: "处理日常工作",
    nav_content_creation: "内容创作",
    nav_research: "完成调研分析",
    nav_design: "设计与创意",
    nav_composer_placeholder: "输入问题或任务，/ 选择技能",
};

pub fn t() -> &'static Locale {
    &ZH_CN
}

pub fn store_category_label(category: &str) -> &'static str {
    let l = t();
    match category {
        "pure" => l.cat_pure,
        "atmosphere" => l.cat_atmosphere,
        "gallery" => l.cat_gallery,
        "codex" => l.cat_codex,
        "brand" => l.cat_brand,
        _ => l.cat_default,
    }
}
