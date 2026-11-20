import { describe, expect, it } from 'vitest'
import { isTaskOverdue } from './taskDates'

const todayLocal = new Date(2024, 5, 20, 23, 59, 0)

describe('isTaskOverdue', () => {
  it('returns false for completed tasks even with an old due date', () => {
    expect(
      isTaskOverdue({ completed: true, due_date: '2024-01-01' }, todayLocal),
    ).toBe(false)
  })

  it('returns false when due_date is missing', () => {
    expect(isTaskOverdue({ completed: false }, todayLocal)).toBe(false)
  })

  it('returns false for unparseable due_date values', () => {
    expect(
      isTaskOverdue({ completed: false, due_date: 'not-a-date' }, todayLocal),
    ).toBe(false)
    expect(
      isTaskOverdue({ completed: false, due_date: '2024-13-40' }, todayLocal),
    ).toBe(false)
  })

  it('returns false when due today (local calendar day)', () => {
    expect(
      isTaskOverdue({ completed: false, due_date: '2024-06-20' }, todayLocal),
    ).toBe(false)
  })

  it('returns true when due yesterday', () => {
    expect(
      isTaskOverdue({ completed: false, due_date: '2024-06-19' }, todayLocal),
    ).toBe(true)
  })

  it('uses local calendar parts of todayLocal, not UTC parsing of due_date', () => {
    const eveningLocal = new Date(2024, 0, 15, 22, 30)
    expect(
      isTaskOverdue({ completed: false, due_date: '2024-01-14' }, eveningLocal),
    ).toBe(true)
    expect(
      isTaskOverdue({ completed: false, due_date: '2024-01-15' }, eveningLocal),
    ).toBe(false)
  })
})