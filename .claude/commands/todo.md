Add a new well-structured todo item to `todos.md`.

## Handling the invocation

The user may invoke this command with varying levels of detail in $ARGUMENTS:

- **No arguments** — ask all interview questions from scratch.
- **Brief summary only** (e.g. "add a todo for fixing the remove button") — treat it as the title/summary and ask the remaining questions.
- **Detailed description** (e.g. includes what's wrong, what done looks like) — extract as much as you can into the right fields and only ask follow-up questions for anything still missing or unclear.
- **Fully specified** — if $ARGUMENTS covers all four fields clearly, skip straight to the preview step.

Use judgement: don't ask a question if the answer is already clear from what the user provided.

## Interview questions (ask only what's missing)

Ask these **one at a time**, waiting for the answer before asking the next:

1. **Summary** — "What's the one-line title for this todo?" *(skip if already clear from $ARGUMENTS)*
2. **Problem** — "Describe the problem or missing feature in more detail." *(skip if already clear)*
3. **Acceptance criteria** — "How will you know it's done? What must be true when complete?" *(skip if already clear)*
4. **Notes** — "Any implementation hints, relevant files, or constraints to capture? (Leave blank to skip.)" *(skip if already provided or clearly not applicable)*

## Preview and confirm

Use the `AskUserQuestion` tool to show the user a formatted preview of the todo entry and ask: **"Does this look right, or would you like to change anything?"** Adjust if needed.

## Write to file

Read `todos.md`, count existing `## N.` headings to get the next number, then append:

```markdown
## N. <Summary>

**Problem:** <Problem>

**Acceptance criteria:**
- <criterion>
- ...

**Notes:** <Notes — omit this section entirely if none>

---
```

Confirm: "Added as todo #N."
