;; Example overlay module (usable as .cli-rag.fnl or a template-specific Fennel)
{
 :scan {
  :filepaths ["ADRs" "IMP"]
  :index_path ".cli-rag/index.json"
 }

 :graph {
  :depth 1
  :include_bidirectional true
  :ai {
   :depth 1
   :default_fanout 5
   :include_bidirectional true
   :neighbor_style "metadata"
   :outline_lines 2
  }
 }

 :hooks {
  :id_generator (fn [ctx]
    (let [n (. ctx.index :next_numeric_id "ADR")
          id (string.format "ADR-%03d" n)
          title (or (. ctx :request :title) "untitled")
          filename (.. id "-" (ctx.util.kebab_case title) ".md")]
      {:id id :filename filename}))

  :render_frontmatter (fn [note ctx]
    {
     :id note.id
     :status "draft"
     :created_date (ctx.clock.today_iso)
     :tags []
    })

  :validate (fn [note ctx]
    (let [diags []
          min_out (or (. ctx :schema :validate :edges :wikilinks :min_outgoing) 0)
          link_count (length (. note :body :links))]
      (when (< link_count min_out)
        (table.insert diags {:severity "warning" :code "LINK_MIN" :msg "Add at least 1 wikilink"}))
      {:diagnostics diags}))

  :template_prompt (fn [ctx]
    "# Instructions\nGenerate an ADR…")

  :template_note (fn [ctx]
    ;; Return full template text. Use TOML template by default; override here if needed.
    nil)
 }
}
