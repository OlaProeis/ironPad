import { afterEach, describe, expect, it, vi } from 'vitest'
import type { GithubSnapshotResponse } from '../../types'
import { githubApi } from '../client'

function jsonResponse(body: unknown, status = 200): Response {
  return new Response(JSON.stringify(body), {
    status,
    headers: { 'Content-Type': 'application/json' },
  })
}

afterEach(() => {
  vi.unstubAllGlobals()
})

describe('githubApi', () => {
  it('getSnapshot resolves to mocked snapshot response', async () => {
    const stub: GithubSnapshotResponse = {
      repo: 'owner/repo',
      login: 'octocat',
      refreshed_at: '2026-01-01T00:00:00Z',
      stale: false,
      issues: [],
      pulls: [],
      issues_truncated: false,
      pulls_truncated: false,
    }
    const fetchMock = vi.fn(async () => jsonResponse(stub))
    vi.stubGlobal('fetch', fetchMock)

    const result = await githubApi.getSnapshot('proj')

    expect(result).toEqual(stub)
    expect(fetchMock).toHaveBeenCalledWith('/api/projects/proj/github', undefined)
  })

  it('putRepo sends PUT with null repo JSON', async () => {
    const fetchMock = vi.fn(async () => new Response(null, { status: 204 }))
    vi.stubGlobal('fetch', fetchMock)

    await githubApi.putRepo('proj', null)

    expect(fetchMock).toHaveBeenCalledWith('/api/projects/proj/github', {
      method: 'PUT',
      headers: { 'Content-Type': 'application/json' },
      body: '{"repo":null}',
    })
  })
})