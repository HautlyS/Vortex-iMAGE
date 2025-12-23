/**
 * TypeScript Module - 1 exports
 * Purpose: Type-safe utilities and composable functions
 * Imports: 1 modules
 */

import { ref, computed } from 'vue'
import { load } from '@tauri-apps/plugin-store'

export type DriverType = 'github' | 'local'

export interface DataDriver {
  id: string
  type: DriverType
  name: string
  path: string 
  isActive: boolean
  createdAt: number
  lastSyncAt?: number
  photoCount?: number
  icon?: string
}

export interface LocalFolderInfo {
  path: string
  name: string
  photoCount: number
  totalSize: number
  lastModified: number
}

const drivers = ref<DataDriver[]>([])
const activeDriverId = ref<string | null>(null)
const loading = ref(false)
let initialized = false

export function useDataDriver() {
  const activeDriver = computed(() => 
    drivers.value.find(d => d.id === activeDriverId.value) || null
  )

  const githubDrivers = computed(() => 
    drivers.value.filter(d => d.type === 'github')
  )

  const localDrivers = computed(() => 
    drivers.value.filter(d => d.type === 'local')
  )

  async function loadDrivers(): Promise<void> {
    if (initialized) return
    loading.value = true
    try {
      const store = await load('settings.json')
      const saved = await store.get<DataDriver[]>('dataDrivers')
      const activeId = await store.get<string>('activeDriverId')
      
      if (saved && Array.isArray(saved)) {
        drivers.value = saved
      }

      const existingRepo = await store.get<string>('repo')
      if (existingRepo && !drivers.value.some(d => d.path === existingRepo)) {
        const githubDriver: DataDriver = {
          id: `github-${Date.now()}`,
          type: 'github',
          name: existingRepo.split('/')[1] || existingRepo,
          path: existingRepo,
          isActive: true,
          createdAt: Date.now()
        }
        drivers.value.push(githubDriver)
        activeDriverId.value = githubDriver.id
        await saveDrivers()
      } else if (activeId) {
        activeDriverId.value = activeId
      } else if (drivers.value.length > 0) {
        activeDriverId.value = drivers.value[0].id
      }
      
      initialized = true
    } catch (e) {
      console.error('Failed to load drivers:', e)
    } finally {
      loading.value = false
    }
  }

  async function saveDrivers(): Promise<void> {
    try {
      const store = await load('settings.json')
      await store.set('dataDrivers', drivers.value)
      await store.set('activeDriverId', activeDriverId.value)
      await store.save()
    } catch (e) {
      console.error('Failed to save drivers:', e)
    }
  }

  async function addGitHubDriver(repo: string, name?: string): Promise<DataDriver> {
    const driver: DataDriver = {
      id: `github-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`,
      type: 'github',
      name: name || repo.split('/')[1] || repo,
      path: repo,
      isActive: false,
      createdAt: Date.now()
    }
    
    drivers.value.push(driver)
    await saveDrivers()
    return driver
  }

  async function addLocalDriver(folderPath: string, name?: string): Promise<DataDriver> {
    const folderName = folderPath.split('/').pop() || folderPath
    const driver: DataDriver = {
      id: `local-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`,
      type: 'local',
      name: name || folderName,
      path: folderPath,
      isActive: false,
      createdAt: Date.now()
    }
    
    drivers.value.push(driver)
    await saveDrivers()
    return driver
  }

  async function removeDriver(driverId: string): Promise<void> {
    const index = drivers.value.findIndex(d => d.id === driverId)
    if (index !== -1) {
      drivers.value.splice(index, 1)

      if (activeDriverId.value === driverId) {
        activeDriverId.value = drivers.value[0]?.id || null
      }
      
      await saveDrivers()
    }
  }

  async function setActiveDriver(driverId: string): Promise<void> {
    const driver = drivers.value.find(d => d.id === driverId)
    if (driver) {
      
      drivers.value.forEach(d => d.isActive = d.id === driverId)
      activeDriverId.value = driverId

      if (driver.type === 'github') {
        const store = await load('settings.json')
        await store.set('repo', driver.path)
        await store.save()
      }
      
      await saveDrivers()
    }
  }

  async function updateDriver(driverId: string, updates: Partial<DataDriver>): Promise<void> {
    const driver = drivers.value.find(d => d.id === driverId)
    if (driver) {
      Object.assign(driver, updates)
      await saveDrivers()
    }
  }

  async function updateDriverStats(driverId: string, photoCount: number): Promise<void> {
    await updateDriver(driverId, { 
      photoCount, 
      lastSyncAt: Date.now() 
    })
  }

  function getDriverIcon(type: DriverType): string {
    return type === 'github' 
      ? '<svg viewBox="0 0 24 24" fill="currentColor"><path d="M12 0c-6.626 0-12 5.373-12 12 0 5.302 3.438 9.8 8.207 11.387.599.111.793-.261.793-.577v-2.234c-3.338.726-4.033-1.416-4.033-1.416-.546-1.387-1.333-1.756-1.333-1.756-1.089-.745.083-.729.083-.729 1.205.084 1.839 1.237 1.839 1.237 1.07 1.834 2.807 1.304 3.492.997.107-.775.418-1.305.762-1.604-2.665-.305-5.467-1.334-5.467-5.931 0-1.311.469-2.381 1.236-3.221-.124-.303-.535-1.524.117-3.176 0 0 1.008-.322 3.301 1.23.957-.266 1.983-.399 3.003-.404 1.02.005 2.047.138 3.006.404 2.291-1.552 3.297-1.23 3.297-1.23.653 1.653.242 2.874.118 3.176.77.84 1.235 1.911 1.235 3.221 0 4.609-2.807 5.624-5.479 5.921.43.372.823 1.102.823 2.222v3.293c0 .319.192.694.801.576 4.765-1.589 8.199-6.086 8.199-11.386 0-6.627-5.373-12-12-12z"/></svg>'
      : '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/></svg>'
  }

  return {
    drivers,
    activeDriverId,
    activeDriver,
    githubDrivers,
    localDrivers,
    loading,
    loadDrivers,
    addGitHubDriver,
    addLocalDriver,
    removeDriver,
    setActiveDriver,
    updateDriver,
    updateDriverStats,
    getDriverIcon
  }
}