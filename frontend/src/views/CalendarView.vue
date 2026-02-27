<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useTasksStore, useProjectsStore, useWorkspaceStore } from '../stores'
import { dailyApi, tasksApi } from '../api/client'
import type { Task, DailyNote } from '../types'

const router = useRouter()
const tasksStore = useTasksStore()
const projectsStore = useProjectsStore()
const workspaceStore = useWorkspaceStore()

// Drag and drop state
const draggedTask = ref<Task | null>(null)
const dragOverDate = ref<string | null>(null)

// View mode: 'month' | 'week' | 'day'
type ViewMode = 'month' | 'week' | 'day'
const viewMode = ref<ViewMode>('month')

// Current date being displayed (month/week/day centered around this)
const currentYear = ref(new Date().getFullYear())
const currentMonth = ref(new Date().getMonth()) // 0-indexed
const currentDay = ref(new Date().getDate())

// For week/day navigation - stores the specific date
const currentDate = computed(() => new Date(currentYear.value, currentMonth.value, currentDay.value))

// Daily notes dates
const dailyDates = ref<Set<string>>(new Set())

const monthNames = [
  'January', 'February', 'March', 'April', 'May', 'June',
  'July', 'August', 'September', 'October', 'November', 'December'
]
const dayNames = ['Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat', 'Sun']

const monthLabel = computed(() => {
  if (viewMode.value === 'month') {
    return `${monthNames[currentMonth.value]} ${currentYear.value}`
  } else if (viewMode.value === 'week') {
    const weekStart = getWeekStart(currentDate.value)
    const weekEnd = new Date(weekStart)
    weekEnd.setDate(weekEnd.getDate() + 6)
    
    const startMonth = monthNames[weekStart.getMonth()]
    const endMonth = monthNames[weekEnd.getMonth()]
    const startYear = weekStart.getFullYear()
    const endYear = weekEnd.getFullYear()
    
    if (startMonth === endMonth && startYear === endYear) {
      return `${startMonth} ${startYear}`
    } else if (startYear === endYear) {
      return `${startMonth} - ${endMonth} ${startYear}`
    } else {
      return `${startMonth} ${startYear} - ${endMonth} ${endYear}`
    }
  } else {
    // day view
    return `${monthNames[currentMonth.value]} ${currentDay.value}, ${currentYear.value}`
  }
})

// Build calendar grid based on view mode
const calendarDays = computed(() => {
  if (viewMode.value === 'week') {
    return getWeekDays(currentDate.value)
  } else if (viewMode.value === 'day') {
    const today = new Date()
    const todayStr = formatDate(today.getFullYear(), today.getMonth(), today.getDate())
    const dateStr = formatDate(currentYear.value, currentMonth.value, currentDay.value)
    return [{
      date: dateStr,
      day: currentDay.value,
      isCurrentMonth: currentMonth.value === today.getMonth() && currentYear.value === today.getFullYear(),
      isToday: dateStr === todayStr,
    }]
  }
  
  // Month view (default)
  const year = currentYear.value
  const month = currentMonth.value

  // First day of the month
  const firstDay = new Date(year, month, 1)
  // Day of week (0=Sun, 1=Mon...) - shift to Mon=0
  let startDow = firstDay.getDay() - 1
  if (startDow < 0) startDow = 6

  // Days in this month
  const daysInMonth = new Date(year, month + 1, 0).getDate()

  // Days in previous month (for padding)
  const daysInPrevMonth = new Date(year, month, 0).getDate()

  const days: { date: string; day: number; isCurrentMonth: boolean; isToday: boolean }[] = []

  // Previous month padding
  for (let i = startDow - 1; i >= 0; i--) {
    const d = daysInPrevMonth - i
    const m = month === 0 ? 11 : month - 1
    const y = month === 0 ? year - 1 : year
    days.push({
      date: formatDate(y, m, d),
      day: d,
      isCurrentMonth: false,
      isToday: false,
    })
  }

  // Current month
  const today = new Date()
  const todayStr = formatDate(today.getFullYear(), today.getMonth(), today.getDate())

  for (let d = 1; d <= daysInMonth; d++) {
    const dateStr = formatDate(year, month, d)
    days.push({
      date: dateStr,
      day: d,
      isCurrentMonth: true,
      isToday: dateStr === todayStr,
    })
  }

  // Next month padding (fill to 6 rows * 7 = 42 cells, or at least complete the row)
  const remaining = 7 - (days.length % 7)
  if (remaining < 7) {
    for (let d = 1; d <= remaining; d++) {
      const m = month === 11 ? 0 : month + 1
      const y = month === 11 ? year + 1 : year
      days.push({
        date: formatDate(y, m, d),
        day: d,
        isCurrentMonth: false,
        isToday: false,
      })
    }
  }

  return days
})

