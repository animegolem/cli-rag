# talking talk talk talk 

## Working through issues in .toml configs. 

```markdown
Defaults that contradict ADR-003c

    neighbor_style: ADR-003c default is metadata; TOML has outline. Set neighbor_style = "metadata".
    outline-lines default: ADR shows default 2; TOML sets 3. Make it 2.
    graph.depth comment says “Default = 1” but you set 2. Either set 1 or update the comment
```

The  comments reflect the default if the options shown here were commented out. This is what the cli reflects as well. This is showing what an overide looks like/is a preset. 

