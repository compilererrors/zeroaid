use gpui::{
    App, Context, Entity, EventEmitter, FocusHandle, Focusable, Pixels, Render, SharedString,
    Subscription, WeakEntity, Window, px,
};
use project::Event as ProjectEvent;
use recent_projects::RecentProjects;
use theme::ActiveTheme;
use ui::utils::TRAFFIC_LIGHT_PADDING;
use ui::{
    ButtonStyle, HighlightedLabel, KeyBinding, ListItem, PopoverMenu, PopoverMenuHandle, TintColor,
    Tooltip, prelude::*,
};
use workspace::{
    FocusWorkspaceSidebar, MultiWorkspace, NewWorkspaceInWindow, Sidebar as WorkspaceSidebar,
    SidebarEvent, ToggleWorkspaceSidebar, Workspace,
};
use zed_actions::OpenRecent;

const DEFAULT_WIDTH: Pixels = px(320.0);
const MIN_WIDTH: Pixels = px(200.0);
const MAX_WIDTH: Pixels = px(800.0);

#[derive(Clone)]
struct WorkspaceEntry {
    label: SharedString,
    full_path: SharedString,
    workspace: Entity<Workspace>,
}

fn workspace_label_and_path(
    workspace: &Entity<Workspace>,
    cx: &App,
) -> (SharedString, SharedString) {
    let workspace_ref = workspace.read(cx);
    let mut paths = Vec::new();
    let mut names = Vec::new();

    for worktree in workspace_ref.worktrees(cx) {
        let worktree_ref = worktree.read(cx);
        if !worktree_ref.is_visible() {
            continue;
        }

        let abs_path = worktree_ref.abs_path();
        paths.push(abs_path.to_path_buf());
        if let Some(name) = abs_path.file_name() {
            names.push(name.to_string_lossy().to_string());
        }
    }

    let label: SharedString = if names.is_empty() {
        "Empty Workspace".into()
    } else {
        names.join(", ").into()
    };

    let full_path: SharedString = paths
        .into_iter()
        .map(|path| path.to_string_lossy().to_string())
        .collect::<Vec<_>>()
        .join("\n")
        .into();

    (label, full_path)
}

pub struct Sidebar {
    multi_workspace: WeakEntity<MultiWorkspace>,
    width: Pixels,
    focus_handle: FocusHandle,
    recent_projects_popover_handle: PopoverMenuHandle<RecentProjects>,
    _subscription: Subscription,
    _project_subscriptions: Vec<Subscription>,
}

impl EventEmitter<SidebarEvent> for Sidebar {}

impl Sidebar {
    pub fn new(
        multi_workspace: Entity<MultiWorkspace>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let subscription = cx.observe_in(&multi_workspace, window, |this, _, window, cx| {
            this._project_subscriptions = this.subscribe_to_projects(window, cx);
            cx.notify();
        });

        let this = Self {
            multi_workspace: multi_workspace.downgrade(),
            width: DEFAULT_WIDTH,
            focus_handle: cx.focus_handle(),
            recent_projects_popover_handle: PopoverMenuHandle::default(),
            _subscription: subscription,
            _project_subscriptions: Vec::new(),
        };
        cx.defer_in(window, |this, window, cx| {
            this._project_subscriptions = this.subscribe_to_projects(window, cx);
            cx.notify();
        });
        this
    }

