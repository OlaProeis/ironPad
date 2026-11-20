import { gitApi } from '../api/client'

export type GitPullDisplay = {
  success: boolean
  message: string
}

/** Map pull API payload to UI success/error display text. */
export function displayPullResult(result: {
  success: boolean
  message: string
}): GitPullDisplay {
  return {
    success: result.success,
    message: result.message,
  }
}

export async function submitPull(): Promise<GitPullDisplay> {
  const result = await gitApi.pull()
  return displayPullResult(result)
}