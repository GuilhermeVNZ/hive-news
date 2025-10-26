# OpenSpec Workflow Instructions

This project uses OpenSpec for spec-driven development with AI coding assistants.

## Directory Structure

```
openspec/
├── specs/          # Source of truth (current system behavior)
└── changes/        # Proposed updates (proposals, tasks, spec deltas)
```

## Workflow Stages

### 1. Draft Change Proposal

Ask your AI assistant to create a change proposal for a feature or improvement.

**For tools with slash commands:** Use `/openspec-proposal <description>`
**For other AI tools:** Request "create an OpenSpec change proposal for..."

The AI will generate:

- `openspec/changes/{feature-name}/proposal.md` - Why and what changes
- `openspec/changes/{feature-name}/tasks.md` - Implementation checklist
- `openspec/changes/{feature-name}/specs/` - Spec deltas showing additions/modifications

### 2. Review & Refine

- Review the proposal with `openspec show {change-name}`
- Validate format with `openspec validate {change-name}`
- Iterate on specs until aligned

### 3. Implement Tasks

Ask your AI to implement the change using the agreed specs.

**For tools with slash commands:** Use `/openspec-apply {change-name}`
**For other AI tools:** Request "implement the OpenSpec change {change-name}"

### 4. Archive Completed Changes

After implementation is complete, archive the change to merge approved updates into source specs.

**For tools with slash commands:** Use `/openspec-archive {change-name} --yes`
**For other AI tools:** Request "archive the OpenSpec change {change-name}"

Or run: `openspec archive {change-name} --yes`

## Commands

```bash
openspec list                         # List active change folders
openspec show {change-name}         # Display change details
openspec validate {change-name}      # Validate spec format
openspec archive {change-name} [--yes]  # Archive completed change
```

## Specification Format

### Delta Format Requirements

- Use `## ADDED Requirements` for new capabilities
- Use `## MODIFIED Requirements` for changed behavior (include complete updated text)
- Use `## REMOVED Requirements` for deprecated features
- Use `### Requirement: <name>` for headers
- Every requirement needs at least one `#### Scenario:` block
- Use SHALL/MUST in requirement text

## Additional Notes

- Specifications live in `openspec/specs/` as the single source of truth
- Proposals track changes in `openspec/changes/` until archived
- Archived changes merge their approved updates back into specs/
- This workflow works with any AI tool (Cursor, Claude, Cline, etc.)
