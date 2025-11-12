import BackgroundFetch from 'react-native-background-fetch';
import {AppState, AppStateStatus} from 'react-native';
import {backendService} from './backend';
import {notificationService} from './notifications';
import {useAppStore} from '@/store/appStore';

class BackgroundService {
  private appState: AppStateStatus = 'active';
  private backgroundTaskId: number | null = null;

  initialize() {
    // Configure background fetch
    BackgroundFetch.configure(
      {
        minimumFetchInterval: 15, // 15 minutes minimum
        stopOnTerminate: false,
        startOnBoot: true,
        enableHeadless: true,
      },
      async (taskId: string) => {
        console.log('[BackgroundFetch] Task:', taskId);
        await this.performBackgroundUpdate();
        BackgroundFetch.finish(taskId);
      },
      (error: Error) => {
        console.error('[BackgroundFetch] Failed to start:', error);
      },
    );

    // Listen to app state changes
    AppState.addEventListener('change', this.handleAppStateChange);

    // Start background fetch
    BackgroundFetch.start();
  }

  private handleAppStateChange = (nextAppState: AppStateStatus) => {
    if (this.appState.match(/inactive|background/) && nextAppState === 'active') {
      // App came to foreground, resume polling
      backendService.startPolling();
    } else if (this.appState === 'active' && nextAppState.match(/inactive|background/)) {
      // App went to background, stop polling (rely on background fetch)
      backendService.stopPolling();
    }
    this.appState = nextAppState;
  };

  private async performBackgroundUpdate() {
    try {
      const metrics = await backendService.getMetrics();
      if (metrics) {
        const {updateMetrics, connectionStatus} = useAppStore.getState();
        
        // Update metrics
        updateMetrics({
          latency: metrics.latency,
          rotationTimer: metrics.rotationTimer,
          bytesIn: metrics.bytesIn,
          bytesOut: metrics.bytesOut,
          peerId: metrics.peerId,
        });

        // Check for rotation
        if (metrics.rotationTimer === 0 && connectionStatus.status === 'connected') {
          await notificationService.notifyRotation();
        }

        // Check connection status changes
        if (metrics.peerId && connectionStatus.status === 'disconnected') {
          await notificationService.notifyConnected(metrics.peerId);
        } else if (!metrics.peerId && connectionStatus.status === 'connected') {
          await notificationService.notifyDisconnected();
        }
      }
    } catch (error) {
      console.error('[BackgroundService] Update failed:', error);
    }
  }

  stop() {
    AppState.removeEventListener('change', this.handleAppStateChange);
    BackgroundFetch.stop();
  }
}

export const backgroundService = new BackgroundService();