function formatDate(y: number, m: number, d: number): string {
  return `${y}-${String(m + 1).padStart(2, '0')}-${String(d).padStart(2, '0')}`
}

// Week helper functions
function getWeekStart(date: Date): Date {
  const d = new Date(date)
  const day = d.getDay()
  const diff = d.getDate() - day + (day === 0 ? -6 : 1) // Adjust when day is Sunday
  return new Date(d.setDate(diff))
}

function getWeekDays(date: Date): { date: string; day: number; isCurrentMonth: boolean; isToday: boolean }[] {
  const weekStart = getWeekStart(date)
  const today = new Date()
  const todayStr = formatDate(today.getFullYear(), today.getMonth(), today.getDate())
  const days: { date: string; day: number; isCurrentMonth: boolean; isToday: boolean }[] = []
  
  for (let i = 0; i < 7; i++) {
    const d = new Date(weekStart)
    d.setDate(weekStart.getDate() + i)
    const dateStr = formatDate(d.getFullYear(), d.getMonth(), d.getDate())
    days.push({
      date: dateStr,
      day: d.getDate(),
      isCurrentMonth: d.getMonth() === currentMonth.value && d.getFullYear() === currentYear.value,
      isToday: dateStr === todayStr,
    })
  }
  
  return days
}

// Calendar entry: a task plus whether it's a recurring occurrence (vs a real due_date)
interface CalendarEntry {
  task: Task
  isRecurring: boolean
}

// Tasks grouped by due date (regular, non-recurring placements)
const tasksByDate = computed(() => {
  const map = new Map<string, CalendarEntry[]>()
  for (const task of tasksStore.allTasks) {
    if (task.due_date && !task.completed) {
      const existing = map.get(task.due_date) || []
      existing.push({ task, isRecurring: false })
      map.set(task.due_date, existing)
    }
  }
  return map
})

