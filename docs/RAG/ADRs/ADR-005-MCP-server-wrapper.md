---
id: ADR-005
tags:
  - MCP
  - LLM
  - Agentic
  - Tooling
status: accepted
depends_on: ADR-001, ADR-002, ADR-003b
created_date: 2025-08-26
last_modified: 2025-08-26
related_files: []
---

# MCP-server-wrapper

## Objective
<!-- A concise statement explaining the goal of this decision. -->

Translate the tools to an MCP to allow an LLM to be the 'head' of our headless knowlege graph so both the user and LLM can contribute to schema constrained notes. This should be an "even better" situation, the agent should be able to theoretically work via cli or MCP. 

Ultimately the CLI is the more universal tool to nearly all deployments. 

## Context
<!-- What is the issue that we're seeing that is motivating this decision or change? -->

LLM's when prompted to document a section of a conversation produce long unstructured brain dumps often in one off ad hoc directories. 

## Decision
<!-- What is the change that we're proposing and/or doing? -->

No development will occur until the CLI tools are completely locked. This should serve largely as a 'thin wrapper' over the existing tooling leaning on the fact all tools produce an ndjson output. 

This will be a standalone server unconnected to my local dev tooling. 

## Consequences
<!-- What becomes easier or more difficult to do because of this change? -->

A clear separation between human and llm control surfaces so neither requires compromising the other. 

## Updates
<!-- Changes that happened when the rubber met the road -->
