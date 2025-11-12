import React, {useEffect, useState} from 'react';
import {NavigationContainer} from '@react-navigation/native';
import {createBottomTabNavigator} from '@react-navigation/bottom-tabs';
import {createNativeStackNavigator} from '@react-navigation/native-stack';
import {StatusBar, useColorScheme} from 'react-native';
import {GestureHandlerRootView} from 'react-native-gesture-handler';
import {DashboardScreen} from '@/screens/DashboardScreen';
import {PeersScreen} from '@/screens/PeersScreen';
import {SettingsScreen} from '@/screens/SettingsScreen';
import {LogsModal} from '@/screens/LogsModal';
import {PrivacyScreen} from '@/screens/PrivacyScreen';
import {AboutScreen} from '@/screens/AboutScreen';
import {FirstRunScreen} from '@/screens/FirstRunScreen';
import {ReportIssueScreen} from '@/screens/Help/ReportIssue';
import {useAppStore} from '@/store/appStore';
import {backendService} from '@/services/backend';
import {backgroundService} from '@/services/background';
import {notificationService} from '@/services/notifications';
import {useTheme} from '@/theme';

const Tab = createBottomTabNavigator();
const Stack = createNativeStackNavigator();

function MainTabs() {
  const theme = useTheme();

  return (
    <Tab.Navigator
      screenOptions={{
        headerStyle: {
          backgroundColor: theme.colors.surface,
        },
        headerTintColor: theme.colors.text,
        tabBarStyle: {
          backgroundColor: theme.colors.surface,
          borderTopColor: theme.colors.border,
        },
        tabBarActiveTintColor: theme.colors.primary,
        tabBarInactiveTintColor: theme.colors.textSecondary,
      }}>
      <Tab.Screen
        name="Dashboard"
        component={DashboardScreen}
        options={{
          title: 'Dashboard',
          tabBarTestID: 'tab-dashboard',
        }}
      />
      <Tab.Screen
        name="Peers"
        component={PeersScreen}
        options={{
          title: 'Peers',
          tabBarTestID: 'tab-peers',
        }}
      />
      <Tab.Screen
        name="Settings"
        component={SettingsScreen}
        options={{
          title: 'Settings',
          tabBarTestID: 'tab-settings',
        }}
      />
    </Tab.Navigator>
  );
}

export default function App() {
  const colorScheme = useColorScheme();
  const {settings, setConnectionStatus} = useAppStore();
  const [firstRunComplete, setFirstRunComplete] = useState(
    settings.eulaAccepted && settings.privacyAccepted,
  );
  const isDark = settings.theme === 'dark' || (settings.theme === 'system' && colorScheme === 'dark');

  useEffect(() => {
    // Initialize services
    notificationService.initialize();
    backgroundService.initialize();
    
    // Initialize crash reporting if enabled
    if (settings.crashReportingEnabled) {
      import('@/services/crashReporting').then(m => {
        m.crashReportingService.initialize();
      });
    }

    // Start metrics polling
    backendService.startPolling();

    // Listen to backend events
    const handleMetrics = async (metrics: any) => {
      // Metrics updates handled in backendService.startPolling
      const {connectionStatus: currentStatus} = useAppStore.getState();
      
      // Notify on connection changes
      if (metrics.peerId && (!currentStatus.peerId || currentStatus.status === 'disconnected')) {
        await notificationService.notifyConnected(metrics.peerId);
      }
    };

    const handleRotation = async () => {
      setConnectionStatus('rotating');
      await notificationService.notifyRotation();
      setTimeout(() => {
        setConnectionStatus('connected');
      }, 1000);
    };

    const handleError = (error: any) => {
      console.error('Backend error:', error);
      setConnectionStatus('errored', {});
    };

    backendService.on('metrics', handleMetrics);
    backendService.on('rotation', handleRotation);
    backendService.on('error', handleError);

    return () => {
      backendService.off('metrics', handleMetrics);
      backendService.off('rotation', handleRotation);
      backendService.off('error', handleError);
      backendService.stopPolling();
      backgroundService.stop();
    };
  }, [setConnectionStatus]);

  // Show first-run screen if not accepted
  if (!firstRunComplete) {
    return (
      <GestureHandlerRootView style={{flex: 1}}>
        <StatusBar
          barStyle={isDark ? 'light-content' : 'dark-content'}
          backgroundColor={isDark ? '#121212' : '#FFFFFF'}
        />
        <FirstRunScreen onComplete={() => setFirstRunComplete(true)} />
      </GestureHandlerRootView>
    );
  }

  return (
    <GestureHandlerRootView style={{flex: 1}}>
      <NavigationContainer>
        <StatusBar
          barStyle={isDark ? 'light-content' : 'dark-content'}
          backgroundColor={isDark ? '#121212' : '#FFFFFF'}
        />
        <Stack.Navigator>
          <Stack.Screen
            name="Main"
            component={MainTabs}
            options={{headerShown: false}}
          />
          <Stack.Screen
            name="Logs"
            component={LogsModal}
            options={{
              presentation: 'modal',
              title: 'Logs',
            }}
          />
          <Stack.Screen
            name="Privacy"
            component={PrivacyScreen}
            options={{
              title: 'Privacy',
            }}
          />
          <Stack.Screen
            name="About"
            component={AboutScreen}
            options={{
              title: 'About',
            }}
          />
          <Stack.Screen
            name="ReportIssue"
            component={ReportIssueScreen}
            options={{
              title: 'Report Issue',
            }}
          />
        </Stack.Navigator>
      </NavigationContainer>
    </GestureHandlerRootView>
  );
}

