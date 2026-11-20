import { afterEach, describe, expect, it, vi } from 'vitest'
import { libraryApi, promptsApi } from '../client'

function jsonResponse(body: unknown, status = 200): Response {
  return new Response(JSON.stringify(body), {
    status,
    headers: { 'Content-Type': 'application/json' },
  })
}

afterEach(() => {
  vi.unstubAllGlobals()
})

describe('libraryApi', () => {
  it('browseLibrary encodes path as query param', async () => {
    const fetchMock = vi.fn(async () =>
      jsonResponse({ path: '/x', parent: null, entries: [] })
    )
    vi.stubGlobal('fetch', fetchMock)

    await libraryApi.browseLibrary('/x')

    expect(fetchMock).toHaveBeenCalledWith(
      '/api/library/browse?path=%2Fx',
      undefined
    )
  })

  it('openLibraryItem POSTs id and root', async () => {
    const fetchMock = vi.fn(async () =>
      jsonResponse({ ok: true, resolved_path: '/root/file.md' })
    )
    vi.stubGlobal('fetch', fetchMock)

    await libraryApi.openLibraryItem('abc', '/root')

    expect(fetchMock).toHaveBeenCalledWith('/api/library/open', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ id: 'abc', root: '/root' }),
    })
  })

  it('pickLibrary POSTs kind and optional directory', async () => {
    const fetchMock = vi.fn(async () => jsonResponse({ paths: ['C:/repo'] }))
    vi.stubGlobal('fetch', fetchMock)

    await libraryApi.pickLibrary('directory', 'C:/repo')

    expect(fetchMock).toHaveBeenCalledWith('/api/library/pick', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ kind: 'directory', directory: 'C:/repo' }),
    })
  })
})

describe('promptsApi.get with root', () => {
  it('appends root query param when provided', async () => {
    const fetchMock = vi.fn(async () =>
      jsonResponse({
        id: 'p1',
        title: 'T',
        path: 'data/prompts/p1.md',
        scope: 'global',
        folder: '',
        tags: [],
        kind: 'prompt',
        template: false,
        frontmatter: {},
        content: '',
        read_only: false,
        source_missing: false,
      })
    )
    vi.stubGlobal('fetch', fetchMock)

    await promptsApi.get('p1', '/my/project/root')

    expect(fetchMock).toHaveBeenCalledWith(
      '/api/prompts/p1?root=%2Fmy%2Fproject%2Froot',
      undefined
    )
  })
})