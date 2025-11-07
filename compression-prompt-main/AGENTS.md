# Compression-Prompt AI Assistant Guidelines

## Documentation Standards

**CRITICAL**: Minimize Markdown files. Keep documentation organized.

### Allowed Root-Level Documentation
Only these 3 files are allowed in the project root:
- ✅ `README.md` - Project overview and quick start
- ✅ `CHANGELOG.md` - Version history and release notes
- ✅ `AGENTS.md` - This file (AI assistant instructions)

### All Other Documentation
**ALL other documentation MUST go in `/docs` directory**:
- `/docs/ARCHITECTURE.md` - System architecture
- `/docs/ROADMAP.md` - Project roadmap
- `/docs/specs/` - Technical specifications

### DO NOT CREATE
- ❌ Individual `.md` files in project root (BUILD.md, SUMMARY.md, etc.)
- ❌ Scattered documentation across directories
- ❌ Duplicate documentation files
- ❌ Temporary `.md` files for notes

**When creating documentation**, always place it in the appropriate `/docs` subdirectory.

## Feature Specifications

**CRITICAL**: All feature specifications are in `/docs` directory.

### Implementation Workflow

1. **Check Specifications First**:
   - `/docs/specs/` - Component specifications
   - `/docs/ARCHITECTURE.md` - System architecture
   - `/docs/ROADMAP.md` - Implementation timeline

2. **Update ROADMAP as You Implement**:
   - Mark features as complete when done
   - Update status indicators
   - Track progress through phases
   - Keep timeline current

3. **Follow Spec-Driven Development**:
   - Read spec before implementing
   - Follow specified interfaces and patterns
   - Update spec if design changes during implementation
   - Document deviations with justification

### Example Implementation Flow

```
1. Read /docs/specs/ALGORITHM.md
2. Implement feature following spec
3. Write tests based on spec requirements
4. Update /docs/ROADMAP.md progress markers
5. Benchmark claims in /paper/sections/
6. Commit with reference to spec
```

## Code Quality

- **Rust Edition**: 2024 (nightly 1.85+)
- **Location**: All Rust code in `/rust` subdirectory
- **Format**: Always run `cargo fmt` before committing
- **Lint**: Code must pass `cargo clippy` with no warnings
- **Tests**: Maintain >80% coverage, all tests must pass

## Dependencies Management

**CRITICAL**: Always verify latest versions before adding dependencies.

### Before Adding Any Dependency

1. **Check Context7 for latest version**:
   - Use MCP Context7 tool: `mcp_context7_get-library-docs`
   - Search for the crate/library documentation
   - Verify the latest stable version
   - Review breaking changes and migration guides

2. **Example Workflow**:
   ```
   Adding tokenizers → Check /huggingface/tokenizers on Context7
   Adding serde → Check latest stable version
   Adding tiktoken-rs → Check for latest version and compatibility
   ```

3. **Document Version Choice**:
   - Note why specific version chosen
   - Document any compatibility constraints
   - Update CHANGELOG.md with new dependencies

### Dependency Guidelines

- ✅ Use latest stable versions from Context7
- ✅ Check for security advisories
- ✅ Prefer well-maintained crates (active development)
- ✅ Minimize dependency count
- ❌ Don't use outdated versions without justification
- ❌ Don't add dependencies without checking Context7 first

## Project-Specific Rules

### Compression Algorithm
- Dictionary entries must have positive gain: `fᵢ*(Lᵢ - r) - Hᵢ > 0`
- Dictionary overhead threshold: 25-30% of total prompt
- Marker format must tokenize to 1-2 tokens consistently
- Graceful degradation: return original if compression ratio < 1.0

### Paper & Documentation
- arXiv paper sections in `/paper/sections/`
- All technical specs in `/docs/specs/`
- Use academic citation format in references.bib
- Maintain consistency between paper and code documentation

### Tokenizer Support
- Pluggable architecture via trait
- Test marker tokenization with each supported tokenizer
- Document tokenizer-specific quirks and optimizations

