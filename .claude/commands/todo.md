Add a new well-structured todo item to `todos.md`.

## Handling the invocation

The user may invoke this command with varying levels of detail in $ARGUMENTS:

- **No arguments** — ask all interview questions from scratch.
- **Brief summary only** (e.g. "add a todo for fixing the remove button") — treat it as the title/summary and ask the remaining questions.
- **Detailed description** (e.g. includes what's wrong, what done looks like) — extract as much as you can into the right fields and only ask follow-up questions for anything still missing or unclear.
- **Fully specified** — if $ARGUMENTS covers all four fields clearly, skip straight to the priority step.

Use judgement: don't ask a question if the answer is already clear from what the user provided.

## Interview questions (ask only what's missing)

Ask these **one at a time**, waiting for the answer before asking the next:

1. **Summary** — "What's the one-line title for this todo?" *(skip if already clear from $ARGUMENTS)*
2. **Problem** — "Describe the problem or missing feature in more detail." *(skip if already clear)*
3. **Acceptance criteria** — "How will you know it's done? What must be true when complete?" *(skip if already clear)*
4. **Notes** — "Any implementation hints, relevant files, or constraints to capture? (Leave blank to skip.)" *(skip if already provided or clearly not applicable)*

## Assign priority

After gathering the fields, assign a priority using your best judgment based on **impact to the user's workflow**, not just bug vs. enhancement:

- **High** — data integrity bugs, broken core features, OR high-value enhancements that significantly improve the primary workflow. Either a critical fix or something that materially changes how useful the app is day-to-day.
- **Medium** — moderate bugs that have workarounds, or useful enhancements that add convenience without changing the core workflow.
- **Low** — cosmetic issues, minor polish, edge-case fixes, or enhancements with narrow applicability.

State your suggested priority and one-sentence rationale, then ask: **"Priority looks right, or would you change it?"**

## Preview and confirm

Show the user a formatted preview of the todo entry (including priority) and ask: **"Does this look right, or would you like to change anything?"** Adjust if needed.

## Write to file

Read `todos.md`, then insert the new todo **after the last existing entry of the same priority tier** (High before Medium before Low) so the file stays sorted High → Medium → Low.

Format:

```markdown
## <Summary>

**Priority:** High | Medium | Low

**Problem:** <Problem>

**Acceptance criteria:**
- <criterion>
- ...

**Notes:** <Notes — omit this section entirely if none>

---
```

Confirm: "Added."
