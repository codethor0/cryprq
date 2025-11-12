import { ipcMain } from 'electron'
import { loadSettings, saveSettings } from './settings'

ipcMain.handle('settings:load', async () => {
  return loadSettings()
})

ipcMain.handle('settings:save', async (_e, settings: any) => {
  saveSettings(settings)
  return { ok: true }
})

