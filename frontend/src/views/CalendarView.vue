<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useTasksStore, useProjectsStore, useWorkspaceStore } from '../stores'
import { dailyApi } from '../api/client'
import type { Task, DailyNote } from '../types'

const router = useRouter()
const tasksStore = useTasksStore()
const projectsStore = useProjectsStore()
const workspaceStore = useWorkspaceStore()

// Current month being displayed
const currentYear = ref(new Date().getFullYear())
const currentMonth = ref(new Date().getMonth()) // 0-indexed

// Daily notes dates
const dailyDates = ref<Set<string>>(new Set())

const monthNames = [
  'January', 'February', 'March', 'April', 'May', 'June',
  'July', 'August', 'September', 'October', 'November', 'December'
]
const dayNames = ['Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat', 'Sun']

const monthLabel = computed(() => `${monthNames[currentMonth.value]} ${currentYear.value}`)

// Build calendar grid
const calendarDays = computed(() => {
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

// Tasks grouped by due date
const tasksByDate = computed(() => {
  const map = new Map<string, Task[]>()
  for (const task of tasksStore.allTasks) {
    if (task.due_date && !task.completed) {
      const existing = map.get(task.due_date) || []
      existing.push(task)
      map.set(task.due_date, existing)
    }
  }
  return map
})

function getTasksForDate(dateStr: string): Task[] {
  return tasksByDate.value.get(dateStr) || []
}

function hasDailyNote(dateStr: string): boolean {
  return dailyDates.value.has(dateStr)
}

// Navigation
function prevMonth() {
  if (currentMonth.value === 0) {
    currentMonth.value = 11
    currentYear.value--
  } else {
    currentMonth.value--
  }
}

function nextMonth() {
  if (currentMonth.value === 11) {
    currentMonth.value = 0
    currentYear.value++
  } else {
    currentMonth.value++
  }
}

function goToToday() {
  const today = new Date()
  currentYear.value = today.getFullYear()
  currentMonth.value = today.getMonth()
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
        <button @click="prevMonth" title="Previous month">&lsaquo;</button>
        <h2>{{ monthLabel }}</h2>
        <button @click="nextMonth" title="Next month">&rsaquo;</button>
        <button class="small today-btn" @click="goToToday">Today</button>
      </div>
    </div>

    <!-- Day names -->
    <div class="calendar-grid day-names">
      <div v-for="name in dayNames" :key="name" class="day-name">{{ name }}</div>
    </div>

    <!-- Calendar grid -->
    <div class="calendar-grid calendar-body">
      <div
        v-for="(day, idx) in calendarDays"
        :key="idx"
        :class="['calendar-cell', {
          'other-month': !day.isCurrentMonth,
          'is-today': day.isToday,
          'has-tasks': getTasksForDate(day.date).length > 0,
        }]"
      >
        <div class="cell-header" @click="clickDate(day.date)">
          <span class="cell-day">{{ day.day }}</span>
          <span v-if="hasDailyNote(day.date)" class="daily-dot" title="Daily note"></span>
        </div>
        <div class="cell-tasks">
          <div
            v-for="task in getTasksForDate(day.date).slice(0, 3)"
            :key="task.id"
            :class="['cell-task', formatDueClass(day.date)]"
            @click.stop="clickTask(task)"
            :title="`${projectName(task.project_id)}: ${task.title}`"
          >
            {{ task.title }}
          </div>
          <div
            v-if="getTasksForDate(day.date).length > 3"
            class="cell-more"
            @click="clickDate(day.date)"
          >
            +{{ getTasksForDate(day.date).length - 3 }} more
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
  grid-auto-rows: minmax(100px, 1fr);
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

.daily-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: var(--color-primary);
  flex-shrink: 0;
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
  cursor: pointer;
  transition: background 0.1s;
  flex-shrink: 0;
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
