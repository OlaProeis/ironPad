# Day view working budget (frontend)

Release 0.4.0 (“Fit the day”) shows estimated minutes against a daily working budget in **Calendar day view**: used/budget label, progress bar, and an hours editor. Persistence and bar math live in `frontend/src/lib/dayBudget.ts`; UI in `frontend/src/views/CalendarView.vue`.

## Module

`frontend/src/lib/dayBudget.ts`

| Export | Role |
|--------|------|
| `DEFAULT_BUDGET_HOURS` | `6` — default and read fallback |
| `readBudgetHours()` | Returns stored hours or default |
| `writeBudgetHours(hours)` | Clamps, validates, and persists |
| `barFillRatio(used, budgetMinutes)` | Bar width ratio `0–1`, capped at **1.0** when used exceeds budget |

## Rules

| Case | Behavior |
|------|----------|
| Nothing stored | **6** hours |
| Valid stored value | Integer **1–24** |
| Corrupt or out-of-range stored value | **6** (no write on read) |
| `writeBudgetHours` input | Clamped to **1–24**; non-finite values are treated like the default before clamping |
| `localStorage` unavailable | Read returns **6**; write is a no-op |

Stored values must be integer strings (e.g. `"8"`). Non-integer strings such as `"6.5"` or `"abc"` fall back on read.

## Persistence

- Key: `ironpad-day-budget-hours`
- Value: decimal string of whole hours (e.g. `"6"`, `"24"`)

Follows the same `ironpad-*` localStorage pattern as task sort (`task-sort.md`).

## Day view UI (`CalendarView.vue`)

Only when `viewMode === 'day'` (month/week keep the existing per-cell total estimate chip).

| Element | Behavior |
|---------|----------|
| **Used** | `getTotalEstimateForDate` for the visible day — regular due tasks plus expanded recurring occurrences; missing `estimated_minutes` counts as **0** |
| **Budget** | `readBudgetHours() × 60` minutes (reactive `budgetHours` ref, refreshed after writes) |
| **Label** | `used / budget` (e.g. `4h / 6h`); shows overflow in the label when over budget |
| **Bar** | Fill width = `barFillRatio(used, budgetMinutes) × 100%`; stays full when over budget |
| **Over budget** | Label and bar use `var(--color-warning)` (amber); at or under budget, primary/neutral styling |
| **Editor** | Number input “Daily budget (hours)”, `min` 1 / `max` 24, `change` → `writeBudgetHours` |

## Tests

Vitest: `frontend/src/lib/dayBudget.test.ts` — persistence (default, round-trip, invalid parse, clamps) and `barFillRatio` (zero used, partial fill, cap at 1.0).

## Related

- Product scope: `docs/ai-workflow/PRD-0.4.0.md` (default 6h budget, 1–24h user range)
- Calendar estimate sums: `getTotalEstimateForDate` in `CalendarView.vue`