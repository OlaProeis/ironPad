import { afterEach, describe, expect, it, vi } from 'vitest'
import { displayPullResult, submitPull } from './gitPull'

function jsonResponse(body: unknown, status = 200): Response {
  return new Response(JSON.stringify(body), {
    status,
    headers: { 'Content-Type': 'application/json' },
  })
}

afterEach(() => {
  vi.unstubAllGlobals()
})

describe('gitPull', () => {
  it('maps success payload to display text', () => {
    expect(
      displayPullResult({
        success: true,
        message: 'Pulled 2 commit(s).',
      })
    ).toEqual({
      success: true,
      message: 'Pulled 2 commit(s).',
    })
  })

  it('maps error payload to display text', () => {
    expect(
      displayPullResult({
        success: false,
        message: 'Your branch and the remote have diverged; pull would change local files.',
      })
    ).toEqual({
      success: false,
      message: 'Your branch and the remote have diverged; pull would change local files.',
    })
  })

  it('submits pull and surfaces success message from JSON', async () => {
    const fetchMock = vi.fn(async () =>
      jsonResponse({
        success: true,
        message: 'Already up to date.',
        commits: 0,
      })
    )
    vi.stubGlobal('fetch', fetchMock)

    const result = await submitPull()

    expect(result.success).toBe(true)
    expect(result.message).toBe('Already up to date.')
    expect(fetchMock).toHaveBeenCalledWith(
      '/api/git/pull',
      expect.objectContaining({ method: 'POST' })
    )
  })

  it('submits pull and surfaces mapped error on HTTP 400', async () => {
    vi.stubGlobal(
      'fetch',
      vi.fn(async () =>
        jsonResponse(
          {
            success: false,
            message: 'Commit or stash local changes before pulling.',
            commits: 0,
          },
          400
        )
      )
    )

    const result = await submitPull()

    expect(result.success).toBe(false)
    expect(result.message).toContain('Commit or stash')
  })
})