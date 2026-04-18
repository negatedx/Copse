Work through a todo item from `todos.md` using the structured dev loop below.

## Step 1 — Pick a todo

Read `todos.md`. The user may invoke this command with varying levels of detail in $ARGUMENTS:

- **No arguments** — display a compact list (number + title + first line of Problem) and ask: "Which todo number do you want to work on?"
- **A number** (e.g. "7") — select that todo immediately without asking.
- **A keyword or partial title** (e.g. "remove button", "cursor") — find the best matching todo, state which one you matched, and confirm before proceeding. If ambiguous, show the candidates and ask.
- **A number plus extra context** (e.g. "3 focus on the row heights first") — select that todo and treat the extra context as additional guidance for the implementation.

Once selected, restate the todo's **acceptance criteria** so both parties are aligned before any code is written.

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

Ask: **"Todo #N is implemented and builds clean. Is it done, or should I keep iterating?"**

- **Done** — remove the entire `## N. Title` block (through its trailing `---`) from `todos.md`, renumber remaining `## N.` headings sequentially, confirm: "Removed. Remaining todos renumbered." Then go to Step 5.
- **Not done** — ask "What did you observe or what needs to change?" then return to Step 2 and repeat until confirmed done.

## Step 5 — Commit and push

Ask: **"Commit and push?"**

- **Yes** — stage all modified tracked files (`git add -u`), write a concise commit message summarising the change (no co-author line unless the user asks), push to the current branch's remote, and report the commit hash and push result.
- **No** — stop here.