// Expand recurring tasks into the visible calendar range
const recurringByDate = computed(() => {
  const map = new Map<string, CalendarEntry[]>()
  const days = calendarDays.value
  if (days.length === 0) return map

  const rangeStart = days[0].date
  const rangeEnd = days[days.length - 1].date

  for (const task of tasksStore.allTasks) {
    if (task.completed || !task.recurrence) continue

    const interval = task.recurrence_interval || 1
    const anchorStr = task.due_date || task.created?.split('T')[0]
    if (!anchorStr) continue

    const anchor = new Date(anchorStr + 'T00:00:00')
    const start = new Date(rangeStart + 'T00:00:00')
    const end = new Date(rangeEnd + 'T00:00:00')
    const occurrences: string[] = []

    switch (task.recurrence) {
      case 'daily': {
        let cur = new Date(anchor)
        // Advance to range start (skip past occurrences efficiently)
        if (cur < start) {
          const daysBehind = Math.floor((start.getTime() - cur.getTime()) / 86400000)
          const skipCycles = Math.floor(daysBehind / interval) * interval
          cur.setDate(cur.getDate() + skipCycles)
        }
        while (cur <= end && occurrences.length < 60) {
          if (cur >= start) {
            occurrences.push(dateFromObj(cur))
          }
          cur.setDate(cur.getDate() + interval)
        }
        break
      }
      case 'weekly': {
        const targetDow = anchor.getDay()
        let cur = new Date(start)
        // Find first matching weekday in range
        while (cur.getDay() !== targetDow && cur <= end) {
          cur.setDate(cur.getDate() + 1)
        }
        while (cur <= end && occurrences.length < 10) {
          // Check interval alignment (weeks since anchor)
          const msDiff = cur.getTime() - anchor.getTime()
          const weeksDiff = Math.round(msDiff / (7 * 86400000))
          if (weeksDiff >= 0 && weeksDiff % interval === 0) {
            occurrences.push(dateFromObj(cur))
          }
          cur.setDate(cur.getDate() + 7)
        }
        break
      }
      case 'monthly': {
        const targetDay = anchor.getDate()
        // Check months visible in the calendar range (usually 3: prev, current, next)
        for (let mOffset = -1; mOffset <= 1; mOffset++) {
          let y = currentYear.value
          let m = currentMonth.value + mOffset
          if (m < 0) { m = 11; y-- }
          if (m > 11) { m = 0; y++ }

          const daysInM = new Date(y, m + 1, 0).getDate()
          const day = Math.min(targetDay, daysInM)
          const dateStr = formatDate(y, m, day)

          if (dateStr >= rangeStart && dateStr <= rangeEnd) {
            const monthsDiff = (y - anchor.getFullYear()) * 12 + (m - anchor.getMonth())
            if (monthsDiff >= 0 && monthsDiff % interval === 0) {
              occurrences.push(dateStr)
            }
          }
        }
        break
      }
      case 'yearly': {
        const tgtMonth = anchor.getMonth()
        const tgtDay = anchor.getDate()
        const y = currentYear.value
        const dateStr = formatDate(y, tgtMonth, tgtDay)
        if (dateStr >= rangeStart && dateStr <= rangeEnd) {
          const yearsDiff = y - anchor.getFullYear()
          if (yearsDiff >= 0 && yearsDiff % interval === 0) {
            occurrences.push(dateStr)
          }
        }
        break
      }
    }

    for (const dateStr of occurrences) {
      // Skip if the task already appears on this date via its due_date
      if (task.due_date === dateStr) continue
      const existing = map.get(dateStr) || []
      existing.push({ task, isRecurring: true })
      map.set(dateStr, existing)
    }
  }

  return map
})

function dateFromObj(d: Date): string {
  return formatDate(d.getFullYear(), d.getMonth(), d.getDate())
}

// Merge regular due-date tasks and recurring occurrences for a given date
function getEntriesForDate(dateStr: string): CalendarEntry[] {
  const regular = tasksByDate.value.get(dateStr) || []
  const recurring = recurringByDate.value.get(dateStr) || []
  return [...regular, ...recurring]
}

// Convenience: check if a date has any tasks (for cell styling)
function hasTasksOnDate(dateStr: string): boolean {
  return getEntriesForDate(dateStr).length > 0
}

// Calculate total estimated time for a date
function getTotalEstimateForDate(dateStr: string): number {
  const entries = getEntriesForDate(dateStr)
  return entries.reduce((total, entry) => total + (entry.task.estimated_minutes || 0), 0)
}

function formatEstimate(minutes: number): string {
  if (minutes === 0) return ''
  if (minutes < 60) return `${minutes}m`
  const hours = Math.floor(minutes / 60)
  const mins = minutes % 60
  if (mins === 0) return `${hours}h`
  return `${hours}h ${mins}m`
}

function hasDailyNote(dateStr: string): boolean {
  return dailyDates.value.has(dateStr)
}

// Navigation
function prevPeriod() {
  if (viewMode.value === 'month') {
    if (currentMonth.value === 0) {
      currentMonth.value = 11
      currentYear.value--
    } else {
      currentMonth.value--
    }
  } else if (viewMode.value === 'week') {
    const newDate = new Date(currentDate.value)
    newDate.setDate(newDate.getDate() - 7)
    currentYear.value = newDate.getFullYear()
    currentMonth.value = newDate.getMonth()
    currentDay.value = newDate.getDate()
  } else {
    // day view
    const newDate = new Date(currentDate.value)
    newDate.setDate(newDate.getDate() - 1)
    currentYear.value = newDate.getFullYear()
    currentMonth.value = newDate.getMonth()
    currentDay.value = newDate.getDate()
  }
}

