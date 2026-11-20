const MINUTE_MS = 60 * 1000
const HOUR_MS = 60 * MINUTE_MS
const DAY_MS = 24 * HOUR_MS
const WEEK_MS = 7 * DAY_MS
const MONTH_MS = 30 * DAY_MS
const YEAR_MS = 365 * DAY_MS

function plural(value: number, unit: string): string {
  return `${value} ${unit}${value === 1 ? '' : 's'} ago`
}

/** Relative time from an ISO 8601 timestamp to now (pure Date arithmetic). */
export function relativeTime(isoString: string): string {
  const then = new Date(isoString).getTime()
  if (Number.isNaN(then)) {
    return ''
  }
  const diff = Date.now() - then
  if (diff < 0) {
    return 'just now'
  }
  if (diff < MINUTE_MS) {
    return 'just now'
  }
  if (diff < HOUR_MS) {
    return plural(Math.floor(diff / MINUTE_MS), 'minute')
  }
  if (diff < DAY_MS) {
    return plural(Math.floor(diff / HOUR_MS), 'hour')
  }
  if (diff < WEEK_MS) {
    return plural(Math.floor(diff / DAY_MS), 'day')
  }
  if (diff < MONTH_MS) {
    return plural(Math.floor(diff / WEEK_MS), 'week')
  }
  if (diff < YEAR_MS) {
    return plural(Math.floor(diff / MONTH_MS), 'month')
  }
  return plural(Math.floor(diff / YEAR_MS), 'year')
}