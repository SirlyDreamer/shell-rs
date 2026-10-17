use gpui_kit::component::{
    ActiveTheme as _, Icon, Selectable as _, Sizable as _,
    button::{Button, ButtonGroup, ButtonVariants as _},
    dock::{BasePanel, Panel, PanelControl, PanelEvent},
    h_flex,
    menu::PopupMenu,
    tooltip::Tooltip,
    v_flex,
};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use crate::app::{CatalogIcon, OpenSettings, ShowCredentials, ShowForwards, ShowHosts};
use crate::credential::CredentialPanel;
use crate::forward::{ForwardManager, ForwardPanel};
use crate::host::{HostPanel, import_menu_items};
use crate::i18n::t;

/// Which list the left dock is showing.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SidebarMode {
    #[default]
    Hosts,
    Forwards,
    Credentials,
}

/// The left dock's one panel: the host tree, the port-forwarding list or
/// the credential list, whichever its header's switch picked, over the
/// 设置 button they share.
///
/// One panel showing either list rather than two panels swapped in the dock:
/// swapping would take the displaced list out of the dock (and the focus
/// with it), and two panels in one tab group always draw a tab bar. Every
/// list stays alive while hidden, so each keeps its search, its selection
/// and its scroll position.
pub struct Sidebar {
    mode: SidebarMode,
    hosts: Entity<HostPanel>,
    forwards: Entity<ForwardPanel>,
    credentials: Entity<CredentialPanel>,
    manager: Entity<ForwardManager>,
    target: FocusHandle,
    _forward_subscription: Subscription,
}

impl Sidebar {
    pub fn new(
        hosts: Entity<HostPanel>,
        forwards: Entity<ForwardPanel>,
        credentials: Entity<CredentialPanel>,
        manager: Entity<ForwardManager>,
        target: FocusHandle,
        cx: &mut Context<Self>,
    ) -> Self {
        let subscription = cx.observe(&manager, |_, _, cx| cx.notify());
        Self {
            mode: SidebarMode::default(),
            hosts,
            forwards,
            credentials,
            manager,
            target,
            _forward_subscription: subscription,
        }
    }

    pub fn mode(&self) -> SidebarMode {
        self.mode
    }

    /// Show one of the lists and give it the focus. The list that was
    /// showing leaves the element tree, and a focus left behind on it would
    /// take every window-level shortcut's dispatch path with it.
    pub fn show(&mut self, mode: SidebarMode, window: &mut Window, cx: &mut Context<Self>) {
        self.mode = mode;
        let handle = self.focus_handle(cx);
        window.focus(&handle, cx);
        cx.notify();
    }

    /// Put the keyboard in the search field of the list being shown.
    pub fn focus_search(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        match self.mode {
            SidebarMode::Hosts => self
                .hosts
                .update(cx, |panel, cx| panel.focus_search(window, cx)),
            SidebarMode::Forwards => self
                .forwards
                .update(cx, |panel, cx| panel.focus_search(window, cx)),
            SidebarMode::Credentials => self
                .credentials
                .update(cx, |panel, cx| panel.focus_search(window, cx)),
        }
    }
}

impl EventEmitter<PanelEvent> for Sidebar {}

impl Focusable for Sidebar {
    fn focus_handle(&self, cx: &App) -> FocusHandle {
        match self.mode {
            SidebarMode::Hosts => self.hosts.read(cx).focus_handle(cx),
            SidebarMode::Forwards => self.forwards.read(cx).focus_handle(cx),
            SidebarMode::Credentials => self.credentials.read(cx).focus_handle(cx),
        }
    }
}

impl BasePanel for Sidebar {
    fn panel_name(&self) -> &'static str {
        "Sidebar"
    }

    fn closable(&self, _: &App) -> bool {
        false
    }

    fn zoomable(&self, _: &App) -> bool {
        false
    }
}

