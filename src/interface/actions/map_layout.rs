//! Graph layout algorithms for the map view.
//!
//! This module calculates 2D positions for cards based on their parent-child
//! relationships in the registry.

use crate::application::GraphTopologyView;
use crate::domain::id::CardId;
use std::collections::HashMap;

const NODE_WIDTH: f64 = 200.0;
const NODE_HEIGHT: f64 = 60.0;
const X_SPACING: f64 = 40.0;
const Y_SPACING: f64 = 100.0;

/// A node positioned in 2D space.
#[derive(Clone, Debug)]
pub struct LayoutNode {
    pub id: CardId,
    pub title: String,
    pub x: f64,
    pub y: f64,
    pub is_center: bool,
    pub width: f64,
    pub height: f64,
}

/// An edge connecting two 2D points.
#[derive(Clone, Debug)]
pub struct LayoutEdge {
    pub source_id: CardId,
    pub target_id: CardId,
    pub source_point: (f64, f64),
    pub target_point: (f64, f64),
}

/// The fully positioned graph.
#[derive(Clone, Debug)]
pub struct GraphLayout {
    pub nodes: Vec<LayoutNode>,
    pub edges: Vec<LayoutEdge>,
    pub min_x: f64,
    pub max_x: f64,
    pub min_y: f64,
    pub max_y: f64,
    pub center_point: (f64, f64),
}

/// Calculates a tree layout for the given topology.
pub fn calculate_layout(topology: &GraphTopologyView) -> GraphLayout {
    if topology.nodes.is_empty() {
        return GraphLayout {
            nodes: vec![],
            edges: vec![],
            min_x: 0.0,
            max_x: 0.0,
            min_y: 0.0,
            max_y: 0.0,
            center_point: (0.0, 0.0),
        };
    }

    let mut children_map: HashMap<CardId, Vec<CardId>> = HashMap::new();
    let mut node_map = HashMap::new();
    let mut root_id = None;

    for node in &topology.nodes {
        node_map.insert(node.id, node);
        if node.parent_id.is_none() {
            root_id = Some(node.id);
        } else if let Some(parent_id) = node.parent_id {
            children_map.entry(parent_id).or_default().push(node.id);
        }
    }

    // Step 1: Calculate subtree widths
    let mut subtree_widths = HashMap::new();
    if let Some(root) = root_id {
        calculate_widths(root, &children_map, &mut subtree_widths);
    }

    // Step 2: Assign coordinates
    let mut positions = HashMap::new();
    if let Some(root) = root_id {
        assign_positions(root, &children_map, &subtree_widths, &mut positions, 0.0, 0.0);
    }

    // Step 3: Build final output
    let mut layout_nodes = Vec::new();
    let mut min_x = f64::MAX;
    let mut max_x = f64::MIN;
    let mut min_y = f64::MAX;
    let mut max_y = f64::MIN;
    let mut center_point = (0.0, 0.0);

    for node in &topology.nodes {
        let (x, y) = positions.get(&node.id).copied().unwrap_or((0.0, 0.0));
        if x < min_x { min_x = x; }
        if x > max_x { max_x = x; }
        if y < min_y { min_y = y; }
        if y > max_y { max_y = y; }

        if node.id == topology.center_id {
            center_point = (x, y);
        }

        layout_nodes.push(LayoutNode {
            id: node.id,
            title: node.title.clone(),
            x,
            y,
            is_center: node.id == topology.center_id,
            width: NODE_WIDTH,
            height: NODE_HEIGHT,
        });
    }

    let mut layout_edges = Vec::new();
    for (source_id, target_id) in &topology.edges {
        let source_pos = positions.get(source_id).copied().unwrap_or((0.0, 0.0));
        let target_pos = positions.get(target_id).copied().unwrap_or((0.0, 0.0));
        
        // Connect bottom of source to top of target
        layout_edges.push(LayoutEdge {
            source_id: *source_id,
            target_id: *target_id,
            source_point: (source_pos.0, source_pos.1 + (NODE_HEIGHT / 2.0)),
            target_point: (target_pos.0, target_pos.1 - (NODE_HEIGHT / 2.0)),
        });
    }

    GraphLayout {
        nodes: layout_nodes,
        edges: layout_edges,
        min_x,
        max_x: max_x + NODE_WIDTH,
        min_y,
        max_y: max_y + NODE_HEIGHT,
        center_point,
    }
}

fn calculate_widths(
    node_id: CardId,
    children_map: &HashMap<CardId, Vec<CardId>>,
    widths: &mut HashMap<CardId, f64>,
) -> f64 {
    let children = children_map.get(&node_id).cloned().unwrap_or_default();
    if children.is_empty() {
        widths.insert(node_id, NODE_WIDTH);
        return NODE_WIDTH;
    }

    let mut total_width = 0.0;
    for (i, child_id) in children.iter().enumerate() {
        total_width += calculate_widths(*child_id, children_map, widths);
        if i < children.len() - 1 {
            total_width += X_SPACING;
        }
    }

    let width = total_width.max(NODE_WIDTH);
    widths.insert(node_id, width);
    width
}

fn assign_positions(
    node_id: CardId,
    children_map: &HashMap<CardId, Vec<CardId>>,
    widths: &HashMap<CardId, f64>,
    positions: &mut HashMap<CardId, (f64, f64)>,
    x: f64,
    y: f64,
) {
    positions.insert(node_id, (x, y));

    let children = children_map.get(&node_id).cloned().unwrap_or_default();
    if children.is_empty() {
        return;
    }

    let total_children_width: f64 = children.iter().map(|id| widths.get(id).copied().unwrap_or(NODE_WIDTH)).sum::<f64>() + ((children.len() - 1) as f64 * X_SPACING);
    
    let mut current_x = x - (total_children_width / 2.0);

    for child_id in children {
        let child_width = widths.get(&child_id).copied().unwrap_or(NODE_WIDTH);
        let child_center_x = current_x + (child_width / 2.0);
        assign_positions(child_id, children_map, widths, positions, child_center_x, y + Y_SPACING + NODE_HEIGHT);
        current_x += child_width + X_SPACING;
    }
}
