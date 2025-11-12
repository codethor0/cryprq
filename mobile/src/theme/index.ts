import {useColorScheme} from 'react-native';

export const lightTheme = {
  colors: {
    primary: '#1976D2',
    secondary: '#424242',
    background: '#FFFFFF',
    surface: '#F5F5F5',
    text: '#212121',
    textSecondary: '#757575',
    border: '#E0E0E0',
    error: '#D32F2F',
    warning: '#F57C00',
    success: '#388E3C',
    info: '#1976D2',
    statusConnected: '#4CAF50',
    statusConnecting: '#FFC107',
    statusDisconnected: '#F44336',
    statusRotating: '#FF9800',
  },
  spacing: {
    xs: 4,
    sm: 8,
    md: 16,
    lg: 24,
    xl: 32,
  },
  borderRadius: {
    sm: 4,
    md: 8,
    lg: 12,
    xl: 16,
  },
  typography: {
    h1: {fontSize: 32, fontWeight: '700' as const},
    h2: {fontSize: 24, fontWeight: '600' as const},
    h3: {fontSize: 20, fontWeight: '600' as const},
    body: {fontSize: 16, fontWeight: '400' as const},
    caption: {fontSize: 14, fontWeight: '400' as const},
    button: {fontSize: 16, fontWeight: '600' as const},
  },
};

export const darkTheme = {
  ...lightTheme,
  colors: {
    ...lightTheme.colors,
    background: '#121212',
    surface: '#1E1E1E',
    text: '#FFFFFF',
    textSecondary: '#B0B0B0',
    border: '#333333',
  },
};

export type Theme = typeof lightTheme;

export function useTheme(): Theme {
  const colorScheme = useColorScheme();
  return colorScheme === 'dark' ? darkTheme : lightTheme;
}