/// The dock's title bar belongs to whichever list is showing.
impl Panel for Sidebar {
    fn title(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let target = self.target.clone();
        let forwards = self.manager.read(cx).active_count();
        let label = match self.mode {
            SidebarMode::Hosts => t!("host.panel.title"),
            SidebarMode::Forwards => t!("forward.panel.title"),
            SidebarMode::Credentials => t!("credential.panel.title"),
        };
        h_flex()
            .id("sidebar-header")
            .test_support()
            .w_full()
            .min_w_0()
            .gap_2()
            // The mode controls belong to the list they change. Keep their
            // hit targets intact; the heading can truncate in a narrow dock.
            .child(
                h_flex()
                    .flex_shrink_0()
                    // Clicking a mode must not begin dragging the dock panel.
                    .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
                    .child(
                        ButtonGroup::new("sidebar-mode")
                            .ghost()
                            .xsmall()
                            .flex_shrink_0()
                            .child(
                                Button::new("show-hosts")
                                    .icon(CatalogIcon::Server)
                                    .tooltip(t!("workspace.title_bar.hosts"))
                                    .accessibility_label(t!("workspace.title_bar.hosts"))
                                    .selected(self.mode == SidebarMode::Hosts),
                            )
                            .child(
                                Button::new("show-forwards")
                                    .icon(CatalogIcon::ArrowLeftRight)
                                    .tooltip(t!("workspace.title_bar.forwards"))
                                    .selected(self.mode == SidebarMode::Forwards)
                                    .map(|button| match forwards {
                                        0 => button.accessibility_label(t!(
                                            "workspace.title_bar.forwards"
                                        )),
                                        running => button
                                            .label(running.to_string())
                                            .accessibility_label(t!(
                                                "workspace.title_bar.forwards_running",
                                                count = running
                                            )),
                                    }),
                            )
                            .child(
                                Button::new("show-credentials")
                                    .icon(CatalogIcon::KeyRound)
                                    .tooltip(t!("workspace.title_bar.credentials"))
                                    .accessibility_label(t!("workspace.title_bar.credentials"))
                                    .selected(self.mode == SidebarMode::Credentials),
                            )
                            .on_click(move |picked: &Vec<usize>, window, cx| {
                                match picked.first() {
                                    Some(0) => target.dispatch_action(&ShowHosts, window, cx),
                                    Some(1) => target.dispatch_action(&ShowForwards, window, cx),
                                    Some(2) => target.dispatch_action(&ShowCredentials, window, cx),
                                    _ => {}
                                }
                            }),
                    ),
            )
            .child(
                div()
                    .id("sidebar-heading")
                    .test_support()
                    .min_w_0()
                    .flex_1()
                    .text_ellipsis()
                    .tooltip({
                        let label = label.clone();
                        move |window, cx| Tooltip::new(label.clone()).build(window, cx)
                    })
                    .child(label),
            )
    }

    fn toolbar_buttons(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Option<Vec<Button>> {
        match self.mode {
            SidebarMode::Hosts => self
                .hosts
                .update(cx, |panel, cx| panel.toolbar_buttons(window, cx)),
            SidebarMode::Forwards => Some(self.forwards.read(cx).toolbar_buttons()),
            SidebarMode::Credentials => Some(self.credentials.read(cx).toolbar_buttons()),
        }
    }

    /// The dock's 「…」 menu: the host list's import commands, which are
    /// used too rarely for a toolbar button of their own.
    fn dropdown_menu(
        &mut self,
        menu: PopupMenu,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> PopupMenu {
        match self.mode {
            SidebarMode::Hosts => import_menu_items(menu),
            SidebarMode::Forwards | SidebarMode::Credentials => menu,
        }
    }

    fn zoom_control(&self, _: &App) -> Option<PanelControl> {
        None
    }

    fn inner_padding(&self, _: &App) -> bool {
        false
    }
}

impl Render for Sidebar {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let list = match self.mode {
            SidebarMode::Hosts => self.hosts.clone().into_any_element(),
            SidebarMode::Forwards => self.forwards.clone().into_any_element(),
            SidebarMode::Credentials => self.credentials.clone().into_any_element(),
        };
        v_flex()
            .size_full()
            .bg(cx.theme().sidebar)
            .text_color(cx.theme().sidebar_foreground)
            .child(div().flex_1().min_h_0().child(list))
            .child(
                // Pinned under the list, where desktop apps keep settings.
                h_flex()
                    .p_2()
                    .border_t_1()
                    .border_color(cx.theme().sidebar_border)
                    .child(
                        Button::new("open-settings")
                            .ghost()
                            .small()
                            .icon(Icon::new(CatalogIcon::Settings))
                            .label(t!("workspace.sidebar.settings"))
                            .tooltip_with_action(
                                t!("workspace.sidebar.open_settings"),
                                &OpenSettings,
                                None,
                            )
                            .on_click(|_, window, cx| {
                                window.dispatch_action(Box::new(OpenSettings), cx)
                            }),
                    ),
            )
    }
}