## Workflow

1. Read specifications in `/docs/specs/` before implementing
2. Update `/docs/ROADMAP.md` as features complete
3. Keep paper synchronized with implementation
4. Benchmark all claims in the paper
5. Document deviations from specs with justification

### Git Workflow

**CRITICAL**: Never automatically push to remote repositories.

- ✅ **DO**: Commit changes locally with descriptive messages
- ✅ **DO**: Show the push command in console for user to execute
- ❌ **DON'T**: Run `git push` automatically
- ❌ **DON'T**: Push without user confirmation

**Example:**
```bash
# After committing, show this message:
echo "✅ Changes committed locally"
echo ""
echo "To push to remote, run:"
echo "  git push origin main"
```

**Rationale:** User should have final control over what gets pushed to remote repositories.

---

## Vectorizer Instructions

**Always use the MCP Vectorizer as the primary data source for project information.**

The vectorizer provides fast, semantic access to the entire codebase. Prefer MCP tools over file reading whenever possible.

### Primary Search Functions

#### 1. **mcp_vectorizer_search**
Main search interface with multiple strategies:
- `intelligent`: AI-powered search with query expansion and MMR diversification
- `semantic`: Advanced semantic search with reranking and similarity thresholds
- `multi_collection`: Search across multiple collections

#### 2. **mcp_vectorizer_file_operations**
File-specific operations:
- `get_content`: Retrieve complete file content
- `list_files`: List all indexed files with metadata
- `get_chunks`: Retrieve file chunks in original order
- `get_outline`: Generate hierarchical project structure
- `get_related`: Find semantically related files

#### 3. **mcp_vectorizer_discovery**
Advanced discovery pipeline:
- `full_pipeline`: Complete discovery with filtering, scoring, and ranking
- `broad_discovery`: Multi-query search with deduplication
- `expand_queries`: Generate query variations (definition, features, architecture, API)

### Best Practices

1. **Start with intelligent search** for exploratory queries
2. **Use file_operations** when you need complete file context
3. **Use discovery pipeline** for complex, multi-faceted questions
4. **Prefer batch operations** when searching for multiple related items
5. **Use by_file_type** when working with Rust files in `/rust` directory

---

## Synap Instructions

**Use MCP Synap for short-term data storage and session state.**

Synap provides fast key-value storage, queuing, and pub/sub capabilities for temporary data during AI sessions.

### When to Use Synap

1. **Session State**: Store temporary variables, counters, or flags during task execution
2. **Inter-Step Data**: Pass data between different steps of complex workflows
3. **Cache Results**: Store intermediate computation results to avoid re-computation
4. **Task Coordination**: Queue tasks or publish events for async operations

### Primary Functions

#### Key-Value Store
- `mcp_Synap_synap_kv_set`: Store temporary data with optional TTL
- `mcp_Synap_synap_kv_get`: Retrieve stored values
- `mcp_Synap_synap_kv_delete`: Remove keys when done
- `mcp_Synap_synap_kv_scan`: Find keys by prefix pattern

#### Queues & Streams
- `mcp_Synap_synap_queue_publish`: Send messages to queues
- `mcp_Synap_synap_queue_consume`: Receive queued messages
- `mcp_Synap_synap_stream_publish`: Publish events to streams
- `mcp_Synap_synap_pubsub_publish`: Broadcast to pub/sub topics

### Best Practices

1. **Use TTL**: Set expiration times for temporary data
2. **Prefix keys**: Use namespaces like `session:`, `cache:`, `task:`
3. **Clean up**: Delete keys when no longer needed
4. **Don't store large files**: Use for metadata and small data only

### Example Use Cases

```
✅ Store progress counter during multi-file refactoring
✅ Cache API responses temporarily
✅ Queue tasks for sequential processing
✅ Store session variables across tool calls
❌ Don't store entire file contents (use vectorizer instead)
❌ Don't use for permanent data (commit to git instead)
```

