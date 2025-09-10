
# I Don't think we will move forward with implementing this idea but we'll leave it around if the need comes up dogfooding. 

Great—let’s lock a minimal, shippable nav you can implement without touching the config layout you just stabilized.

Command
- nav <ID|path>
- Optional flags (all default to existing config/behavior):
  - --depth N (default: `[config.graph].depth`, else 1)
  - --include-bidirectional (default: `[config.graph].include_bidirectional`)
  - --edges <csv> (allowlist; e.g., depends_on,blocked_by)
  - --fanout <n> (default 5; per-edge cap in the UI)

What nav does
- Starts focused on the given node.
- Builds a tiny local neighborhood (depth 1 or 2) respecting:
  - edge kinds from `schema.frontmatter.edges.graph_edges`
  - cross-schema rules from `schema.validate.edges.cross_schema.allowed_targets`
  - backlinks only if include_bidirectional
- Ranks neighbors by weight from `schema.validate.edges.<edge>.weight` (default 1.0), then by `last_modified` desc, then by `priority` desc if present.
- Shows a compact ASCII view plus a numbered list of top neighbors (fanout per edge).
- Single-key navigation: numbers jump focus, o opens in editor, b toggles backlinks, q quits.

Default layout (one screen, no TUI dependency)
Example for depth=1, fanout=5, backlinks off:

```
Focus: ADR-002  Visual-Mode-planning
Path : ADRs/ADR-002-Visual-Mode-planning.md
Updated: 2025-08-26 12:14  |  Schema: ADR

Neighbors (ranked by edge weight → recency):
  depends_on (w=1.0)
   [1] ADR-001  cli-rag.toml
  implements (w=0.7)
   [2] ADR-003b v1-CLI-commands
  supersedes (w=0.5)
   [3] ADR-000 legacy-planning

Hints: 1..9 jump • o open in editor • b backlinks on/off • d depth 1/2 • e edge filter • q quit
```

If include_bidirectional is on, add a “dependents” group (same ranking rules). If there are more than 9 visible entries, page them (n next page, p prev page), keeping numbers 1–9 per page.

Keybindings (v1)
- 1–9: move focus to that neighbor
- o: open focused note in `$EDITOR` (or `config.editor`, fallback to `nvim`/`vim`)
- b: toggle include_bidirectional and redraw
- d: toggle depth 1 ↔ 2 and redraw
- e: enter a tiny prompt to filter edges (e.g., “depends_on,implements”); Enter to apply
- h: go back (pop focus history stack)
- q: quit

Ranking and fanout (human UI uses the same policy as ai get)
- Edge kinds come from `schema.frontmatter.edges.graph_edges`.
- Weights come from `schema.validate.edges.<edge>.weight` (default 1.0).
- Sort neighbors by (weight desc, last_modified desc, priority desc).
- Apply fanout per edge group (default 5; overridable with --fanout).
- Depth=1 shows direct neighbors of the focus only. Depth=2 also shows edges among neighbors in the ASCII sketch (optional), but the numbered list is always the direct neighbors of the focus.

Behavior details
- Cross-schema: only traverse to schemas listed in `schema.validate.edges.cross_schema.allowed_targets` (if present). Otherwise, stay within the focus schema.
- Missing metadata: if a neighbor lacks `last_modified` or `priority`, treat them as oldest/lowest for tie-breaking.
- Stability: within a redraw, numbering is stable. After toggles, rerank and renumber.
- No mutation: nav never edits files or frontmatter. Pure read + launch editor on o.

Minimal flags and defaults mapping
- depth: from `[config.graph].depth` if set; otherwise default 1 for nav.
- include_bidirectional: from `[config.graph].include_bidirectional`.
- fanout: default 5 unless `--fanout` is provided. (No need to add a new config key.)
- edges: default is “all in graph_edges”; `--edges` can narrow it.

