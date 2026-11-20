import { describe, expect, it } from 'vitest'
import { countActiveOverdueTasks } from './dashboardProjectStats'

const todayLocal = new Date(2024, 5, 20, 12, 0, 0)

describe('countActiveOverdueTasks', () => {
  it('counts only active, incomplete tasks with due_date before today', () => {
    const tasks = [
      {
        completed: false,
        is_active: true,
        due_date: '2024-06-19',
      },
      {
        completed: false,
        is_active: true,
        due_date: '2024-06-20',
      },
      {
        completed: false,
        is_active: true,
        due_date: '2024-06-21',
      },
    ]
    expect(countActiveOverdueTasks(tasks, todayLocal)).toBe(1)
  })

  it('ignores backlog, completed, and tasks without due_date', () => {
    const tasks = [
      { completed: false, is_active: false, due_date: '2024-01-01' },
      { completed: true, is_active: true, due_date: '2024-01-01' },
      { completed: false, is_active: true },
    ]
    expect(countActiveOverdueTasks(tasks, todayLocal)).toBe(0)
  })

  it('does not treat UTC midnight parsing quirks as overdue when due is today', () => {
    const eveningLocal = new Date(2024, 0, 15, 22, 30)
    const tasks = [
      { completed: false, is_active: true, due_date: '2024-01-15' },
    ]
    expect(countActiveOverdueTasks(tasks, eveningLocal)).toBe(0)
  })
})