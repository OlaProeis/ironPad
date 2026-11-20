const STORAGE_KEY = 'ironpad-day-budget-hours'

export const DEFAULT_BUDGET_HOURS = 6
const MIN_BUDGET_HOURS = 1
const MAX_BUDGET_HOURS = 24

function parseStoredHours(raw: string): number | null {
  const trimmed = raw.trim()
  if (!/^-?\d+$/.test(trimmed)) return null
  const parsed = Number.parseInt(trimmed, 10)
  if (!Number.isInteger(parsed)) return null
  if (parsed < MIN_BUDGET_HOURS || parsed > MAX_BUDGET_HOURS) return null
  return parsed
}

export function readBudgetHours(): number {
  try {
    if (typeof localStorage === 'undefined') return DEFAULT_BUDGET_HOURS
    const stored = localStorage.getItem(STORAGE_KEY)
    if (stored === null) return DEFAULT_BUDGET_HOURS
    return parseStoredHours(stored) ?? DEFAULT_BUDGET_HOURS
  } catch {
    return DEFAULT_BUDGET_HOURS
  }
}

/** Progress bar fill: 0–1, capped at 1.0 when used exceeds budget. */
export function barFillRatio(used: number, budgetMinutes: number): number {
  if (budgetMinutes <= 0) return used > 0 ? 1 : 0
  if (used <= 0) return 0
  return Math.min(1, used / budgetMinutes)
}

export function writeBudgetHours(hours: number): void {
  let value = hours
  if (!Number.isFinite(value)) value = DEFAULT_BUDGET_HOURS
  value = Math.round(value)
  value = Math.min(MAX_BUDGET_HOURS, Math.max(MIN_BUDGET_HOURS, value))

  try {
    if (typeof localStorage === 'undefined') return
    localStorage.setItem(STORAGE_KEY, String(value))
  } catch {
    // private mode / blocked storage
  }
}