    fn subscribe_to_projects(
        &self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Vec<Subscription> {
        let Some(multi_workspace) = self.multi_workspace.upgrade() else {
            return Vec::new();
        };

        let projects: Vec<_> = multi_workspace
            .read(cx)
            .workspaces()
            .iter()
            .map(|workspace| workspace.read(cx).project().clone())
            .collect();

        projects
            .iter()
            .map(|project| {
                cx.subscribe_in(
                    project,
                    window,
                    |_, _project, event, _window, cx| match event {
                        ProjectEvent::WorktreeAdded(_)
                        | ProjectEvent::WorktreeRemoved(_)
                        | ProjectEvent::WorktreeOrderChanged => {
                            cx.notify();
                        }
                        _ => {}
                    },
                )
            })
            .collect()
    }

    fn workspace_entries(&self, cx: &App) -> Vec<WorkspaceEntry> {
        let Some(multi_workspace) = self.multi_workspace.upgrade() else {
            return Vec::new();
        };

        multi_workspace
            .read(cx)
            .workspaces()
            .iter()
            .map(|workspace| {
                let (label, full_path) = workspace_label_and_path(workspace, cx);
                WorkspaceEntry {
                    label,
                    full_path,
                    workspace: workspace.clone(),
                }
            })
            .collect()
    }

    fn activate_workspace(
        &self,
        workspace: &Entity<Workspace>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(multi_workspace) = self.multi_workspace.upgrade() else {
            return;
        };

        multi_workspace.update(cx, |multi_workspace, cx| {
            multi_workspace.activate(workspace.clone(), cx);
        });

        multi_workspace.update(cx, |multi_workspace, cx| {
            multi_workspace.focus_active_workspace(window, cx);
        });
    }

    fn remove_workspace(
        &self,
        workspace: &Entity<Workspace>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(multi_workspace) = self.multi_workspace.upgrade() else {
            return;
        };

        multi_workspace.update(cx, |multi_workspace, cx| {
            let Some(index) = multi_workspace
                .workspaces()
                .iter()
                .position(|candidate| candidate == workspace)
            else {
                return;
            };

            multi_workspace.remove_workspace(index, window, cx);
        });
    }

    fn render_recent_projects_button(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let workspace = self
            .multi_workspace
            .upgrade()
            .map(|multi_workspace| multi_workspace.read(cx).workspace().downgrade());
        let focus_handle = workspace
            .as_ref()
            .and_then(|workspace| workspace.upgrade())
            .map(|workspace| workspace.read(cx).focus_handle(cx))
            .unwrap_or_else(|| cx.focus_handle());
        let popover_handle = self.recent_projects_popover_handle.clone();

        PopoverMenu::new("sidebar-recent-projects-menu")
            .with_handle(popover_handle)
            .menu(move |window, cx| {
                workspace.as_ref().map(|workspace| {
                    RecentProjects::popover(
                        workspace.clone(),
                        false,
                        focus_handle.clone(),
                        window,
                        cx,
                    )
                })
            })
            .trigger_with_tooltip(
                IconButton::new("open-project", IconName::OpenFolder)
                    .icon_size(IconSize::Small)
                    .selected_style(ButtonStyle::Tinted(TintColor::Accent)),
                |_window, cx| {
                    Tooltip::for_action(
                        "Recent Projects",
                        &OpenRecent {
                            create_new_window: false,
                        },
                        cx,
                    )
                },
            )
            .anchor(gpui::Corner::TopLeft)
            .offset(gpui::Point {
                x: px(0.0),
                y: px(2.0),
            })
    }
}

impl WorkspaceSidebar for Sidebar {
    fn width(&self, _cx: &App) -> Pixels {
        self.width
    }

    fn set_width(&mut self, width: Option<Pixels>, cx: &mut Context<Self>) {
        self.width = width.unwrap_or(DEFAULT_WIDTH).clamp(MIN_WIDTH, MAX_WIDTH);
        cx.notify();
    }

    fn has_notifications(&self, _cx: &App) -> bool {
        false
    }

    fn toggle_recent_projects_popover(&self, window: &mut Window, cx: &mut App) {
        self.recent_projects_popover_handle.toggle(window, cx);
    }

    fn is_recent_projects_popover_deployed(&self) -> bool {
        self.recent_projects_popover_handle.is_deployed()
    }
}

impl Focusable for Sidebar {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for Sidebar {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let titlebar_height = ui::utils::platform_title_bar_height(window);
        let ui_font = theme::setup_ui_font(window, cx);
        let focus_handle = self.focus_handle.clone();

        let workspace_entries = self.workspace_entries(cx);
        let workspace_count = workspace_entries.len();
        let (active_workspace, multi_workspace_enabled) = self
            .multi_workspace
            .upgrade()
            .map(|multi_workspace| {
                multi_workspace.read_with(cx, |multi_workspace, cx| {
                    (
                        Some(multi_workspace.workspace().clone()),
                        multi_workspace.multi_workspace_enabled(cx),
                    )
                })
            })
            .unwrap_or((None, false));

        let mut content = v_flex().flex_1().p_2().gap_1();

