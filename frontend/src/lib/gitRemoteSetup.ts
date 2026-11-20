import { gitApi } from '../api/client'
import type { SetRemoteResponse } from '../types'

/** Placeholder until task 22 wires the doc route in the app shell. */
export const GIT_BACKUP_DOC_HREF = '/docs/git-backup'

export function trimRemoteUrlInput(raw: string): string {
  return raw.trim()
}

export async function submitSetRemote(url: string): Promise<SetRemoteResponse> {
  const trimmed = trimRemoteUrlInput(url)
  return gitApi.setRemote(trimmed)
}