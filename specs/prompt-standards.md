---
Kind: constraint
---

# Prompt Standards

These standards constrain all prompts produced by this system.

## Role-First Structure

- Each prompt starts with an explicit role statement that defines the agent's responsibility.
- The role states priorities, allowed actions, and disallowed actions for the task.

## Explicit Constraints

- Prompts list non-negotiable constraints as concrete rules.
- Constraints cover safety, scope limits, and forbidden behaviors.
- Constraints are testable by review using this checklist:
  - each constraint uses observable language (`must`, `must not`, `exactly`, `at least`);
  - each constraint references a concrete artifact or field when applicable (file path, section, output key);
  - each constraint avoids undefined qualifiers (for example `reasonable`, `sufficient`, `appropriate`) unless separately defined.

## Output Contract

- Each prompt includes a required output format.
- Output format defines sections, ordering, and machine-readable elements where required.
- Prompts reject free-form output when structured output is required downstream.

## Context Delimiters

- Prompts separate instructions from provided context using explicit delimiters.
- Delimiters identify each context block type (specs, code, diffs, logs, prior output).
- Prompts avoid mixing instruction text with context payloads in the same block.

## Single Responsibility

- One prompt is responsible for one primary task.
- Multi-step workflows are split into multiple prompts with explicit handoff artifacts.
- Prompts avoid bundling analysis, implementation, review, and summarization in one request unless the workflow explicitly requires it.

## Reproducibility and Reviewability

- Prompts include enough context references (file paths, section references, constraints) to explain why an output was produced.
- Prompt templates are stored as versioned files and reviewed like code.
- Prompt changes must preserve compatibility with the consuming component's output contract.
