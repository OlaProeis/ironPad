import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import {
  DEFAULT_BUDGET_HOURS,
  barFillRatio,
  readBudgetHours,
  writeBudgetHours,
} from './dayBudget'

describe('barFillRatio', () => {
  it('returns 0 when nothing is used', () => {
    expect(barFillRatio(0, 360)).toBe(0)
  })

  it('scales linearly up to the budget', () => {
    expect(barFillRatio(180, 360)).toBe(0.5)
    expect(barFillRatio(360, 360)).toBe(1)
  })

  it('caps at 1.0 when used exceeds budget', () => {
    expect(barFillRatio(400, 360)).toBe(1)
    expect(barFillRatio(720, 360)).toBe(1)
  })
})

describe('day budget persistence', () => {
  const store = new Map<string, string>()

  beforeEach(() => {
    store.clear()
    vi.stubGlobal('localStorage', {
      getItem: (key: string) => store.get(key) ?? null,
      setItem: (key: string, value: string) => {
        store.set(key, value)
      },
      removeItem: (key: string) => {
        store.delete(key)
      },
    })
  })

  afterEach(() => {
    vi.unstubAllGlobals()
  })

  it('defaults to 6 hours when nothing is stored', () => {
    expect(readBudgetHours()).toBe(DEFAULT_BUDGET_HOURS)
    expect(DEFAULT_BUDGET_HOURS).toBe(6)
  })

  it('round-trips valid budget hours via localStorage', () => {
    writeBudgetHours(8)
    expect(store.get('ironpad-day-budget-hours')).toBe('8')
    expect(readBudgetHours()).toBe(8)
  })

  it('falls back to 6 for invalid stored values', () => {
    store.set('ironpad-day-budget-hours', 'not-a-number')
    expect(readBudgetHours()).toBe(6)

    store.set('ironpad-day-budget-hours', '0')
    expect(readBudgetHours()).toBe(6)

    store.set('ironpad-day-budget-hours', '25')
    expect(readBudgetHours()).toBe(6)
  })

  it('clamps writes to a minimum of 1 hour', () => {
    writeBudgetHours(0)
    expect(store.get('ironpad-day-budget-hours')).toBe('1')
    expect(readBudgetHours()).toBe(1)

    writeBudgetHours(-5)
    expect(readBudgetHours()).toBe(1)
  })

  it('clamps writes to a maximum of 24 hours', () => {
    writeBudgetHours(48)
    expect(store.get('ironpad-day-budget-hours')).toBe('24')
    expect(readBudgetHours()).toBe(24)
  })
})