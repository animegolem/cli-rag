-- Example overlay module (usable as .cli-rag.lua or a template-specific Lua)
local M = {}

M.scan = {
filepaths = { "ADRs", "IMP" },
index_path = ".cli-rag/index.json",
}

M.graph = {
depth = 1,
include_bidirectional = true,
ai = {
depth = 1,
default_fanout = 5,
include_bidirectional = true,
neighbor_style = "metadata",
outline_lines = 2,
}
}

M.hooks = {
id_generator = function(ctx)
local n = ctx.index:next_numeric_id("ADR")
local id = string.format("ADR-%03d", n)
local title = (ctx.request and ctx.request.title) or "untitled"
local filename = string.format("%s-%s.md", id, ctx.util.kebab_case(title))
return { id = id, filename = filename }
end,

render_frontmatter = function(note, ctx)
return {
id = note.id,
status = "draft",
created_date = ctx.clock.today_iso(),
tags = {},
}
end,

validate = function(note, ctx)
local diags = {}
local min_out = (((ctx.schema or {}).validate or {}).edges or {}).wikilinks and (((ctx.schema.validate.edges.wikilinks or {}).min_outgoing) or 0) or 0
local link_count = #(note.body.links or {})
if link_count < min_out then
table.insert(diags, { severity = "warning", code = "LINK_MIN", msg = "Add at least 1 wikilink" })
end
return { diagnostics = diags }
end,

template_prompt = function(ctx)
return "# Instructions\nGenerate an ADR…"
end,

template_note = function(ctx)
-- Return full template text. Use TOML template by default; override here if needed.
return nil
end,
}

return M
