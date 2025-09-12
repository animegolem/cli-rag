use regex::Regex;
use std::collections::{BTreeSet, HashMap, HashSet, VecDeque};

use crate::model::AdrDoc;

// Compute a dependency path between two ADR ids using BFS over
// a bidirectional graph (depends_on edges + reverse dependents).
pub fn bfs_path(
    from: &str,
    to: &str,
    max_depth: usize,
    by_id: &HashMap<String, AdrDoc>,
) -> Option<Vec<String>> {
    // Compile mentions regex once
    let mention_re: Regex = Regex::new(r"\[\[([A-Za-z]+-[0-9A-Za-z_-]+)\]\]").unwrap();
    if from == to {
        return Some(vec![from.into()]);
    }
    let mut q: VecDeque<(String, Vec<String>, usize)> = VecDeque::new();
    let mut visited = HashSet::new();
    q.push_back((from.into(), vec![from.into()], 0));
    visited.insert(from.into());
    while let Some((cur, path, depth)) = q.pop_front() {
        if depth >= max_depth {
            continue;
        }
        if let Some(doc) = by_id.get(&cur) {
            let mut neighbors: BTreeSet<String> = BTreeSet::new();
            for dep in &doc.depends_on {
                neighbors.insert(dep.clone());
            }
            for (oid, other) in by_id.iter() {
                if other.depends_on.iter().any(|d| d == &cur) {
                    neighbors.insert(oid.clone());
                }
            }
            // mentions-based neighbors: scan for wikilinks [[ID]]
            if let Ok(content) = std::fs::read_to_string(&doc.file) {
                for cap in mention_re.captures_iter(&content) {
                    if let Some(m) = cap.get(1) {
                        let t = m.as_str().to_string();
                        if by_id.contains_key(&t) {
                            neighbors.insert(t);
                        }
                    }
                }
            }
            for n in neighbors {
                if n == to {
                    let mut p = path.clone();
                    p.push(n);
                    return Some(p);
                }
                if !visited.contains(&n) {
                    visited.insert(n.clone());
                    let mut p = path.clone();
                    p.push(n.clone());
                    q.push_back((n, p, depth + 1));
                }
            }
        }
    }
    None
}

// Compute a cluster around an ADR id up to a depth, optionally including dependents.
pub fn compute_cluster(
    id: &str,
    depth: usize,
    include_bidirectional: bool,
    by_id: &HashMap<String, AdrDoc>,
) -> std::collections::BTreeMap<String, AdrDoc> {
    let mut visited = HashSet::new();
    let mut cluster: std::collections::BTreeMap<String, AdrDoc> = std::collections::BTreeMap::new();
    fn traverse(
        current: &str,
        depth: usize,
        include_bidir: bool,
        by_id: &HashMap<String, AdrDoc>,
        acc: &mut std::collections::BTreeMap<String, AdrDoc>,
        visited: &mut HashSet<String>,
    ) {
        if depth == 0 || visited.contains(current) {
            return;
        }
        visited.insert(current.to_string());
        if let Some(doc) = by_id.get(current) {
            acc.insert(current.to_string(), doc.clone());
            for dep in &doc.depends_on {
                traverse(dep, depth - 1, include_bidir, by_id, acc, visited);
            }
            if include_bidir {
                for (oid, other) in by_id.iter() {
                    if other.depends_on.iter().any(|d| d == current) {
                        traverse(oid, depth - 1, include_bidir, by_id, acc, visited);
                    }
                }
            }
        }
    }
    traverse(
        id,
        depth,
        include_bidirectional,
        by_id,
        &mut cluster,
        &mut visited,
    );
    cluster
}
