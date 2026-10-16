use gpui_kit::component::{
    ActiveTheme as _, IconName, Sizable as _, TitleBar,
    button::{Button, ButtonVariants as _},
    h_flex,
};
use gpui_kit::*;

use crate::app::{
    CatalogIcon, NewLocalTerminal, NewTemporaryConnection, ShowUpdate, ToggleHostPanel, ToggleTheme,
};
use crate::i18n::t;
use crate::shared::tinted;
use crate::update::UpdateBadge;

use super::sidebar::SidebarMode;

/// The custom title bar: the app name and connection commands on the left,
/// appearance and updates on the right. Every button dispatches an action on
/// `target` (the workspace's focus handle) so the workspace handles it even
/// when nothing is focused.
///
/// `sidebar` is what the left dock is showing, or `None` while it is hidden;
/// `update` is the newer ShellRS waiting to be installed, when there is one.
pub fn render_title_bar(
    sidebar: Option<SidebarMode>,
    update: Option<UpdateBadge>,
    target: &FocusHandle,
    cx: &App,
) -> TitleBar {
    let dark = cx.theme().is_dark();
    let (toggle, temporary, new_local, theme, show_update) = (
        target.clone(),
        target.clone(),
        target.clone(),
        target.clone(),
        target.clone(),
    );
    // The title bar drags the window on mouse down; keep clicks on its
    // controls from starting a drag.
    let no_drag = |_: &MouseDownEvent, _: &mut Window, cx: &mut App| cx.stop_propagation();
    TitleBar::new()
        .child(
            h_flex()
                .gap_3()
                .child(
                    div()
                        .text_sm()
                        .font_weight(FontWeight::MEDIUM)
                        .child("ShellRS"),
                )
                .child(
                    h_flex()
                        .id("title-connection-commands")
                        .test_support()
                        .flex_shrink_0()
                        .gap_1()
                        .on_mouse_down(MouseButton::Left, no_drag)
                        .child(
                            Button::new("toggle-hosts")
                                .ghost()
                                .small()
                                .icon(IconName::PanelLeft)
                                .tooltip(if sidebar.is_some() {
                                    t!("workspace.title_bar.hide_sidebar")
                                } else {
                                    t!("workspace.title_bar.show_sidebar")
                                })
                                .on_click(move |_, window, cx| {
                                    toggle.dispatch_action(&ToggleHostPanel, window, cx)
                                }),
                        )
                        .child(
                            // Saved hosts are made in the host list; up here is the
                            // quick way in, to a host that is not saved.
                            Button::new("temporary-connection")
                                .ghost()
                                .small()
                                .icon(CatalogIcon::Plug)
                                .label(t!("workspace.title_bar.quick_connect"))
                                .tooltip(t!("workspace.title_bar.quick_connect_tooltip"))
                                .on_click(move |_, window, cx| {
                                    temporary.dispatch_action(&NewTemporaryConnection, window, cx)
                                }),
                        )
                        .child(
                            Button::new("new-local-terminal")
                                .ghost()
                                .small()
                                .icon(CatalogIcon::Terminal)
                                .label(t!("workspace.title_bar.local_terminal"))
                                .tooltip_with_action(
                                    t!("workspace.title_bar.new_local_terminal"),
                                    &NewLocalTerminal,
                                    None,
                                )
                                .on_click(move |_, window, cx| {
                                    new_local.dispatch_action(&NewLocalTerminal, window, cx)
                                }),
                        ),
                ),
        )
        .child(
            h_flex()
                .justify_end()
                .px_2()
                .gap_1()
                .on_mouse_down(MouseButton::Left, no_drag)
                .children(update.map(|update| {
                    // The one coloured button up here, in the colour of
                    // what it says: the only sign that an update waits.
                    Button::new("update-available")
                        .custom(tinted(
                            if update.trouble {
                                cx.theme().warning
                            } else {
                                cx.theme().success
                            },
                            cx,
                        ))
                        .small()
                        .icon(CatalogIcon::CircleArrowUp)
                        .tooltip(update.label.clone())
                        .accessibility_label(update.label)
                        .on_click(move |_, window, cx| {
                            show_update.dispatch_action(&ShowUpdate, window, cx)
                        })
                }))
                .child(
                    Button::new("theme-toggle")
                        .ghost()
                        .small()
                        .icon(if dark { IconName::Sun } else { IconName::Moon })
                        .tooltip(if dark {
                            t!("workspace.title_bar.to_light")
                        } else {
                            t!("workspace.title_bar.to_dark")
                        })
                        .on_click(move |_, window, cx| {
                            theme.dispatch_action(&ToggleTheme, window, cx)
                        }),
                ),
        )
}
