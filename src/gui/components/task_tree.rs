use eframe::egui;

#[derive(Debug, Clone)]
pub struct TreeNode {
    pub id: String,
    pub label: String,
    pub status: NodeStatus,
    pub expanded: bool,
    pub children: Vec<TreeNode>,
}

#[derive(Debug, Clone)]
pub enum NodeStatus {
    Completed,
    InProgress,
    Failed,
    Pending,
}

pub struct TaskTree {
    nodes: Vec<TreeNode>,
}

impl TaskTree {
    pub fn new(nodes: Vec<TreeNode>) -> Self {
        Self { nodes }
    }
    
    pub fn ui(&mut self, ui: &mut egui::Ui) -> bool {
        let mut changed = false;
        
        let nodes = &mut self.nodes;
        for i in 0..nodes.len() {
            changed |= Self::render_node_static(&mut nodes[i], ui, 0);
        }
        
        changed
    }
    
    fn render_node_static(node: &mut TreeNode, ui: &mut egui::Ui, depth: usize) -> bool {
        let mut changed = false;
        
        ui.horizontal(|ui| {
            // Indentation
            ui.add_space(depth as f32 * 20.0);
            
            // Expand/collapse button
            if !node.children.is_empty() {
                let icon = if node.expanded { "📂" } else { "📁" };
                if ui.small_button(icon).clicked() {
                    node.expanded = !node.expanded;
                    changed = true;
                }
            } else {
                ui.add_space(20.0); // Space for alignment
            }
            
            // Status icon
            let (icon, color) = match node.status {
                NodeStatus::Completed => ("✅", egui::Color32::GREEN),
                NodeStatus::InProgress => ("🔄", egui::Color32::BLUE),
                NodeStatus::Failed => ("❌", egui::Color32::RED),
                NodeStatus::Pending => ("⏳", egui::Color32::YELLOW),
            };
            
            ui.colored_label(color, icon);
            ui.label(&node.label);
        });
        
        // Render children if expanded
        if node.expanded {
            for child in &mut node.children {
                changed |= Self::render_node_static(child, ui, depth + 1);
            }
        }
        
        changed
    }
}

impl NodeStatus {
    pub fn icon(&self) -> &'static str {
        match self {
            NodeStatus::Completed => "✅",
            NodeStatus::InProgress => "🔄",
            NodeStatus::Failed => "❌",
            NodeStatus::Pending => "⏳",
        }
    }
    
    pub fn color(&self) -> egui::Color32 {
        match self {
            NodeStatus::Completed => egui::Color32::GREEN,
            NodeStatus::InProgress => egui::Color32::BLUE,
            NodeStatus::Failed => egui::Color32::RED,
            NodeStatus::Pending => egui::Color32::YELLOW,
        }
    }
}