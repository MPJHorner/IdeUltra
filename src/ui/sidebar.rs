use std::path::PathBuf;

use egui::{CollapsingHeader, Id, ScrollArea, Ui};

use crate::workspace::tree::{FileTree, TreeNode};

/// Action requested by the sidebar during this frame.
pub enum SidebarAction {
    None,
    OpenFile(PathBuf),
}

pub fn show(ui: &mut Ui, tree: &mut FileTree) -> SidebarAction {
    let mut action = SidebarAction::None;
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(&tree.root.name).strong());
    });
    ui.separator();

    ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            // Render children of the root (we don't want the root itself collapsible).
            let mut to_toggle: Option<Vec<usize>> = None;
            let mut to_load: Vec<Vec<usize>> = Vec::new();
            render_children(
                ui,
                &tree.root,
                &mut Vec::new(),
                &mut action,
                &mut to_toggle,
                &mut to_load,
            );
            for p in to_load {
                tree.load_children(&p);
            }
            if let Some(p) = to_toggle {
                tree.toggle(&p);
            }
        });

    action
}

fn render_children(
    ui: &mut Ui,
    node: &TreeNode,
    path: &mut Vec<usize>,
    action: &mut SidebarAction,
    to_toggle: &mut Option<Vec<usize>>,
    to_load: &mut Vec<Vec<usize>>,
) {
    let Some(children) = node.children.as_ref() else {
        return;
    };
    for (idx, child) in children.iter().enumerate() {
        path.push(idx);
        render_node(ui, child, path, action, to_toggle, to_load);
        path.pop();
    }
}

fn render_node(
    ui: &mut Ui,
    node: &TreeNode,
    path: &mut Vec<usize>,
    action: &mut SidebarAction,
    to_toggle: &mut Option<Vec<usize>>,
    to_load: &mut Vec<Vec<usize>>,
) {
    if node.is_dir {
        let id = Id::new(("dir", node.path.as_path()));
        let header = CollapsingHeader::new(format!("{} {}", folder_glyph(node.expanded), node.name))
            .id_source(id)
            .default_open(node.expanded);
        let resp = header.show(ui, |ui| {
            // Lazy: request load if not yet loaded.
            if node.children.is_none() {
                to_load.push(path.clone());
            } else {
                render_children(ui, node, path, action, to_toggle, to_load);
            }
        });
        // Reflect open/close state back into our model so we keep it stable.
        if resp.fully_open() != node.expanded && to_toggle.is_none() {
            *to_toggle = Some(path.clone());
        }
    } else {
        let label = format!("📄 {}", node.name);
        let resp = ui.add(egui::SelectableLabel::new(false, label));
        if resp.clicked() {
            *action = SidebarAction::OpenFile(node.path.clone());
        }
    }
}

fn folder_glyph(expanded: bool) -> &'static str {
    if expanded {
        "📂"
    } else {
        "📁"
    }
}
