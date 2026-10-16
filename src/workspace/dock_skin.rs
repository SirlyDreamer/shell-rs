use std::{
    cell::{Cell, RefCell},
    rc::Rc,
    sync::Arc,
};

use super::tab_bar::CenterTabBar;

use crate::app::{CatalogIcon, ToggleTerminalBroadcast};
use crate::i18n::t;
use gpui_kit::component::dock::{
    BasePanelView, DockArea, DockAreaRenderer, DockContext, DockSkin, DropIndicator, NodeId,
    PanelState, TabGroupContext, TabGroupRenderer, TilesRenderer,
};
use gpui_kit::component::{
    ActiveTheme as _, Icon, Selectable as _, Sizable as _,
    button::{Button, ButtonVariants as _},
    h_flex, v_flex,
};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

/// Renderers consume this value, never the Workspace entity being rendered.
#[derive(Clone, Copy, Default, PartialEq, Eq)]
struct BroadcastBar {
    node: Option<NodeId>,
    enabled: bool,
    count: usize,
}

impl BroadcastBar {
    fn controls(self, target: &FocusHandle) -> impl IntoElement {
        let broadcast = target.clone();
        let label = if !self.enabled {
            t!("workspace.broadcast.title")
        } else if self.count < 2 {
            t!("workspace.broadcast.choose")
        } else {
            t!("workspace.broadcast.active", count = self.count)
        };
        h_flex().flex_shrink_0().pl_1().child(
            Button::new("toggle-terminal-broadcast")
                .ghost()
                .xsmall()
                .icon(Icon::new(CatalogIcon::Cast))
                .selected(self.enabled)
                .accessibility_label(label)
                .tooltip(t!("workspace.broadcast.title"))
                .on_click(move |_, window, cx| {
                    broadcast.dispatch_action(&ToggleTerminalBroadcast, window, cx)
                }),
        )
    }
}

/// The workspace's dock appearance: gpui-kit's `DockSkin`, plus a view that
/// fills the center while it holds no tab.
///
/// The dock draws nothing for an empty center and `DockAreaRenderer` has no
/// empty-center hook, so the placeholder rides on `center_frame`. It is drawn
/// deferred, above the empty layout the dock paints there, whenever the
/// workspace last said the center is empty (`set_center_empty`); the
/// workspace keeps that in step with `DockEvent::LayoutChanged`.
pub(super) struct WorkspaceDockSkin {
    skin: Rc<DockSkin>,
    center_groups: Rc<RefCell<Vec<NodeId>>>,
    broadcast: Rc<Cell<BroadcastBar>>,
    target: FocusHandle,
    area: WeakEntity<DockArea>,
    placeholder: AnyView,
    center_empty: Cell<bool>,
}

impl WorkspaceDockSkin {
    /// Build a `DockArea` wearing this skin, together with the handle its
    /// settings are changed through. `placeholder` is what the center shows
    /// while it is empty.
    pub(super) fn dock_area(
        id: &'static str,
        version: Option<usize>,
        placeholder: AnyView,
        target: FocusHandle,
        window: &mut Window,
        cx: &mut App,
    ) -> (Entity<DockArea>, Rc<Self>) {
        let mut skin = None;
        let area = cx.new(|cx| {
            let this = Rc::new(Self {
                skin: DockSkin::new(cx),
                center_groups: Rc::default(),
                broadcast: Rc::default(),
                target,
                area: cx.weak_entity(),
                placeholder,
                center_empty: Cell::new(false),
            });
            skin = Some(this.clone());
            DockArea::new(id, version, window, cx).with_renderer(this)
        });
        // The closure above runs before `cx.new` returns.
        let skin = skin.expect("the skin is built inside the constructor");
        (area, skin)
    }

    pub(super) fn is_center_empty(&self) -> bool {
        self.center_empty.get()
    }

    pub(super) fn set_broadcast(
        &self,
        groups: Vec<NodeId>,
        node: Option<NodeId>,
        enabled: bool,
        count: usize,
        cx: &mut App,
    ) {
        let state = BroadcastBar {
            node,
            enabled,
            count,
        };
        let groups_changed = *self.center_groups.borrow() != groups;
        if groups_changed {
            self.center_groups.replace(groups);
        }
        if self.broadcast.replace(state) != state || groups_changed {
            _ = self.area.update(cx, |_, cx| cx.notify());
        }
    }

    /// Show (`true`) or hide the placeholder. The skin is not an entity, so
    /// it redraws the area itself when the value changes.
    pub(super) fn set_center_empty(&self, empty: bool, cx: &mut App) {
        if self.center_empty.replace(empty) != empty {
            _ = self.area.update(cx, |_, cx| cx.notify());
        }
    }
}

