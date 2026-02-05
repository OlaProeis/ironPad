import { createRouter, createWebHistory } from 'vue-router'

const router = createRouter({
  history: createWebHistory(),
  routes: [
    {
      path: '/',
      name: 'home',
      component: () => import('../views/DashboardView.vue')
    },
    {
      path: '/projects',
      name: 'projects',
      component: () => import('../views/ProjectsView.vue')
    },
    {
      path: '/projects/:id',
      name: 'project',
      component: () => import('../views/ProjectView.vue'),
      props: true
    },
    {
      path: '/projects/:id/notes/:noteId?',
      name: 'project-notes',
      component: () => import('../views/ProjectNotesView.vue'),
      props: true
    },
    {
      path: '/projects/:id/tasks/:taskId?',
      name: 'project-tasks',
      component: () => import('../views/TasksView.vue'),
      props: true
    },
    {
      path: '/calendar',
      name: 'calendar',
      component: () => import('../views/CalendarView.vue')
    },
    {
      path: '/daily',
      name: 'daily',
      component: () => import('../views/DailyView.vue')
    },
    {
      path: '/daily/:date',
      name: 'daily-note',
      component: () => import('../views/DailyView.vue'),
      props: true
    }
  ]
})

export default router
