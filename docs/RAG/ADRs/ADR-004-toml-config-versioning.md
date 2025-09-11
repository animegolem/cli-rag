---
id: ADR-4
tags:
  - toml
  - config
  - versioning
  - future_proof
status: accepted
depends_on: [ADR-001]
created_date: 2025-08-25
last_modified: 2025-08-25
related_files: [.cli-rag.toml]
---

# toml-config-versioning

## Objective
<!-- A concise statement explaining the goal of this decision. -->

As we change schema processing we need a way to support and deprecate older config versions. 

see examples in contracts or (less updated) [[ADR-001-cli-rag.toml]]

## Context
<!-- What is the issue that we're seeing that is motivating this decision or change? -->

reasonable statement of the issue 

```gemini 2.5 pro  
- **You are developing cli-rag v0.3.** In this version, you decide to make a breaking change. You rename the legal_entry key to allowed_values because it's clearer.
    
- **In your Rust code, you make the following changes:**
    
    - The parser for config_version = "0.3" now only accepts allowed_values.
        
    - The parser for config_version = "0.1" and "0.2" is still there, but you add a compile-time warning to it:
        
        code Rust
        - - IGNORE_WHEN_COPYING_START
        
        IGNORE_WHEN_COPYING_END
        
            `#[deprecated(since = "0.3.0", note = "v0.1 support will be removed in v0.4.0. Please upgrade your config.")] fn parse_v01_config(...) { ... }`
          
        
- **A user has a project configured with config_version = "0.1".**
    
    - They upgrade their cli-rag binary to 0.3.
        
    - When they run any command (validate, get, etc.), the tool reads their config_version = "0.1".
        
    - The tool works perfectly, but it prints a clear, non-blocking warning to the console: "Warning: Your configuration is using a deprecated format (v0.1). This format will be removed in a future release (v0.4). Please consider upgrading."
        
- **The user decides to upgrade.**
    
    - You can provide a simple cli-rag config upgrade command that reads their v0.1 file, renames the keys, and writes out a new file with config_version = "0.3".
```

Simple model of you can add features but breaking changes are a new api version. 

## Decision
<!-- What is the change that we're proposing and/or doing? -->

Implement a config versioning flag 

## Consequences
<!-- What becomes easier or more difficult to do because of this change? -->

we can change the application and not break user notes. 

## Updates
<!-- Changes that happened when the rubber met the road -->