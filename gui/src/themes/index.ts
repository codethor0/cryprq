export const lightTheme = {
  background: '#FFFFFF',
  surface: '#F5F5F5',
  primary: '#1DE9B6',
  primaryDark: '#00BFA5',
  text: '#212121',
  textSecondary: '#757575',
  border: '#E0E0E0',
  error: '#F44336',
  warning: '#FF9800',
  success: '#4CAF50',
  sidebar: '#FAFAFA',
  sidebarHover: '#F0F0F0',
}

export const darkTheme = {
  background: '#121212',
  surface: '#1E1E1E',
  primary: '#1DE9B6',
  primaryDark: '#00BFA5',
  text: '#E0E0E0',
  textSecondary: '#B0B0B0',
  border: '#333333',
  error: '#EF5350',
  warning: '#FFB74D',
  success: '#66BB6A',
  sidebar: '#1A1A1A',
  sidebarHover: '#252525',
}

export type Theme = typeof lightTheme

export const getTheme = (mode: 'light' | 'dark'): Theme => {
  return mode === 'dark' ? darkTheme : lightTheme
}

