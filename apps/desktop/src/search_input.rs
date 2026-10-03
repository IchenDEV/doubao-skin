//! A single-line text input for the theme search box.
//!
//! Unlike the old hand-rolled key handling it talks to the platform text
//! input system (`EntityInputHandler`), so Chinese input methods, dead keys,
//! copy/paste, selection and cursor movement all work.
//!
//! The element and input-handler plumbing is adapted from GPUI's own
//! `crates/gpui/examples/input.rs` at the pinned revision
//! (Copyright Zed Industries, Inc., licensed under Apache-2.0). Text editing
//! rules live in [`TextBuffer`], which has no GPUI dependency and is unit
//! tested; cursor movement uses `char` boundaries rather than grapheme
//! clusters to avoid adding a direct dependency.

use std::ops::Range;

use gpui::{
    actions, div, fill, point, prelude::*, px, relative, size, App, Bounds, ClipboardItem, Context,
    CursorStyle, ElementId, ElementInputHandler, Entity, EntityInputHandler, EventEmitter,
    FocusHandle, Focusable, GlobalElementId, Hsla, LayoutId, MouseButton, MouseDownEvent,
    MouseMoveEvent, MouseUpEvent, PaintQuad, Pixels, Point, Role, ShapedLine, SharedString, Style,
    TextRun, UTF16Selection, UnderlineStyle, Window,
};

actions!(
    search_input,
    [
        Backspace,
        Delete,
        Left,
        Right,
        SelectLeft,
        SelectRight,
        SelectAll,
        Home,
        End,
        ShowCharacterPalette,
        Paste,
        Cut,
        Copy,
    ]
);

pub const KEY_CONTEXT: &str = "SearchInput";

/// Key bindings scoped to the search box, to be registered once at startup.
pub fn key_bindings() -> Vec<gpui::KeyBinding> {
    use gpui::KeyBinding;
    vec![
        KeyBinding::new("backspace", Backspace, Some(KEY_CONTEXT)),
        KeyBinding::new("delete", Delete, Some(KEY_CONTEXT)),
        KeyBinding::new("left", Left, Some(KEY_CONTEXT)),
        KeyBinding::new("right", Right, Some(KEY_CONTEXT)),
        KeyBinding::new("shift-left", SelectLeft, Some(KEY_CONTEXT)),
        KeyBinding::new("shift-right", SelectRight, Some(KEY_CONTEXT)),
        KeyBinding::new("cmd-a", SelectAll, Some(KEY_CONTEXT)),
        KeyBinding::new("cmd-v", Paste, Some(KEY_CONTEXT)),
        KeyBinding::new("cmd-c", Copy, Some(KEY_CONTEXT)),
        KeyBinding::new("cmd-x", Cut, Some(KEY_CONTEXT)),
        KeyBinding::new("home", Home, Some(KEY_CONTEXT)),
        KeyBinding::new("end", End, Some(KEY_CONTEXT)),
        KeyBinding::new("ctrl-cmd-space", ShowCharacterPalette, Some(KEY_CONTEXT)),
    ]
}

/// Flatten pasted text to one line.
pub fn single_line(text: &str) -> String {
    text.replace("\r\n", " ").replace(['\n', '\r', '\t'], " ")
}

/// Editable text with a selection and optional input-method marked range.
/// All offsets are UTF-8 byte offsets unless a method says `utf16`.
#[derive(Default, Debug)]
pub struct TextBuffer {
    content: String,
    selected: Range<usize>,
    reversed: bool,
    marked: Option<Range<usize>>,
}

impl TextBuffer {
    pub fn text(&self) -> &str {
        &self.content
    }

    pub fn is_empty(&self) -> bool {
        self.content.is_empty()
    }

    pub fn selection(&self) -> Range<usize> {
        self.selected.clone()
    }

    pub fn selected_text(&self) -> &str {
        &self.content[self.selected.clone()]
    }

    pub fn marked_utf8(&self) -> Option<Range<usize>> {
        self.marked.clone()
    }

    pub fn is_composing(&self) -> bool {
        self.marked.is_some()
    }

