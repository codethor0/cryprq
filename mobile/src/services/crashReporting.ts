// Crash reporting service (Sentry/Bugsnag integration)
// Opt-in via Privacy settings

import {useAppStore} from '@/store/appStore';
import {redactSecrets} from './security';

class CrashReportingService {
  private initialized = false;
  private enabled = false;

  async initialize() {
    const {settings} = useAppStore.getState();
    this.enabled = settings.crashReportingEnabled || false;

    if (!this.enabled) {
      return; // No initialization if disabled
    }

    // Initialize Sentry/Bugsnag here
    // Example with Sentry:
    // import * as Sentry from '@sentry/react-native';
    // Sentry.init({
    //   dsn: 'YOUR_DSN',
    //   beforeSend: (event) => {
    //     // Redact sensitive data
    //     if (event.message) {
    //       event.message = redactSecrets(event.message);
    //     }
    //     if (event.extra) {
    //       event.extra = Object.fromEntries(
    //         Object.entries(event.extra).map(([k, v]) => [
    //           k,
    //           typeof v === 'string' ? redactSecrets(v) : v,
    //         ])
    //       );
    //     }
    //     return event;
    //   },
    // });

    this.initialized = true;
  }

  setEnabled(enabled: boolean) {
    this.enabled = enabled;
    if (enabled && !this.initialized) {
      this.initialize();
    }
  }

  addBreadcrumb(category: string, message: string, level: 'info' | 'warn' | 'error' = 'info') {
    if (!this.enabled || !this.initialized) {
      return;
    }

    // Only track connect/disconnect/rotation events
    const allowedCategories = ['connect', 'disconnect', 'rotation'];
    if (!allowedCategories.includes(category)) {
      return;
    }

    const redactedMessage = redactSecrets(message);

    // Sentry/Bugsnag breadcrumb
    // Sentry.addBreadcrumb({
    //   category,
    //   message: redactedMessage,
    //   level,
    // });
  }

  captureException(error: Error, context?: Record<string, any>) {
    if (!this.enabled || !this.initialized) {
      return;
    }

    const redactedContext = context
      ? Object.fromEntries(
          Object.entries(context).map(([k, v]) => [
            k,
            typeof v === 'string' ? redactSecrets(v) : v,
          ])
        )
      : undefined;

    // Sentry.captureException(error, {
    //   extra: redactedContext,
    // });
  }

  testCrash() {
    if (!this.enabled || !this.initialized) {
      throw new Error('Crash reporting is not enabled');
    }

    // Sentry.captureException(new Error('Test crash'));
  }
}

export const crashReportingService = new CrashReportingService();

