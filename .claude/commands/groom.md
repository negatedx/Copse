Review and adjust priorities in `todos.md`.

## Step 1 — Display the backlog

Read `todos.md` and display every todo as a numbered list grouped by priority:

```
**High**
1. Title — Problem first line

**Medium**
2. Title — Problem first line
...

**Low**
8. Title — Problem first line
```

If a todo has no `**Priority:**` field, treat it as Medium. Show counts per tier at the end: e.g. "2 High · 6 Medium · 3 Low"

## Step 2 — Ask what to change

Ask: **"Which items would you like to reprioritise, and to what priority?"**

Accept any of these forms:
- `3 → High` — move item 3 to High
- `1 low, 4 high` — batch changes
- `swap 2 and 5` — exchange the two items' priorities
- `remove 6` — delete the todo entirely (confirm before deleting)
- `done` or nothing — exit with no changes

If the user says `done` or provides no input, stop here.

## Step 3 — Apply changes

For each change:
- Update the `**Priority:**` field in the matching todo block.
- Reorder the todos in the file so they stay sorted High → Medium → Low (preserve original order within each tier).
- If removing a todo, delete the entire `## Title` block including its trailing `---`.

After applying, show a brief confirmation of what changed, then redisplay the updated numbered list (same format as Step 1).

## Step 4 — Continue or done

Ask: **"Anything else to adjust?"**

- If yes, return to Step 2.
- If no, stop.
