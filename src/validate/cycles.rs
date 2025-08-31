use std::collections::{HashMap, HashSet};

// Find simple cycles in a directed graph represented by adjacency list of id -> neighbors
pub fn find_cycles(adj: &HashMap<String, Vec<String>>) -> Vec<Vec<String>> {
    let mut cycles: Vec<Vec<String>> = Vec::new();
    let mut on_path: HashSet<String> = HashSet::new();
    let mut visited: HashSet<String> = HashSet::new();
    let mut stack: Vec<String> = Vec::new();

    fn dfs(
        node: &str,
        adj: &HashMap<String, Vec<String>>,
        visited: &mut HashSet<String>,
        on_path: &mut HashSet<String>,
        stack: &mut Vec<String>,
        out: &mut Vec<Vec<String>>,
    ) {
        visited.insert(node.to_string());
        on_path.insert(node.to_string());
        stack.push(node.to_string());
        if let Some(neigh) = adj.get(node) {
            for n in neigh {
                if !visited.contains(n) {
                    dfs(n, adj, visited, on_path, stack, out);
                } else if on_path.contains(n) {
                    // Found a cycle; extract path from first occurrence of n
                    if let Some(pos) = stack.iter().position(|s| s == n) {
                        let mut cyc = stack[pos..].to_vec();
                        cyc.push(n.clone());
                        out.push(cyc);
                    }
                }
            }
        }
        stack.pop();
        on_path.remove(node);
    }

    for node in adj.keys() {
        if !visited.contains(node) {
            dfs(
                node,
                adj,
                &mut visited,
                &mut on_path,
                &mut stack,
                &mut cycles,
            );
        }
    }
    // Deduplicate cycles by canonical key
    use std::collections::HashSet as HS;
    let mut seen: HS<String> = HS::new();
    let mut unique = Vec::new();
    for c in cycles {
        // Canonicalize by sorted unique nodes joined with '>'
        let mut nodes: Vec<String> = c.clone();
        nodes.sort();
        nodes.dedup();
        let key = nodes.join(">");
        if seen.insert(key) {
            unique.push(c);
        }
    }
    unique
}
