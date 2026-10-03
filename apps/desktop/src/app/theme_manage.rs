//! Revealing and removing installed themes.

use gpui::Context;

use skin_core::theme;

use crate::app::SkinApp;
use crate::i18n::t;
use crate::trash;

impl SkinApp {
    pub(crate) fn selected_theme_is_user_installed(&self) -> bool {
        self.themes
            .get(self.selected)
            .is_some_and(|row| theme::is_user_theme(&row.theme, &theme::user_themes_dir()))
    }

    /// Deleting is offered only where it can be undone from the system Trash,
    /// and only for themes the user installed (never bundled ones).
    pub(crate) fn can_delete_selected(&self) -> bool {
        trash::SUPPORTED && self.selected_theme_is_user_installed()
    }

    pub(crate) fn selected_theme_in_use(&self) -> bool {
        self.themes
            .get(self.selected)
            .is_some_and(|row| self.theme_sessions.uses_theme(&row.theme.id))
    }

    pub(crate) fn reveal_selected_theme(&mut self, cx: &mut Context<Self>) {
        if let Some(row) = self.themes.get(self.selected) {
            cx.reveal_path(&row.theme.path);
        }
    }

    pub(crate) fn request_delete_selected(&mut self, cx: &mut Context<Self>) {
        let Some(row) = self.themes.get(self.selected) else {
            return;
        };
        if !self.can_delete_selected() {
            return;
        }
        if self.selected_theme_in_use() {
            self.message = t().delete_in_use.into();
        } else {
            self.confirm_delete = Some(row.theme.id.clone());
        }
        cx.notify();
    }

    pub(crate) fn cancel_delete(&mut self, cx: &mut Context<Self>) {
        self.confirm_delete = None;
        cx.notify();
    }

    pub(crate) fn delete_selected(&mut self, cx: &mut Context<Self>) {
        let Some(row) = self.themes.get(self.selected) else {
            return;
        };
        let id = row.theme.id.clone();
        if self.confirm_delete.as_deref() != Some(id.as_str()) {
            return;
        }
        self.confirm_delete = None;
        let l = t();
        if self.theme_sessions.uses_theme(&id) {
            self.message = l.delete_in_use.into();
            cx.notify();
            return;
        }
        let neighbor_id = self
            .themes
            .get(self.selected + 1)
            .or_else(|| {
                self.selected
                    .checked_sub(1)
                    .and_then(|index| self.themes.get(index))
            })
            .map(|row| row.theme.id.clone());
        let result = theme::resolve_user_theme_dir(&id, &theme::user_themes_dir())
            .and_then(|path| trash::move_to_trash(&path));
        match result {
            Ok(_) => {
                self.reload_themes(Some(&id));
                let restored_bundled = self.themes.iter().any(|row| row.theme.id == id);
                if !restored_bundled {
                    self.reload_themes(neighbor_id.as_deref());
                }
                self.ensure_selected_match();
                self.message = if restored_bundled {
                    l.delete_done_bundled.into()
                } else {
                    l.delete_done.into()
                };
            }
            Err(error) => self.message = l.format_delete_failed(&error),
        }
        cx.notify();
    }
}