function nextPeriod() {
  if (viewMode.value === 'month') {
    if (currentMonth.value === 11) {
      currentMonth.value = 0
      currentYear.value++
    } else {
      currentMonth.value++
    }
  } else if (viewMode.value === 'week') {
    const newDate = new Date(currentDate.value)
    newDate.setDate(newDate.getDate() + 7)
    currentYear.value = newDate.getFullYear()
    currentMonth.value = newDate.getMonth()
    currentDay.value = newDate.getDate()
  } else {
    // day view
    const newDate = new Date(currentDate.value)
    newDate.setDate(newDate.getDate() + 1)
    currentYear.value = newDate.getFullYear()
    currentMonth.value = newDate.getMonth()
    currentDay.value = newDate.getDate()
  }
}

function goToToday() {
  const today = new Date()
  currentYear.value = today.getFullYear()
  currentMonth.value = today.getMonth()
  currentDay.value = today.getDate()
}

function setViewMode(mode: ViewMode) {
  viewMode.value = mode
}

function clickDate(dateStr: string) {
  // Navigate to daily note for this date
  router.push({ name: 'daily-note', params: { date: dateStr } })
}

function clickTask(task: Task) {
  workspaceStore.setActiveProject(task.project_id)
  router.push({
    name: 'project-tasks',
    params: { id: task.project_id, taskId: task.id }
  })
}

function projectName(projectId: string): string {
  const p = projectsStore.getProjectById(projectId)
  return p?.name || projectId
}

function formatDueClass(dateStr: string): string {
  const now = new Date()
  const date = new Date(dateStr)
  const diff = Math.ceil((date.getTime() - now.getTime()) / (1000 * 60 * 60 * 24))
  if (diff < 0) return 'overdue'
  if (diff === 0) return 'today'
  if (diff <= 3) return 'soon'
  return ''
}

// Drag and drop handlers
function onDragStart(task: Task, event: DragEvent) {
  draggedTask.value = task
  if (event.dataTransfer) {
    event.dataTransfer.effectAllowed = 'move'
    event.dataTransfer.setData('text/plain', task.id)
  }
}

function onDragEnd() {
  draggedTask.value = null
  dragOverDate.value = null
}

function onDragOver(dateStr: string, event: DragEvent) {
  event.preventDefault()
  if (event.dataTransfer) {
    event.dataTransfer.dropEffect = 'move'
  }
  dragOverDate.value = dateStr
}

function onDragLeave() {
  dragOverDate.value = null
}

async function onDrop(dateStr: string, event: DragEvent) {
  event.preventDefault()
  dragOverDate.value = null

  if (!draggedTask.value) return

  const task = draggedTask.value
  const projectId = task.project_id
  const taskId = task.id
  const oldDate = task.due_date

  // Don't update if dropping on the same date
  if (oldDate === dateStr) {
    draggedTask.value = null
    return
  }

  try {
    // Update the task's due date
    await tasksApi.updateMeta(projectId, taskId, { due_date: dateStr })

    // Refresh tasks to reflect the change
    await tasksStore.loadAllTasks()

    // Show brief feedback (could be a toast notification in the future)
    console.log(`Rescheduled "${task.title}" to ${dateStr}`)
  } catch (err) {
    console.error('Failed to reschedule task:', err)
    alert('Failed to reschedule task. Please try again.')
  }

  draggedTask.value = null
}

// Load data
async function loadDailyDates() {
  try {
    const notes: DailyNote[] = await dailyApi.list()
    dailyDates.value = new Set(notes.map(n => n.date))
  } catch {
    // Ignore
  }
}

onMounted(async () => {
  await Promise.all([
    tasksStore.loadAllTasks(),
    projectsStore.loadProjects(),
    loadDailyDates(),
  ])
})
</script>