    pub fn is_reversed(&self) -> bool {
        self.reversed
    }

    pub fn cursor(&self) -> usize {
        if self.reversed {
            self.selected.start
        } else {
            self.selected.end
        }
    }

    pub fn set_text(&mut self, text: &str) {
        self.content = text.to_string();
        let end = self.content.len();
        self.selected = end..end;
        self.reversed = false;
        self.marked = None;
    }

    pub fn prev_boundary(&self, offset: usize) -> usize {
        self.content[..offset.min(self.content.len())]
            .char_indices()
            .next_back()
            .map(|(index, _)| index)
            .unwrap_or(0)
    }

    pub fn next_boundary(&self, offset: usize) -> usize {
        self.content[offset.min(self.content.len())..]
            .chars()
            .next()
            .map(|ch| offset + ch.len_utf8())
            .unwrap_or(self.content.len())
    }

    pub fn move_to(&mut self, offset: usize) {
        let offset = offset.min(self.content.len());
        self.selected = offset..offset;
        self.reversed = false;
    }

    pub fn select_to(&mut self, offset: usize) {
        let offset = offset.min(self.content.len());
        if self.reversed {
            self.selected.start = offset;
        } else {
            self.selected.end = offset;
        }
        if self.selected.end < self.selected.start {
            self.reversed = !self.reversed;
            self.selected = self.selected.end..self.selected.start;
        }
    }

    pub fn select_all(&mut self) {
        self.selected = 0..self.content.len();
        self.reversed = false;
    }

    /// Delete the selection, or the character before the cursor. Returns
    /// whether anything changed.
    pub fn backspace(&mut self) -> bool {
        if self.selected.is_empty() {
            let previous = self.prev_boundary(self.cursor());
            if previous == self.cursor() {
                return false;
            }
            self.select_to(previous);
        }
        self.replace(None, "");
        true
    }

    /// Delete the selection, or the character after the cursor.
    pub fn delete(&mut self) -> bool {
        if self.selected.is_empty() {
            let next = self.next_boundary(self.cursor());
            if next == self.cursor() {
                return false;
            }
            self.select_to(next);
        }
        self.replace(None, "");
        true
    }

    fn target_range(&self, range_utf16: Option<Range<usize>>) -> Range<usize> {
        range_utf16
            .as_ref()
            .map(|range| self.range_from_utf16(range))
            .or_else(|| self.marked.clone())
            .unwrap_or_else(|| self.selected.clone())
    }

    /// Insert `new_text` over `range_utf16`, the marked text or the selection
    /// and commit it (the input method is done composing).
    pub fn replace(&mut self, range_utf16: Option<Range<usize>>, new_text: &str) {
        let range = self.target_range(range_utf16);
        self.content.replace_range(range.clone(), new_text);
        let end = range.start + new_text.len();
        self.selected = end..end;
        self.reversed = false;
        self.marked = None;
    }

    /// Like [`replace`](Self::replace) but keeps the inserted text marked as
    /// still being composed.
    pub fn replace_and_mark(
        &mut self,
        range_utf16: Option<Range<usize>>,
        new_text: &str,
        new_selected_utf16: Option<Range<usize>>,
    ) {
        let range = self.target_range(range_utf16);
        self.content.replace_range(range.clone(), new_text);
        self.marked = (!new_text.is_empty()).then(|| range.start..range.start + new_text.len());
        self.selected = new_selected_utf16
            .map(|selected| {
                let local = self.local_range_from_utf16(new_text, &selected);
                range.start + local.start..range.start + local.end
            })
            .unwrap_or_else(|| {
                let end = range.start + new_text.len();
                end..end
            });
        self.reversed = false;
    }

    fn local_range_from_utf16(&self, text: &str, range: &Range<usize>) -> Range<usize> {
        fn offset(text: &str, target: usize) -> usize {
            let mut utf8 = 0;
            let mut utf16 = 0;
            for ch in text.chars() {
                if utf16 >= target {
                    break;
                }
                utf16 += ch.len_utf16();
                utf8 += ch.len_utf8();
            }
            utf8
        }
        offset(text, range.start)..offset(text, range.end)
    }

