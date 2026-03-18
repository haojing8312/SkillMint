# Chat Reasoning Layout Design

**Date:** 2026-03-18

**Goal:** Improve the assistant reply experience by removing the heavy nested-card look from reasoning and tool status areas, and by keeping the reply content width visually stable when reasoning is expanded or collapsed.

## Problem

The current runtime chat UI mixes three different container models inside a single assistant response:

- the assistant bubble itself
- a bordered reasoning card inside that bubble
- a centered floating tool island with its own width rules

This creates two UX issues:

1. The response feels visually over-framed because the reasoning block adds a second card boundary inside the assistant bubble.
2. The visible width relationship between reasoning, tool history, and answer content changes across states, which makes expansion feel like a drawer opening rather than content revealing.

## Design Principles

- Keep one dominant content rail per assistant message.
- Use weaker hierarchy for reasoning and tool process details than for the final answer.
- Expansion should change height, not primary width.
- Preserve scanability without adding visual noise.
- Match top-tier agent products that favor soft grouping over stacked card borders.

## Chosen Direction

Use a unified message rail with minimal internal chrome:

- Keep the assistant message bubble as the main container.
- Turn reasoning into a low-emphasis inset section with soft background instead of a bordered card.
- Make tool execution history align to the same content width as the rest of the message rather than rendering as a narrow centered floating capsule.
- Keep expand/collapse animation focused on height and disclosure state, not on width shifts.

## Component-Level Decisions

### Thinking Block

- Remove the outer border.
- Replace the card treatment with a soft neutral background.
- Keep the status dot as the primary state signifier.
- Keep expand/collapse affordance lightweight.
- Keep expanded reasoning content aligned to the same internal width as the header.

### Tool Execution Summary

- Remove the narrow centered island behavior.
- Stretch to the message content width.
- Reduce container emphasis by using subtle background and light separators instead of a strong floating card treatment.
- Preserve the current disclosure behavior for per-step details.

### Assistant Bubble

- Continue using the assistant bubble as the primary structural container.
- Avoid repeated white card plus border plus shadow combinations inside that bubble.
- Keep reasoning, execution history, and markdown content visually related as parts of one answer.

## Interaction Rules

- Expanding reasoning must not change the assistant message width.
- Expanding tool history must not recenter or resize the message region.
- Collapsed and expanded states should share the same left edge and width baseline.
- State transitions should feel like revealing more content in place.

## Risks

- Over-flattening could reduce hierarchy too much and make process details blend into answer text.
- Updating width behavior may affect snapshots or assumptions in existing chat UI tests.
- Tool history styling must stay readable for running, completed, and failed states.

## Verification Intent

The change should be verified with runtime component tests that cover:

- reasoning block rendering and disclosure behavior
- tool island summary/detail rendering
- assistant streaming and historical message composition

The smallest honest verification set should focus on runtime tests that exercise `ThinkingBlock`, `ToolIsland`, and the relevant `ChatView` behaviors.