impl DockAreaRenderer for WorkspaceDockSkin {
    fn frame(&self, window: &mut Window, cx: &mut App) -> Stateful<Div> {
        self.skin.frame(window, cx)
    }

    fn split_frame(
        &self,
        node: NodeId,
        axis: Axis,
        window: &mut Window,
        cx: &mut App,
    ) -> Stateful<Div> {
        self.skin.split_frame(node, axis, window, cx)
    }

    fn center_frame(&self, window: &mut Window, cx: &mut App) -> Stateful<Div> {
        self.skin
            .center_frame(window, cx)
            .relative()
            .when(self.center_empty.get(), |frame| {
                // Deferred, so it paints after the empty layout's own
                // background, which the dock draws later in the same frame.
                frame.child(deferred(
                    v_flex()
                        .absolute()
                        .inset_0()
                        .child(
                            h_flex()
                                .h_8()
                                .flex_shrink_0()
                                .bg(cx.theme().tokens.tab_bar)
                                .border_b_1()
                                .border_color(cx.theme().border)
                                .child(self.broadcast.get().controls(&self.target)),
                        )
                        .child(div().flex_1().min_h_0().child(self.placeholder.clone())),
                ))
            })
    }

    fn render_dock(
        &self,
        dock: &DockContext,
        content: AnyElement,
        window: &mut Window,
        cx: &mut App,
    ) -> AnyElement {
        self.skin.render_dock(dock, content, window, cx)
    }

    fn build_placeholder(
        &self,
        state: &PanelState,
        window: &mut Window,
        cx: &mut App,
    ) -> Option<Arc<dyn BasePanelView>> {
        self.skin.build_placeholder(state, window, cx)
    }

    fn tab_group_renderer(&self) -> Rc<dyn TabGroupRenderer> {
        Rc::new(WorkspaceTabBar {
            skin: self.skin.tab_group_renderer(),
            center_groups: self.center_groups.clone(),
            tabs: CenterTabBar::default(),
            broadcast: self.broadcast.clone(),
            target: self.target.clone(),
        })
    }

    fn tiles_renderer(&self) -> Rc<dyn TilesRenderer> {
        self.skin.tiles_renderer()
    }
}

/// Side docks keep the library skin; every center group shares one tab style.
/// Dock owns selection, dragging, splitting and focus in both cases.
struct WorkspaceTabBar {
    skin: Rc<dyn TabGroupRenderer>,
    center_groups: Rc<RefCell<Vec<NodeId>>>,
    tabs: CenterTabBar,
    broadcast: Rc<Cell<BroadcastBar>>,
    target: FocusHandle,
}

impl TabGroupRenderer for WorkspaceTabBar {
    fn frame(&self, group: &TabGroupContext, window: &mut Window, cx: &mut App) -> Stateful<Div> {
        self.skin.frame(group, window, cx)
    }
    fn content_frame(
        &self,
        group: &TabGroupContext,
        window: &mut Window,
        cx: &mut App,
    ) -> Stateful<Div> {
        self.skin.content_frame(group, window, cx)
    }
    fn render_tab_bar(
        &self,
        group: &TabGroupContext,
        window: &mut Window,
        cx: &mut App,
    ) -> AnyElement {
        let state = self.broadcast.get();
        if self.center_groups.borrow().contains(&group.node()) {
            let controls = (state.node == Some(group.node()))
                .then(|| state.controls(&self.target).into_any_element());
            return self.tabs.render(group, controls, window, cx);
        }
        h_flex()
            .w_full()
            .h_8()
            .flex_shrink_0()
            .text_sm()
            .bg(cx.theme().tokens.tab_bar)
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .child(self.skin.render_tab_bar(group, window, cx)),
            )
            .into_any_element()
    }
    fn render_active_panel(
        &self,
        panel: AnyView,
        group: &TabGroupContext,
        window: &mut Window,
        cx: &mut App,
    ) -> AnyElement {
        self.skin.render_active_panel(panel, group, window, cx)
    }
    fn render_drop_indicator(
        &self,
        indicator: DropIndicator,
        window: &mut Window,
        cx: &mut App,
    ) -> Option<AnyElement> {
        self.skin.render_drop_indicator(indicator, window, cx)
    }
    fn render_empty(
        &self,
        group: &TabGroupContext,
        window: &mut Window,
        cx: &mut App,
    ) -> Option<AnyElement> {
        self.skin.render_empty(group, window, cx)
    }
}