<template>
  <div class="calendar-view">
    <!-- Header -->
    <div class="calendar-header">
      <div class="header-left">
        <button @click="prevPeriod" title="Previous">&lsaquo;</button>
        <h2>{{ monthLabel }}</h2>
        <button @click="nextPeriod" title="Next">&rsaquo;</button>
        <button class="small today-btn" @click="goToToday">Today</button>
      </div>
      <div class="header-right">
        <div class="view-toggle">
          <button 
            :class="['view-btn', { active: viewMode === 'month' }]" 
            @click="setViewMode('month')"
            title="Month view"
          >Month</button>
          <button 
            :class="['view-btn', { active: viewMode === 'week' }]" 
            @click="setViewMode('week')"
            title="Week view"
          >Week</button>
          <button 
            :class="['view-btn', { active: viewMode === 'day' }]" 
            @click="setViewMode('day')"
            title="Day view"
          >Day</button>
        </div>
      </div>
    </div>

    <!-- Day names (hidden in day view) -->
    <div v-if="viewMode !== 'day'" class="calendar-grid day-names" :class="{ 'week-view': viewMode === 'week' }">
      <div v-for="name in dayNames" :key="name" class="day-name">{{ name }}</div>
    </div>

    <!-- Calendar grid -->
    <div 
      class="calendar-grid calendar-body" 
      :class="{ 
        'month-layout': viewMode === 'month',
        'week-layout': viewMode === 'week',
        'day-layout': viewMode === 'day'
      }"
    >
      <div
        v-for="(day, idx) in calendarDays"
        :key="idx"
        :class="['calendar-cell', {
          'other-month': !day.isCurrentMonth && viewMode === 'month',
          'is-today': day.isToday,
          'has-tasks': hasTasksOnDate(day.date),
          'drag-over': dragOverDate === day.date,
        }]"
        @dragover="onDragOver(day.date, $event)"
        @dragleave="onDragLeave"
        @drop="onDrop(day.date, $event)"
      >
        <div class="cell-header" @click="clickDate(day.date)">
          <span class="cell-day">{{ day.day }}</span>
          <span v-if="viewMode !== 'month'" class="cell-weekday">{{ dayNames[idx] }}</span>
          <span v-if="hasDailyNote(day.date)" class="daily-dot" title="Daily note"></span>
          <span v-if="getTotalEstimateForDate(day.date) > 0" class="cell-total-estimate">
            {{ formatEstimate(getTotalEstimateForDate(day.date)) }}
          </span>
        </div>
        <div class="cell-tasks">
          <div
            v-for="entry in getEntriesForDate(day.date)"
            :key="`${entry.task.id}-${entry.isRecurring ? 'r' : 'd'}`"
            :class="['cell-task', formatDueClass(day.date), { recurring: entry.isRecurring }]"
            @click.stop="clickTask(entry.task)"
            :title="`${projectName(entry.task.project_id)}: ${entry.task.title}${entry.isRecurring ? ' (recurring)' : ''}`"
            draggable="true"
            @dragstart="onDragStart(entry.task, $event)"
            @dragend="onDragEnd"
          >
            <span v-if="entry.isRecurring" class="recurring-icon" title="Recurring">&#x21bb;</span>
            {{ entry.task.title }}
            <span v-if="entry.task.estimated_minutes" class="task-estimate">
              {{ formatEstimate(entry.task.estimated_minutes) }}
            </span>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.calendar-view {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.calendar-header {
  height: var(--header-height);
  min-height: var(--header-height);
  padding: 0 24px;
  border-bottom: 1px solid var(--color-border);
  display: flex;
  align-items: center;
  flex-shrink: 0;
}

.header-left {
  display: flex;
  align-items: center;
  gap: 12px;
}

.header-left button {
  font-size: 18px;
  padding: 4px 10px;
  line-height: 1;
}

.today-btn {
  font-size: 12px !important;
  padding: 4px 10px !important;
}

.calendar-header h2 {
  font-size: 16px;
  font-weight: 600;
  margin: 0;
  min-width: 180px;
  text-align: center;
}

.header-right {
  display: flex;
  align-items: center;
}

.view-toggle {
  display: flex;
  gap: 0;
  border: 1px solid var(--color-border);
  border-radius: 4px;
  overflow: hidden;
}

.view-btn {
  font-size: 12px;
  padding: 4px 12px;
  border: none;
  border-right: 1px solid var(--color-border);
  background: var(--color-bg-secondary);
  color: var(--color-text-secondary);
  cursor: pointer;
  transition: background 0.15s, color 0.15s;
}

.view-btn:last-child {
  border-right: none;
}

.view-btn:hover {
  background: var(--color-bg-hover);
  color: var(--color-text);
}

.view-btn.active {
  background: var(--color-primary);
  color: white;
}

/* Grid layout */
.calendar-grid {
  display: grid;
  grid-template-columns: repeat(7, 1fr);
}

.day-names {
  border-bottom: 1px solid var(--color-border);
  flex-shrink: 0;
}

.day-name {
  padding: 8px 4px;
  text-align: center;
  font-size: 11px;
  font-weight: 600;
  color: var(--color-text-secondary);
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.calendar-body {
  flex: 1;
  overflow-y: auto;
}

/* Month view - 7 columns, auto rows */
.calendar-body.month-layout {
  grid-auto-rows: minmax(100px, 1fr);
}

/* Week view - 7 columns side by side */
.calendar-body.week-layout {
  grid-template-columns: repeat(7, 1fr);
  grid-auto-rows: 1fr;
}

.calendar-body.week-layout .calendar-cell {
  min-height: 200px;
}

/* Day view - single column */
.calendar-body.day-layout {
  grid-template-columns: 1fr;
}

.calendar-body.day-layout .calendar-cell {
  min-height: 300px;
  padding: 16px;
}

.calendar-body.day-layout .cell-header {
  font-size: 24px;
  margin-bottom: 16px;
}

.calendar-body.day-layout .cell-day {
  font-size: 32px;
  font-weight: 600;
}

/* Calendar cells */
.calendar-cell {
  border-right: 1px solid var(--color-border);
  border-bottom: 1px solid var(--color-border);
  padding: 4px;
  min-height: 100px;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.calendar-cell:nth-child(7n) {
  border-right: none;
}

.calendar-cell.other-month {
  opacity: 0.35;
}

.calendar-cell.is-today {
  background: rgba(88, 166, 255, 0.08);
}

.calendar-cell.drag-over {
  background: rgba(88, 166, 255, 0.15);
  box-shadow: inset 0 0 0 2px var(--color-primary);
}

.calendar-cell.is-today .cell-day {
  background: var(--color-primary);
  color: white;
  border-radius: 50%;
  width: 24px;
  height: 24px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.cell-header {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 2px 4px;
  cursor: pointer;
  border-radius: 4px;
  flex-shrink: 0;
}

.cell-header:hover {
  background: var(--color-bg-hover);
}

.cell-day {
  font-size: 13px;
  font-weight: 500;
}

.cell-weekday {
  font-size: 11px;
  font-weight: 400;
  color: var(--color-text-secondary);
  margin-left: 4px;
}

.daily-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: var(--color-primary);
  flex-shrink: 0;
}

.cell-total-estimate {
  font-size: 10px;
  font-weight: 500;
  color: var(--color-primary);
  margin-left: auto;
  padding: 1px 5px;
  background: rgba(88, 166, 255, 0.1);
  border-radius: 4px;
  border: 1px solid rgba(88, 166, 255, 0.2);
}

/* Tasks in cells */
.cell-tasks {
  flex: 1;
  overflow: hidden;
  display: flex;
  flex-direction: column;
  gap: 2px;
  padding-top: 2px;
}

.cell-task {
  font-size: 11px;
  padding: 2px 6px;
  border-radius: 4px;
  background: var(--color-bg-secondary);
  border-left: 2px solid var(--color-primary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  cursor: grab;
  transition: background 0.1s, opacity 0.15s;
  flex-shrink: 0;
  user-select: none;
}

.cell-task:active {
  cursor: grabbing;
}

.cell-task:hover {
  background: var(--color-bg-hover);
}

.cell-task.overdue {
  border-left-color: var(--color-danger);
  color: var(--color-danger);
}

.cell-task.today {
  border-left-color: var(--color-danger);
}

.cell-task.soon {
  border-left-color: var(--color-warning);
}

.cell-task.recurring {
  border-left-style: dashed;
  opacity: 0.85;
}

.recurring-icon {
  font-size: 10px;
  margin-right: 2px;
  opacity: 0.7;
}

.task-estimate {
  font-size: 9px;
  color: var(--color-text-secondary);
  opacity: 0.8;
  margin-left: 4px;
  white-space: nowrap;
}

.cell-more {
  font-size: 10px;
  color: var(--color-text-secondary);
  padding: 2px 6px;
  cursor: pointer;
}

.cell-more:hover {
  color: var(--color-primary);
}
</style>
