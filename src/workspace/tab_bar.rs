//! Compact center tabs. Base keeps the tab semantics and Dock keeps all
//! selection and drag operations; this skin only supplies their presentation.

use std::{cell::Cell, sync::Arc};

use gpui_kit::base::{InteractiveElementExt as _, Tab, Tabs};
use gpui_kit::component::{
    ActiveTheme as _, IconName, Selectable as _, Sizable as _,
    button::{Button, ButtonVariants as _},
    dock::{
        AnyDrag, BasePanelView, ClosePanel, DragPanel, PanelHandle, PanelId, TabGroupContext,
        ToggleZoom,
    },
    h_flex,
    menu::DropdownMenu as _,
};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use crate::i18n::t;

/// One instance per group, so splitting and rearranging preserve its scroll.
#[derive(Default)]
pub(super) struct CenterTabBar {
    scroll: ScrollHandle,
    displayed: Cell<Option<PanelId>>,
}

impl CenterTabBar {
    pub(super) fn render(
        &self,
        group: &TabGroupContext,
        controls: Option<AnyElement>,
        window: &mut Window,
        cx: &mut App,
    ) -> AnyElement {
        let displayed = group.active_panel().map(|panel| panel.panel_id(cx));
        let visible: Vec<_> = group
            .panels()
            .iter()
            .enumerate()
            .filter(|(_, panel)| panel.visible(cx))
            .collect();
        if self.displayed.replace(displayed) != displayed
            && let Some(ix) = visible
                .iter()
                .position(|(_, panel)| Some(panel.panel_id(cx)) == displayed)
        {
            self.scroll.scroll_to_item(ix);
        }
        let count = visible.len();
        let tabs = visible
            .into_iter()
            .enumerate()
            .map(|(position, (ix, panel))| {
                let selected = Some(panel.panel_id(cx)) == displayed;
                let focused = selected && panel.focus_handle(cx).contains_focused(window, cx);
                let theme = cx.theme();
                let tab = Tab::new(("center-tab", panel.panel_id(cx).as_u64()))
                    .selected(selected)
                    .set_position(position + 1, count)
                    .flex()
                    .items_center()
                    .justify_center()
                    .line_height(relative(1.))
                    .h_7()
                    .max_w_64()
                    .flex_shrink_0()
                    .px_2()
                    .border_1()
                    .border_color(theme.transparent)
                    .rounded(theme.radius)
                    .text_sm()
                    .text_color(theme.muted_foreground)
                    .styles(|styles| {
                        styles.selected(|style| {
                            style
                                .bg(theme.tokens.tab_active)
                                .text_color(theme.foreground)
                                .border_color(if focused { theme.ring } else { theme.border })
                        })
                    })
                    .when(!selected, |tab| {
                        tab.hover(|style| style.bg(theme.accent).text_color(theme.foreground))
                    })
                    .child(title(panel, window, cx))
                    .on_click({
                        let group = group.clone();
                        move |_, window, cx| group.select_tab(ix, window, cx)
                    })
                    .when_some(
                        group
                            .is_draggable()
                            .then(|| group.drag_panel(ix, cx))
                            .flatten(),
                        |tab, drag| {
                            tab.on_drag(drag, {
                                let panel = panel.clone();
                                move |drag, offset, window, cx| {
                                    cx.stop_propagation();
                                    drag.set_drag_offset(offset);
                                    // Drag geometry is in logical pixels; scale it with the same rem as the tab.
                                    drag.set_preview_size(size(
                                        window.rem_size() * 16.,
                                        window.rem_size() * 1.75,
                                    ));
                                    cx.new(|_| TabPreview {
                                        panel: panel.clone(),
                                    })
                                }
                            })
                        },
                    )
                    .when(group.is_droppable(), |tab| {
                        tab.drag_over::<DragPanel>(|style, _, _, cx| {
                            style.border_color(cx.theme().drag_border)
                        })
                        .on_drop({
                            let group = group.clone();
                            move |drag: &DragPanel, window, cx| {
                                group.drop_panel(drag.clone(), Some(ix), true, window, cx)
                            }
                        })
                        .drag_over::<AnyDrag>(|style, _, _, cx| {
                            style.border_color(cx.theme().drag_border)
                        })
                        .on_drop({
                            let group = group.clone();
                            move |item: &AnyDrag, window, cx| {
                                group.drop_item(item.clone(), None, window, cx)
                            }
                        })
                    });
                tab.into_any_element()
            })
            .collect::<Vec<_>>();
        let trailing = div()
            .id("tab-bar-empty-space")
            .h_7()
            .min_w_8()
            .flex_1()
            .when(group.is_droppable(), |space| {
                space
                    .drag_over::<DragPanel>(|style, _, _, cx| {
                        style.bg(cx.theme().tokens.drop_target)
                    })
                    .on_drop({
                        let group = group.clone();
                        move |drag: &DragPanel, window, cx| {
                            let ix =
                                (drag.source() == group.node()).then(|| group.panels().len() - 1);
                            group.drop_panel(drag.clone(), ix, false, window, cx);
                        }
                    })
                    .drag_over::<AnyDrag>(|style, _, _, cx| style.bg(cx.theme().tokens.drop_target))
                    .on_drop({
                        let group = group.clone();
                        move |item: &AnyDrag, window, cx| {
                            group.drop_item(item.clone(), None, window, cx)
                        }
                    })
            });
        h_flex()
            .id(("center-tab-bar", group.node().as_u64()))
            .test_support()
            .w_full()
            .h_8()
            .flex_shrink_0()
            .px_1()
            .gap_1()
            .border_b_1()
            .border_color(cx.theme().border)
            .bg(cx.theme().tokens.tab_bar)
            .children(controls)
            .child(
                Tabs::new("center-tabs")
                    .flex()
                    .flex_1()
                    .min_w_0()
                    .overflow_hidden()
                    .child(
                        h_flex()
                            .id("tabs-scroll")
                            .w_full()
                            .min_w_0()
                            .gap_1()
                            .overflow_x_scroll()
                            .lock_scroll_axis()
                            .track_scroll(&self.scroll)
                            .children(tabs)
                            .child(trailing),
                    ),
            )
            .child(toolbar(group, window, cx))
            .into_any_element()
    }
}