Implementation sketch (Rust-ish pseudocode, but trivial to port)
```
struct ViewState {
  focus_id: String,
  depth: u8,
  include_backlinks: bool,
  edge_filter: Option<HashSet<String>>,
  page: usize,
  history: Vec<String>,
}

fn rank_neighbors(focus: &Node, cfg: &Cfg, index: &Index, vs: &ViewState) -> Vec<NeighborRow> {
  let edge_kinds = cfg.graph_edges(); // from schema.frontmatter.edges.graph_edges
  let allowed = vs.edge_filter.as_ref().unwrap_or(&edge_kinds);
  let mut rows = vec![];

  for edge in allowed {
    let w = cfg.edge_weight(edge).unwrap_or(1.0); // from schema.validate.edges
    for nbr in index.out_neighbors(focus, edge) {
      rows.push(NeighborRow {
        edge: edge.clone(),
        id: nbr.id.clone(),
        title: nbr.title.clone(),
        weight: w,
        last_modified: nbr.last_modified.unwrap_or(EPOCH),
        priority: nbr.priority.unwrap_or(0),
      });
    }
    if vs.include_backlinks {
      for nbr in index.in_neighbors(focus, edge) {
        rows.push(NeighborRow { /* same */ });
      }
    }
  }

  rows.sort_by(|a,b| (
    b.weight.partial_cmp(&a.weight).unwrap_or(Ordering::Equal),
    b.last_modified.cmp(&a.last_modified),
    b.priority.cmp(&a.priority)
  ));

  // fanout per edge: group, then take K, then flatten preserving order
  let k = vs.fanout; // from flag or default
  let mut grouped = group_by_edge(rows);
  let mut final_rows = vec![];
  for edge in allowed_in_defined_order(allowed, &cfg) {
    let mut take = grouped.remove(edge).unwrap_or_default();
    take.truncate(k);
    final_rows.extend(take);
  }
  final_rows
}

fn nav_loop(start: &str, cfg: &Cfg, index: &Index) {
  let mut vs = ViewState { focus_id: start.to_string(), depth: default_depth(cfg), include_backlinks: cfg.include_backlinks(), edge_filter: None, page: 0, history: vec![] };
  terminal::enter_raw_mode();
  loop {
    let focus = index.get(&vs.focus_id).unwrap();
    let neighbors = rank_neighbors(&focus, cfg, index, &vs);
    draw_ascii(&focus, &neighbors, &vs);
    match read_key()? {
      Key::Char('q') => break,
      Key::Char('o') => open_in_editor(&focus.path),
      Key::Char('b') => { vs.include_backlinks = !vs.include_backlinks; vs.page = 0; }
      Key::Char('d') => { vs.depth = if vs.depth == 1 {2} else {1}; }
      Key::Char('h') => { if let Some(prev) = vs.history.pop() { vs.focus_id = prev; } }
      Key::Char('n') => { vs.page += 1; }
      Key::Char('p') => { if vs.page > 0 { vs.page -= 1; } }
      Key::Char('e') => { vs.edge_filter = prompt_edges(); vs.page = 0; }
      Key::Digit(d @ '1'..='9') => {
        if let Some(row) = pick_from_page(&neighbors, vs.page, d) {
          vs.history.push(vs.focus_id.clone());
          vs.focus_id = row.id;
          vs.page = 0;
        }
      }
      _ => {}
    }
  }
  terminal::leave_raw_mode();
}
```

ASCII drawing (keep it simple)
- Always print the focus header (id, title, path).
- For depth=1: just the grouped neighbor list as shown above.
- For depth=2 (optional, non-blocking): add a tiny “edge peek” under each neighbor, e.g.:
  - ADR-003b: implements -> ADR-007, blocked_by -> ADR-099
  Keep it to at most 2 lines per neighbor to avoid scroll.

Performance notes
- Use your existing index for edges, titles, last_modified, priority.
- No file reads on each redraw; only when opening in editor.
- Precompute edge weights per schema at load time.

Tests (light)
- nav_respects_graph_edges_and_cross_schema
- nav_ranks_by_weight_then_recency
- nav_fanout_per_edge_enforced
- nav_backlinks_toggle_changes_candidate_set
- nav_history_back_works

No config churn required
- Uses existing `[config.graph].depth` and `[config.graph].include_bidirectional`.
- Adds only a CLI `--fanout` with a sensible default (5).
- Uses `schema.frontmatter.edges.graph_edges` for edge kinds and `schema.validate.edges.*.weight` for ranking.

