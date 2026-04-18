Work through a todo item from `todos.md` using the structured dev loop below.

## Step 1 — Pick a todo

Read `todos.md`. The user may invoke this command with varying levels of detail in $ARGUMENTS:

- **No arguments** — display a compact list grouped by priority (High → Medium → Low), showing title + first line of Problem for each. Ask: "Which todo do you want to work on?"
- **A keyword or partial title** (e.g. "remove button", "cursor") — find the best matching todo, state which one you matched, and confirm before proceeding. If ambiguous, show the candidates and ask.
- **A keyword plus extra context** (e.g. "cursor focus on the row heights first") — select that todo and treat the extra context as additional guidance for the implementation.

When displaying the compact list, use priority headings:
```
**High**
1. Title — Problem first line
...

**Medium**
5. Title — Problem first line
...

**Low**
8. Title — Problem first line
```

If a todo has no `**Priority:**` field, treat it as Medium.

Once selected:
- Run `/rename` with a short title based on the todo selected. Do not ask for confirmation — just rename.
- Restate the todo's **acceptance criteria** so both parties are aligned before any code is written.
- Ask: **"Anything else you want included in this session?"** Wait for the answer. If the user adds extra work, ask any clarifying questions needed before proceeding. If not, move on.

## Step 2 — Implement

Implement the changes needed to satisfy the acceptance criteria. Follow all conventions in `CLAUDE.md`:
- No business logic in `ui/` files
- No egui imports in `git/` or `state/`
- Use `git2`, not shell-out
- `tracing` for logging, never `println!`

Make all edits without asking for confirmation — just do the work.

## Step 3 — Build

Run `cargo build` in the `src/` directory. Fix any compile errors and rebuild until it succeeds. Report the result.

## Step 4 — Check in

Ask: **"Todo is implemented and builds clean. Is it done, or should I keep iterating?"**

- **Done** — remove the entire `## Title` block (through its trailing `---`) from `todos.md`, confirm: "Removed." Then go to Step 5.
- **Not done** — ask "What did you observe or what needs to change?" then return to Step 2 and repeat until confirmed done.

## Step 5 — Commit and push

Ask: **"Commit and push?"**

- **Yes** — stage all modified tracked files (`git add -u`), write a concise commit message summarising the change (no co-author line unless the user asks), push to the current branch's remote, and report the commit hash and push result.
- **No** — stop here.