    pub fn offset_from_utf16(&self, offset: usize) -> usize {
        let mut utf8_offset = 0;
        let mut utf16_count = 0;
        for ch in self.content.chars() {
            if utf16_count >= offset {
                break;
            }
            utf16_count += ch.len_utf16();
            utf8_offset += ch.len_utf8();
        }
        utf8_offset
    }

    pub fn offset_to_utf16(&self, offset: usize) -> usize {
        let mut utf16_offset = 0;
        let mut utf8_count = 0;
        for ch in self.content.chars() {
            if utf8_count >= offset {
                break;
            }
            utf8_count += ch.len_utf8();
            utf16_offset += ch.len_utf16();
        }
        utf16_offset
    }

    pub fn range_to_utf16(&self, range: &Range<usize>) -> Range<usize> {
        self.offset_to_utf16(range.start)..self.offset_to_utf16(range.end)
    }

    pub fn range_from_utf16(&self, range: &Range<usize>) -> Range<usize> {
        self.offset_from_utf16(range.start)..self.offset_from_utf16(range.end)
    }
}

/// Colours the host app pushes into the input so it follows the palette.
#[derive(Clone, Copy, PartialEq)]
pub struct InputColors {
    pub text: Hsla,
    pub placeholder: Hsla,
    pub cursor: Hsla,
    pub selection: Hsla,
}

pub enum SearchEvent {
    /// The committed text changed (not emitted while composing).
    Changed(String),
}

pub struct SearchInput {
    focus_handle: FocusHandle,
    buffer: TextBuffer,
    placeholder: SharedString,
    pub colors: InputColors,
    last_layout: Option<ShapedLine>,
    last_bounds: Option<Bounds<Pixels>>,
    is_selecting: bool,
}

impl EventEmitter<SearchEvent> for SearchInput {}