        if !multi_workspace_enabled {
            content = content.child(
                Label::new("Multi-workspace is unavailable in this build.")
                    .size(LabelSize::Small)
                    .color(Color::Muted),
            );
        } else if workspace_entries.is_empty() {
            content = content.child(
                Label::new("No workspaces open")
                    .size(LabelSize::Small)
                    .color(Color::Muted),
            );
        } else {
            content = content.children(workspace_entries.into_iter().enumerate().map(
                |(index, entry)| {
                    let WorkspaceEntry {
                        label,
                        full_path,
                        workspace,
                    } = entry;
                    let remove_workspace = workspace.clone();
                    let activate_workspace = workspace.clone();
                    let is_active = active_workspace
                        .as_ref()
                        .is_some_and(|active_workspace| *active_workspace == workspace);

                    let label_element = if is_active {
                        HighlightedLabel::new(label.clone(), Vec::new())
                            .size(LabelSize::Small)
                            .color(Color::Default)
                            .into_any_element()
                    } else {
                        Label::new(label.clone())
                            .size(LabelSize::Small)
                            .color(Color::Muted)
                            .into_any_element()
                    };

                    ListItem::new(SharedString::from(format!("workspace-entry-{index}")))
                        .toggle_state(is_active)
                        .child(h_flex().min_w_0().w_full().p_1p5().child(label_element))
                        .end_hover_slot(h_flex().when(workspace_count > 1, |this| {
                            this.child(
                                IconButton::new(
                                    SharedString::from(format!("remove-workspace-{index}")),
                                    IconName::Close,
                                )
                                .icon_size(IconSize::Small)
                                .icon_color(Color::Muted)
                                .tooltip(Tooltip::text("Remove Workspace"))
                                .on_click(cx.listener(
                                    move |this, _, window, cx| {
                                        this.remove_workspace(&remove_workspace, window, cx);
                                    },
                                )),
                            )
                        }))
                        .when(!full_path.is_empty(), |this| {
                            this.tooltip(move |_, cx| {
                                Tooltip::with_meta(label.clone(), None, full_path.clone(), cx)
                            })
                        })
                        .on_click(cx.listener(move |this, _, window, cx| {
                            this.activate_workspace(&activate_workspace, window, cx);
                        }))
                },
            ));
        }

        v_flex()
            .id("workspace-sidebar")
            .key_context("WorkspaceSidebar")
            .font(ui_font)
            .track_focus(&self.focus_handle)
            .h_full()
            .w(self.width)
            .bg(cx.theme().colors().surface_background)
            .border_r_1()
            .border_color(cx.theme().colors().border)
            .child(
                h_flex()
                    .flex_none()
                    .h(titlebar_height)
                    .w_full()
                    .mt_px()
                    .pb_px()
                    .pr_1()
                    .when_else(
                        cfg!(target_os = "macos") && !window.is_fullscreen(),
                        |this| this.pl(px(TRAFFIC_LIGHT_PADDING)),
                        |this| this.pl_2(),
                    )
                    .justify_between()
                    .border_b_1()
                    .border_color(cx.theme().colors().border)
                    .child({
                        let focus_handle = focus_handle.clone();
                        IconButton::new("close-sidebar", IconName::ThreadsSidebarLeftOpen)
                            .icon_size(IconSize::Small)
                            .tooltip(Tooltip::element(move |_, cx| {
                                v_flex()
                                    .gap_1()
                                    .child(
                                        h_flex()
                                            .gap_2()
                                            .justify_between()
                                            .child(Label::new("Close Sidebar"))
                                            .child(KeyBinding::for_action_in(
                                                &ToggleWorkspaceSidebar,
                                                &focus_handle,
                                                cx,
                                            )),
                                    )
                                    .child(
                                        h_flex()
                                            .pt_1()
                                            .gap_2()
                                            .border_t_1()
                                            .border_color(cx.theme().colors().border_variant)
                                            .justify_between()
                                            .child(Label::new("Focus Workspace"))
                                            .child(KeyBinding::for_action_in(
                                                &FocusWorkspaceSidebar,
                                                &focus_handle,
                                                cx,
                                            )),
                                    )
                                    .into_any_element()
                            }))
                            .on_click(cx.listener(|_, _, _window, cx| {
                                cx.emit(SidebarEvent::Close);
                            }))
                    })
                    .child(
                        h_flex()
                            .gap_1()
                            .child(self.render_recent_projects_button(cx))
                            .child(
                                IconButton::new("new-workspace", IconName::Plus)
                                    .icon_size(IconSize::Small)
                                    .tooltip(|_window, cx| {
                                        Tooltip::for_action(
                                            "New Workspace",
                                            &NewWorkspaceInWindow,
                                            cx,
                                        )
                                    })
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        if let Some(multi_workspace) =
                                            this.multi_workspace.upgrade()
                                        {
                                            multi_workspace.update(cx, |multi_workspace, cx| {
                                                multi_workspace.create_workspace(window, cx);
                                            });
                                        }
                                    })),
                            ),
                    ),
            )
            .child(content)
    }
}
