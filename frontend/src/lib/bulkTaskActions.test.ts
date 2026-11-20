import { afterEach, describe, expect, it, vi } from 'vitest'
import {
  metaForSectionMove,
  normalizeTag,
  runBulkComplete,
  runBulkMetaUpdates,
  tagsAfterAdd,
  tagsAfterRemove,
  tasksForBulkComplete,
} from './bulkTaskActions'

function jsonResponse(body: unknown, status = 200): Response {
  return new Response(JSON.stringify(body), {
    status,
    headers: { 'Content-Type': 'application/json' },
  })
}

afterEach(() => {
  vi.unstubAllGlobals()
})

describe('bulk task helpers', () => {
  it('skips already completed tasks for bulk complete', () => {
    const tasks = [
      { id: 'a', title: 'Open', completed: false },
      { id: 'b', title: 'Done', completed: true },
    ]
    expect(tasksForBulkComplete(tasks).map((task) => task.id)).toEqual(['a'])
  })

  it('rejects an empty tag and dedupes add/remove', () => {
    expect(normalizeTag('  ')).toBeNull()
    expect(normalizeTag(' bug ')).toBe('bug')
    expect(tagsAfterAdd(['bug'], 'bug')).toEqual(['bug'])
    expect(tagsAfterAdd(['bug'], 'ui')).toEqual(['bug', 'ui'])
    expect(tagsAfterRemove(['bug', 'ui'], 'bug')).toEqual(['ui'])
  })

  it('sends completed false when moving a completed task', () => {
    expect(metaForSectionMove({ completed: true }, 'Backlog')).toEqual({
      section: 'Backlog',
      completed: false,
    })
    expect(metaForSectionMove({ completed: false }, 'Active')).toEqual({ section: 'Active' })
  })
})

describe('runBulkComplete', () => {
  it('does not call toggle for completed tasks and stops on the first error', async () => {
    const fetchMock = vi.fn(async (input: RequestInfo | URL) => {
      const url = String(input)
      if (url.endsWith('/tasks/second/toggle')) {
        return new Response('locked', { status: 409 })
      }
      return jsonResponse({ id: 'ok' })
    })
    vi.stubGlobal('fetch', fetchMock)

    const result = await runBulkComplete('proj', [
      { id: 'done', title: 'Already done', completed: true },
      { id: 'first', title: 'First', completed: false },
      { id: 'second', title: 'Second', completed: false },
      { id: 'third', title: 'Third', completed: false },
    ])

    expect(result.ok).toBe(false)
    expect(result.applied).toBe(1)
    expect(result.failure).toEqual({ title: 'Second', error: 'locked' })
    const urls = fetchMock.mock.calls.map((call) => String(call[0]))
    expect(urls.some((url) => url.includes('/tasks/done/'))).toBe(false)
    expect(urls.some((url) => url.includes('/tasks/third/'))).toBe(false)
    expect(urls.filter((url) => url.includes('/toggle'))).toEqual([
      '/api/projects/proj/tasks/first/toggle',
      '/api/projects/proj/tasks/second/toggle',
    ])
  })
})

describe('runBulkMetaUpdates', () => {
  it('stops on the first meta failure and leaves earlier requests sent', async () => {
    const bodies: string[] = []
    const fetchMock = vi.fn(async (input: RequestInfo | URL, init?: RequestInit) => {
      const url = String(input)
      bodies.push(String(init?.body ?? ''))
      if (url.endsWith('/tasks/b/meta')) {
        return new Response('nope', { status: 400 })
      }
      return jsonResponse({ id: 'ok' })
    })
    vi.stubGlobal('fetch', fetchMock)

    const result = await runBulkMetaUpdates('proj', [
      { id: 'a', title: 'Alpha', meta: { section: 'Active', completed: false } },
      { id: 'b', title: 'Beta', meta: { tags: ['x'] } },
      { id: 'c', title: 'Gamma', meta: { section: 'Backlog' } },
    ])

    expect(result.ok).toBe(false)
    expect(result.applied).toBe(1)
    expect(result.failure?.title).toBe('Beta')
    expect(result.failure?.error).toBe('nope')
    expect(fetchMock).toHaveBeenCalledTimes(2)
    expect(bodies[0]).toContain('"completed":false')
    expect(bodies[1]).toContain('"tags"')
  })
})