impl SearchInput {
    pub fn new(placeholder: impl Into<SharedString>, colors: InputColors, cx: &mut App) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
            buffer: TextBuffer::default(),
            placeholder: placeholder.into(),
            colors,
            last_layout: None,
            last_bounds: None,
            is_selecting: false,
        }
    }

    pub fn text(&self) -> &str {
        self.buffer.text()
    }

    pub fn is_composing(&self) -> bool {
        self.buffer.is_composing()
    }

    /// Replace the text without emitting [`SearchEvent::Changed`]; used when
    /// the host app changed the query itself.
    pub fn sync_text(&mut self, text: &str, cx: &mut Context<Self>) {
        self.buffer.set_text(text);
        cx.notify();
    }

    pub fn clear(&mut self, cx: &mut Context<Self>) {
        self.buffer.set_text("");
        self.emit_changed(cx);
        cx.notify();
    }

    fn emit_changed(&mut self, cx: &mut Context<Self>) {
        if !self.buffer.is_composing() {
            cx.emit(SearchEvent::Changed(self.buffer.text().to_string()));
        }
    }

    fn left(&mut self, _: &Left, _: &mut Window, cx: &mut Context<Self>) {
        if self.buffer.selection().is_empty() {
            let target = self.buffer.prev_boundary(self.buffer.cursor());
            self.buffer.move_to(target);
        } else {
            self.buffer.move_to(self.buffer.selection().start);
        }
        cx.notify();
    }

    fn right(&mut self, _: &Right, _: &mut Window, cx: &mut Context<Self>) {
        if self.buffer.selection().is_empty() {
            let target = self.buffer.next_boundary(self.buffer.selection().end);
            self.buffer.move_to(target);
        } else {
            self.buffer.move_to(self.buffer.selection().end);
        }
        cx.notify();
    }

    fn select_left(&mut self, _: &SelectLeft, _: &mut Window, cx: &mut Context<Self>) {
        let target = self.buffer.prev_boundary(self.buffer.cursor());
        self.buffer.select_to(target);
        cx.notify();
    }

    fn select_right(&mut self, _: &SelectRight, _: &mut Window, cx: &mut Context<Self>) {
        let target = self.buffer.next_boundary(self.buffer.cursor());
        self.buffer.select_to(target);
        cx.notify();
    }

    fn select_all(&mut self, _: &SelectAll, _: &mut Window, cx: &mut Context<Self>) {
        self.buffer.select_all();
        cx.notify();
    }

    fn home(&mut self, _: &Home, _: &mut Window, cx: &mut Context<Self>) {
        self.buffer.move_to(0);
        cx.notify();
    }

    fn end(&mut self, _: &End, _: &mut Window, cx: &mut Context<Self>) {
        self.buffer.move_to(self.buffer.text().len());
        cx.notify();
    }

    fn backspace(&mut self, _: &Backspace, window: &mut Window, cx: &mut Context<Self>) {
        if self.buffer.backspace() {
            self.emit_changed(cx);
            cx.notify();
        } else {
            window.play_system_bell();
        }
    }

    fn delete(&mut self, _: &Delete, window: &mut Window, cx: &mut Context<Self>) {
        if self.buffer.delete() {
            self.emit_changed(cx);
            cx.notify();
        } else {
            window.play_system_bell();
        }
    }

    fn show_character_palette(
        &mut self,
        _: &ShowCharacterPalette,
        window: &mut Window,
        _: &mut Context<Self>,
    ) {
        window.show_character_palette();
    }

    fn paste(&mut self, _: &Paste, _window: &mut Window, cx: &mut Context<Self>) {
        if let Some(text) = cx.read_from_clipboard().and_then(|item| item.text()) {
            self.buffer.replace(None, &single_line(&text));
            self.emit_changed(cx);
            cx.notify();
        }
    }

    fn copy(&mut self, _: &Copy, _: &mut Window, cx: &mut Context<Self>) {
        if !self.buffer.selection().is_empty() {
            cx.write_to_clipboard(ClipboardItem::new_string(
                self.buffer.selected_text().to_string(),
            ));
        }
    }

    fn cut(&mut self, _: &Cut, _window: &mut Window, cx: &mut Context<Self>) {
        if !self.buffer.selection().is_empty() {
            cx.write_to_clipboard(ClipboardItem::new_string(
                self.buffer.selected_text().to_string(),
            ));
            self.buffer.replace(None, "");
            self.emit_changed(cx);
            cx.notify();
        }
    }

    fn on_mouse_down(&mut self, event: &MouseDownEvent, _: &mut Window, cx: &mut Context<Self>) {
        self.is_selecting = true;
        let index = self.index_for_mouse_position(event.position);
        if event.modifiers.shift {
            self.buffer.select_to(index);
        } else {
            self.buffer.move_to(index);
        }
        cx.notify();
    }

    fn on_mouse_up(&mut self, _: &MouseUpEvent, _: &mut Window, _: &mut Context<Self>) {
        self.is_selecting = false;
    }

    fn on_mouse_move(&mut self, event: &MouseMoveEvent, _: &mut Window, cx: &mut Context<Self>) {
        if self.is_selecting {
            let index = self.index_for_mouse_position(event.position);
            self.buffer.select_to(index);
            cx.notify();
        }
    }

    fn index_for_mouse_position(&self, position: Point<Pixels>) -> usize {
        if self.buffer.is_empty() {
            return 0;
        }
        let (Some(bounds), Some(line)) = (self.last_bounds.as_ref(), self.last_layout.as_ref())
        else {
            return 0;
        };
        if position.y < bounds.top() {
            return 0;
        }
        if position.y > bounds.bottom() {
            return self.buffer.text().len();
        }
        line.closest_index_for_x(position.x - bounds.left())
    }
}