fn title(panel: &Arc<dyn BasePanelView>, window: &mut Window, cx: &mut App) -> AnyElement {
    match PanelHandle::of(panel) {
        Some(handle) => handle.title(window, cx),
        None => SharedString::from(panel.panel_name(cx)).into_any_element(),
    }
}

fn toolbar(group: &TabGroupContext, window: &mut Window, cx: &mut App) -> impl IntoElement {
    let handle = group.active_panel().and_then(PanelHandle::of);
    let control = group
        .active_panel()
        .filter(|panel| panel.zoomable(cx))
        .and_then(|_| handle.and_then(|handle| handle.zoom_control(cx)));
    let zoomed = group.is_zoomed();
    let zoom_label = if zoomed {
        t!("workspace.tabs.zoom_out")
    } else {
        t!("workspace.tabs.zoom_in")
    };
    let menu_zoom = control.is_some_and(|control| control.menu_visible());
    let toolbar_zoom = zoomed || control.is_some_and(|control| control.toolbar_visible());
    let closable = group.is_closable();
    let panel = handle.map(|handle| handle.panel());
    h_flex()
        .flex_shrink_0()
        .gap_1()
        .occlude()
        .children(handle.and_then(|handle| handle.title_suffix(window, cx)))
        .children(
            handle
                .and_then(|handle| handle.toolbar_buttons(window, cx))
                .unwrap_or_default()
                .into_iter()
                .map(|button| button.xsmall().ghost().tab_stop(false)),
        )
        .when(toolbar_zoom, |toolbar| {
            toolbar.child(
                Button::new(if zoomed { "zoom-out" } else { "zoom-in" })
                    .icon(if zoomed {
                        IconName::Minimize
                    } else {
                        IconName::Maximize
                    })
                    .xsmall()
                    .ghost()
                    .tab_stop(false)
                    .selected(zoomed)
                    .tooltip_with_action(zoom_label.clone(), &ToggleZoom, None)
                    .on_click({
                        let group = group.clone();
                        move |_, window, cx| group.toggle_zoom(window, cx)
                    }),
            )
        })
        .child(
            Button::new("menu")
                .icon(IconName::Ellipsis)
                .xsmall()
                .ghost()
                .tab_stop(false)
                .tooltip(t!("workspace.tabs.actions"))
                .dropdown_menu(move |menu, window, cx| {
                    menu.when_some(panel.clone(), |menu, panel| {
                        panel.dropdown_menu(menu, window, cx)
                    })
                    .separator()
                    .menu_with_disabled(
                        zoom_label.clone(),
                        Box::new(ToggleZoom),
                        !menu_zoom && !zoomed,
                    )
                    .when(closable, |menu| {
                        menu.separator()
                            .menu(t!("common.close"), Box::new(ClosePanel))
                    })
                })
                .anchor(Anchor::TopRight),
        )
}

struct TabPreview {
    panel: Arc<dyn BasePanelView>,
}

impl Render for TabPreview {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        h_flex()
            .w_64()
            .h_7()
            .px_2()
            .text_sm()
            .border_1()
            .border_color(cx.theme().border)
            .rounded(cx.theme().radius)
            .bg(cx.theme().tokens.tab_active)
            .text_color(cx.theme().foreground)
            .opacity(0.75)
            .overflow_hidden()
            .child(title(&self.panel, window, cx))
    }
}
