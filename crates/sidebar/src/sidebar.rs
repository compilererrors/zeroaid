#[cfg(feature = "ai")]
mod sidebar_ai;

#[cfg(feature = "ai")]
pub use sidebar_ai::Sidebar;

#[cfg(not(feature = "ai"))]
mod lean_sidebar {
    use gpui::{App, Context, Entity, FocusHandle, Focusable, Pixels, Render, Window, px};
    use workspace::{MultiWorkspace, Sidebar as WorkspaceSidebar, SidebarSide};

    const DEFAULT_WIDTH: Pixels = px(320.0);
    const MIN_WIDTH: Pixels = px(200.0);
    const MAX_WIDTH: Pixels = px(800.0);

    pub struct Sidebar {
        width: Pixels,
        focus_handle: FocusHandle,
    }

    impl Sidebar {
        pub fn new(
            _multi_workspace: Entity<MultiWorkspace>,
            _window: &mut Window,
            cx: &mut Context<Self>,
        ) -> Self {
            Self {
                width: DEFAULT_WIDTH,
                focus_handle: cx.focus_handle(),
            }
        }
    }

    impl Focusable for Sidebar {
        fn focus_handle(&self, _cx: &App) -> FocusHandle {
            self.focus_handle.clone()
        }
    }

    impl WorkspaceSidebar for Sidebar {
        fn width(&self, _cx: &App) -> Pixels {
            self.width
        }

        fn set_width(&mut self, width: Option<Pixels>, _cx: &mut Context<Self>) {
            if let Some(width) = width {
                self.width = width.max(MIN_WIDTH).min(MAX_WIDTH);
            } else {
                self.width = DEFAULT_WIDTH;
            }
        }

        fn has_notifications(&self, _cx: &App) -> bool {
            false
        }

        fn side(&self, _cx: &App) -> SidebarSide {
            SidebarSide::Left
        }
    }

    impl Render for Sidebar {
        fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl gpui::IntoElement {
            "Sidebar is enabled in this build."
        }
    }
}

#[cfg(not(feature = "ai"))]
pub use lean_sidebar::Sidebar;
