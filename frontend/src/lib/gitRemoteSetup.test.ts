import { afterEach, describe, expect, it, vi } from 'vitest'
import { submitSetRemote, trimRemoteUrlInput } from './gitRemoteSetup'

function jsonResponse(body: unknown, status = 200): Response {
  return new Response(JSON.stringify(body), {
    status,
    headers: { 'Content-Type': 'application/json' },
  })
}

afterEach(() => {
  vi.unstubAllGlobals()
})

describe('gitRemoteSetup', () => {
  it('trims URL input before submit', () => {
    expect(trimRemoteUrlInput('  https://github.com/org/repo.git  ')).toBe(
      'https://github.com/org/repo.git'
    )
  })

  it('submits a valid URL and returns success payload', async () => {
    const fetchMock = vi.fn(async () =>
      jsonResponse({
        success: true,
        message: 'Remote origin was added.',
        url: 'https://github.com/org/repo.git',
        replaced: false,
        fetch_error: null,
      })
    )
    vi.stubGlobal('fetch', fetchMock)

    const result = await submitSetRemote(' https://github.com/org/repo.git ')

    expect(result.success).toBe(true)
    expect(result.url).toBe('https://github.com/org/repo.git')
    const [, init] = fetchMock.mock.calls[0] as [string, RequestInit]
    expect(JSON.parse(String(init.body))).toEqual({ url: 'https://github.com/org/repo.git' })
  })

  it('surfaces server validation errors without throwing', async () => {
    vi.stubGlobal(
      'fetch',
      vi.fn(async () =>
        jsonResponse(
          {
            success: false,
            message:
              'URLs must not embed credentials. Use SSH or a Git credential helper instead.',
            url: '',
            replaced: false,
            fetch_error: null,
          },
          400
        )
      )
    )

    const result = await submitSetRemote('https://token@github.com/org/repo.git')

    expect(result.success).toBe(false)
    expect(result.message).toContain('credentials')
  })
})