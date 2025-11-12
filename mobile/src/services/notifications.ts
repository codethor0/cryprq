import notifee, {AndroidImportance} from '@notifee/react-native';
import {useAppStore} from '@/store/appStore';

class NotificationService {
  private initialized = false;

  async initialize() {
    if (this.initialized) return;

    // Request permissions
    await notifee.requestPermission();

    // Create Android channel
    await notifee.createChannel({
      id: 'cryprq-status',
      name: 'CrypRQ Status',
      importance: AndroidImportance.HIGH,
    });

    this.initialized = true;
  }

  async notifyConnected(peerId?: string) {
    const {settings} = useAppStore.getState();
    if (!settings.notifications.connectDisconnect) return;

    await this.initialize();

    await notifee.displayNotification({
      title: 'CrypRQ Connected',
      body: peerId ? `Connected to ${peerId.slice(0, 16)}...` : 'Connection established',
      android: {
        channelId: 'cryprq-status',
        importance: AndroidImportance.HIGH,
        pressAction: {
          id: 'default',
        },
      },
    });
  }

  async notifyDisconnected() {
    const {settings} = useAppStore.getState();
    if (!settings.notifications.connectDisconnect) return;

    await this.initialize();

    await notifee.displayNotification({
      title: 'CrypRQ Disconnected',
      body: 'Connection terminated',
      android: {
        channelId: 'cryprq-status',
        importance: AndroidImportance.HIGH,
        pressAction: {
          id: 'default',
        },
      },
    });
  }

  async notifyRotation() {
    const {settings} = useAppStore.getState();
    if (!settings.notifications.rotations) return;

    await this.initialize();

    const now = new Date();
    const timeStr = now.toLocaleTimeString();

    await notifee.displayNotification({
      title: 'Keys Rotated',
      body: `New keys generated at ${timeStr}`,
      android: {
        channelId: 'cryprq-status',
        importance: AndroidImportance.DEFAULT,
        pressAction: {
          id: 'default',
        },
      },
    });
  }
}

export const notificationService = new NotificationService();

