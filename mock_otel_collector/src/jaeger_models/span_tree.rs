use std::collections::HashMap;

use anyhow::{anyhow, bail};
use itertools::Itertools as _;
use nonempty::NonEmpty;
use rctree::Node;

use super::Span;

pub fn build_span_tree(spans: impl IntoIterator<Item = Span>) -> Result<Node<Span>, anyhow::Error> {
    let mut spans_grouped_by_parent: HashMap<i64, NonEmpty<Span>> = spans
        .into_iter()
        .sorted_by_key(|s| s.parent_span_id)
        .group_by(|s| s.parent_span_id)
        .into_iter()
        .map(|(key, group)| (key, NonEmpty::from_vec(group.collect()).unwrap()))
        .collect();

    if spans_grouped_by_parent.is_empty() {
        bail!("Traces must include at least one span.");
    }

    let spans_without_parent = spans_grouped_by_parent
        .remove(&0_i64)
        .ok_or_else(|| anyhow!("No root span found"))?;

    if spans_without_parent.len() > 1 {
        bail!("Multiple root spans found");
    }

    let tree = Node::new(spans_without_parent.first().to_owned());
    let mut leaf_nodes_to_populate = vec![tree.clone()];

    while !leaf_nodes_to_populate.is_empty() {
        let mut leaf_node = leaf_nodes_to_populate.remove(0);
        let span_id = leaf_node.borrow().span_id;

        if let Some(child_spans) = spans_grouped_by_parent.remove(&span_id) {
            let new_child_span_trees: Vec<_> = child_spans.into_iter().map(Node::new).collect();
            for span in new_child_span_trees {
                leaf_node.append(span);
            }
            let new_leaf_nodes: Vec<_> = leaf_node.children().collect();
            for new_leaf_node in new_leaf_nodes {
                leaf_nodes_to_populate.push(new_leaf_node);
            }
        }
    }

    if !spans_grouped_by_parent.is_empty() {
        anyhow::bail!("Spans found with missing parents");
    }

    Ok(tree)
}