impl EntityInputHandler for SearchInput {
    fn text_for_range(
        &mut self,
        range_utf16: Range<usize>,
        actual_range: &mut Option<Range<usize>>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<String> {
        let range = self.buffer.range_from_utf16(&range_utf16);
        actual_range.replace(self.buffer.range_to_utf16(&range));
        Some(self.buffer.text()[range].to_string())
    }

    fn selected_text_range(
        &mut self,
        _ignore_disabled_input: bool,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<UTF16Selection> {
        Some(UTF16Selection {
            range: self.buffer.range_to_utf16(&self.buffer.selection()),
            reversed: self.buffer.is_reversed(),
        })
    }

    fn marked_text_range(
        &self,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<Range<usize>> {
        self.buffer
            .marked_utf8()
            .map(|range| self.buffer.range_to_utf16(&range))
    }

    fn unmark_text(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        let was_composing = self.buffer.is_composing();
        self.buffer.marked = None;
        if was_composing {
            self.emit_changed(cx);
        }
    }

    fn replace_text_in_range(
        &mut self,
        range_utf16: Option<Range<usize>>,
        new_text: &str,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.buffer.replace(range_utf16, &single_line(new_text));
        self.emit_changed(cx);
        cx.notify();
    }

    fn replace_and_mark_text_in_range(
        &mut self,
        range_utf16: Option<Range<usize>>,
        new_text: &str,
        new_selected_range_utf16: Option<Range<usize>>,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.buffer.replace_and_mark(
            range_utf16,
            &single_line(new_text),
            new_selected_range_utf16,
        );
        cx.notify();
    }

    fn bounds_for_range(
        &mut self,
        range_utf16: Range<usize>,
        bounds: Bounds<Pixels>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<Bounds<Pixels>> {
        let last_layout = self.last_layout.as_ref()?;
        let range = self.buffer.range_from_utf16(&range_utf16);
        Some(Bounds::from_corners(
            point(
                bounds.left() + last_layout.x_for_index(range.start),
                bounds.top(),
            ),
            point(
                bounds.left() + last_layout.x_for_index(range.end),
                bounds.bottom(),
            ),
        ))
    }

    fn character_index_for_point(
        &mut self,
        point: gpui::Point<Pixels>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<usize> {
        let line_point = self.last_bounds?.localize(&point)?;
        let last_layout = self.last_layout.as_ref()?;
        if last_layout.text != self.buffer.text() {
            // The placeholder is what was shaped; there is no text to hit.
            return None;
        }
        let utf8_index = last_layout.index_for_x(point.x - line_point.x)?;
        Some(self.buffer.offset_to_utf16(utf8_index))
    }
}

struct TextElement {
    input: Entity<SearchInput>,
}

struct PrepaintState {
    line: Option<ShapedLine>,
    cursor: Option<PaintQuad>,
    selection: Option<PaintQuad>,
}

impl IntoElement for TextElement {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

impl Element for TextElement {
    type RequestLayoutState = ();
    type PrepaintState = PrepaintState;

    fn id(&self) -> Option<ElementId> {
        None
    }

    fn source_location(&self) -> Option<&'static core::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&gpui::InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let mut style = Style::default();
        style.size.width = relative(1.).into();
        style.size.height = window.line_height().into();
        (window.request_layout(style, [], cx), ())
    }

    fn prepaint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&gpui::InspectorElementId>,
        bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        window: &mut Window,
        cx: &mut App,
    ) -> Self::PrepaintState {
        let input = self.input.read(cx);
        let content: SharedString = input.buffer.text().to_string().into();
        let selected_range = input.buffer.selection();
        let cursor = input.buffer.cursor();
        let colors = input.colors;
        let style = window.text_style();

        let (display_text, text_color) = if content.is_empty() {
            (input.placeholder.clone(), colors.placeholder)
        } else {
            (content, colors.text)
        };

        let run = TextRun {
            len: display_text.len(),
            font: style.font(),
            color: text_color,
            background_color: None,
            underline: None,
            strikethrough: None,
        };
        let runs = if let Some(marked_range) = input.buffer.marked_utf8() {
            vec![
                TextRun {
                    len: marked_range.start,
                    ..run.clone()
                },
                TextRun {
                    len: marked_range.end - marked_range.start,
                    underline: Some(UnderlineStyle {
                        color: Some(run.color),
                        thickness: px(1.0),
                        wavy: false,
                    }),
                    ..run.clone()
                },
                TextRun {
                    len: display_text.len() - marked_range.end,
                    ..run
                },
            ]
            .into_iter()
            .filter(|run| run.len > 0)
            .collect()
        } else {
            vec![run]
        };

        let font_size = style.font_size.to_pixels(window.rem_size());
        let line = window
            .text_system()
            .shape_line(display_text, font_size, &runs, None);

        let cursor_pos = line.x_for_index(cursor);
        let (selection, cursor) = if selected_range.is_empty() {
            (
                None,
                Some(fill(
                    Bounds::new(
                        point(bounds.left() + cursor_pos, bounds.top()),
                        size(px(1.5), bounds.bottom() - bounds.top()),
                    ),
                    colors.cursor,
                )),
            )
        } else {
            (
                Some(fill(
                    Bounds::from_corners(
                        point(
                            bounds.left() + line.x_for_index(selected_range.start),
                            bounds.top(),
                        ),
                        point(
                            bounds.left() + line.x_for_index(selected_range.end),
                            bounds.bottom(),
                        ),
                    ),
                    colors.selection,
                )),
                None,
            )
        };
        PrepaintState {
            line: Some(line),
            cursor,
            selection,
        }
    }

    fn paint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&gpui::InspectorElementId>,
        bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        prepaint: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        let focus_handle = self.input.read(cx).focus_handle.clone();
        window.handle_input(
            &focus_handle,
            ElementInputHandler::new(bounds, self.input.clone()),
            cx,
        );
        if let Some(selection) = prepaint.selection.take() {
            window.paint_quad(selection)
        }
        let Some(line) = prepaint.line.take() else {
            return;
        };
        let _ = line.paint(
            bounds.origin,
            window.line_height(),
            gpui::TextAlign::Left,
            None,
            window,
            cx,
        );

        if focus_handle.is_focused(window) {
            if let Some(cursor) = prepaint.cursor.take() {
                window.paint_quad(cursor);
            }
        }

        self.input.update(cx, |input, _cx| {
            input.last_layout = Some(line);
            input.last_bounds = Some(bounds);
        });
    }
}

impl Render for SearchInput {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .id("search-input")
            .role(Role::TextInput)
            .aria_label("搜索主题")
            .flex_1()
            .min_w(px(0.))
            .overflow_hidden()
            .key_context(KEY_CONTEXT)
            .track_focus(&self.focus_handle(cx))
            .cursor(CursorStyle::IBeam)
            .on_action(cx.listener(Self::backspace))
            .on_action(cx.listener(Self::delete))
            .on_action(cx.listener(Self::left))
            .on_action(cx.listener(Self::right))
            .on_action(cx.listener(Self::select_left))
            .on_action(cx.listener(Self::select_right))
            .on_action(cx.listener(Self::select_all))
            .on_action(cx.listener(Self::home))
            .on_action(cx.listener(Self::end))
            .on_action(cx.listener(Self::show_character_palette))
            .on_action(cx.listener(Self::paste))
            .on_action(cx.listener(Self::cut))
            .on_action(cx.listener(Self::copy))
            .on_mouse_down(MouseButton::Left, cx.listener(Self::on_mouse_down))
            .on_mouse_up(MouseButton::Left, cx.listener(Self::on_mouse_up))
            .on_mouse_up_out(MouseButton::Left, cx.listener(Self::on_mouse_up))
            .on_mouse_move(cx.listener(Self::on_mouse_move))
            .line_height(px(18.))
            .text_size(px(14.))
            .child(TextElement { input: cx.entity() })
    }
}

impl Focusable for SearchInput {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn buffer(text: &str) -> TextBuffer {
        let mut buffer = TextBuffer::default();
        buffer.set_text(text);
        buffer
    }

    #[test]
    fn typing_chinese_advances_the_cursor_by_bytes() {
        let mut buffer = TextBuffer::default();
        buffer.replace(None, "你好");
        assert_eq!(buffer.text(), "你好");
        assert_eq!(buffer.selection(), 6..6);
    }

    #[test]
    fn backspace_and_delete_remove_whole_characters() {
        let mut buffer = buffer("你好a");
        assert!(buffer.backspace());
        assert_eq!(buffer.text(), "你好");
        assert!(buffer.backspace());
        assert_eq!(buffer.text(), "你");
        buffer.move_to(0);
        assert!(buffer.delete());
        assert_eq!(buffer.text(), "");
        assert!(!buffer.backspace());
        assert!(!buffer.delete());
    }

    #[test]
    fn cursor_moves_by_character_boundaries() {
        let mut buffer = buffer("a你b");
        buffer.move_to(buffer.prev_boundary(buffer.cursor()));
        assert_eq!(buffer.cursor(), 4);
        buffer.move_to(buffer.prev_boundary(buffer.cursor()));
        assert_eq!(buffer.cursor(), 1);
        buffer.move_to(buffer.next_boundary(buffer.cursor()));
        assert_eq!(buffer.cursor(), 4);
        buffer.move_to(buffer.next_boundary(buffer.cursor()));
        buffer.move_to(buffer.next_boundary(buffer.cursor()));
        assert_eq!(buffer.cursor(), 5, "cannot move past the end");
    }

    #[test]
    fn selecting_backwards_keeps_the_range_ordered() {
        let mut buffer = buffer("hello");
        buffer.move_to(3);
        buffer.select_to(1);
        assert_eq!(buffer.selection(), 1..3);
        assert_eq!(buffer.cursor(), 1);
        buffer.select_to(5);
        assert_eq!(buffer.selection(), 3..5);
        assert_eq!(buffer.selected_text(), "lo");
    }

    #[test]
    fn typing_over_a_selection_replaces_it() {
        let mut buffer = buffer("hello world");
        buffer.select_all();
        assert_eq!(buffer.selected_text(), "hello world");
        buffer.replace(None, "主题");
        assert_eq!(buffer.text(), "主题");
        assert_eq!(buffer.selection(), 6..6);
    }

    #[test]
    fn input_method_composition_replaces_the_marked_text() {
        let mut buffer = TextBuffer::default();
        buffer.replace_and_mark(None, "ni", Some(2..2));
        assert_eq!(buffer.text(), "ni");
        assert_eq!(buffer.marked_utf8(), Some(0..2));

        buffer.replace_and_mark(None, "nihao", Some(5..5));
        assert_eq!(buffer.text(), "nihao");
        assert_eq!(buffer.marked_utf8(), Some(0..5));

        buffer.replace(None, "你好");
        assert_eq!(buffer.text(), "你好");
        assert_eq!(buffer.marked_utf8(), None);
        assert_eq!(buffer.selection(), 6..6);
    }

    #[test]
    fn composition_keeps_text_typed_before_it() {
        let mut buffer = buffer("豆");
        buffer.replace_and_mark(None, "bao", Some(3..3));
        assert_eq!(buffer.text(), "豆bao");
        buffer.replace(None, "包");
        assert_eq!(buffer.text(), "豆包");
    }

    #[test]
    fn utf16_offsets_round_trip_through_surrogate_pairs() {
        let buffer = buffer("a😀b");
        assert_eq!(buffer.offset_to_utf16(0), 0);
        assert_eq!(buffer.offset_to_utf16(1), 1);
        assert_eq!(buffer.offset_to_utf16(5), 3);
        assert_eq!(buffer.offset_to_utf16(6), 4);
        assert_eq!(buffer.offset_from_utf16(3), 5);
        assert_eq!(buffer.range_to_utf16(&(1..5)), 1..3);
        assert_eq!(buffer.range_from_utf16(&(1..3)), 1..5);
    }

    #[test]
    fn replacing_an_explicit_utf16_range_edits_the_right_bytes() {
        let mut buffer = buffer("你好吗");
        buffer.replace(Some(1..2), "们");
        assert_eq!(buffer.text(), "你们吗");
    }

    #[test]
    fn pasted_text_is_flattened_to_a_single_line() {
        assert_eq!(single_line("a\nb\r\nc\td"), "a b c d");
        assert_eq!(single_line("  keep  "), "  keep  ");
    }
}
