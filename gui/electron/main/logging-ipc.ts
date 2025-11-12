import { ipcMain } from 'electron'
import { readTail, getLogFiles } from './logging'

ipcMain.handle('logs:tail', async (_e, { lines = 1000 }: { lines?: number }) => {
  return readTail({ lines })
})

ipcMain.handle('logs:list', async () => {
  return getLogFiles()
})

