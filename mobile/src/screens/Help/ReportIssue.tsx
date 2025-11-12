import React, {useState} from 'react';
import {
  View,
  Text,
  StyleSheet,
  ScrollView,
  TouchableOpacity,
  Alert,
  Platform,
  Share,
} from 'react-native';
import {useAppStore} from '@/store/appStore';
import {useTheme} from '@/theme';
import {Card} from '@/components/Card';
import {Button} from '@/components/Button';
import {redactSecrets} from '@/services/security';

export function ReportIssueScreen() {
  const theme = useTheme();
  const {logs, settings, connectionStatus} = useAppStore();
  const [isGenerating, setIsGenerating] = useState(false);

  const generateDiagnostics = async () => {
    setIsGenerating(true);
    try {
      // Collect diagnostics data
      const diagnostics = {
        timestamp: new Date().toISOString(),
        appVersion: '1.0.0', // TODO: Get from package.json or build config
        buildNumber: '1', // TODO: Get from build config
        platform: Platform.OS,
        platformVersion: Platform.Version.toString(),
        deviceInfo: {
          // Using Platform API since DeviceInfo may not be installed
          platform: Platform.OS,
          version: Platform.Version.toString(),
        },
        settings: {
          profile: settings.profile,
          endpoint: settings.profile === 'REMOTE' ? '[REDACTED]' : settings.endpoint,
          rotationInterval: settings.rotationInterval,
          logLevel: settings.logLevel,
          theme: settings.theme,
          notifications: settings.notifications,
        },
        connectionStatus: {
          status: connectionStatus.status,
          peerId: connectionStatus.peerId ? '[REDACTED]' : undefined,
          latency: connectionStatus.latency,
        },
        logs: logs.slice(-200).map(log => ({
          ts: log.ts,
          level: log.level,
          source: log.source,
          msg: redactSecrets(log.msg),
          meta: log.meta ? Object.fromEntries(
            Object.entries(log.meta).map(([k, v]) => [
              k,
              typeof v === 'string' ? redactSecrets(v) : v,
            ])
          ) : undefined,
        })),
      };

      // Convert to JSON string
      const jsonString = JSON.stringify(diagnostics, null, 2);

      // Share via native share sheet
      const result = await Share.share({
        message: `CrypRQ Diagnostics Report\n\n${jsonString}`,
        title: 'CrypRQ Diagnostics',
      });

      if (result.action === Share.sharedAction) {
        // Show toast feedback
        Alert.alert('Report Prepared', 'Diagnostics report has been prepared and is ready to share.', [
          { text: 'OK', style: 'default' },
        ]);
      } else if (result.action === Share.dismissedAction) {
        // User dismissed share sheet - no action needed
      }
    } catch (error: any) {
      Alert.alert('Error', `Failed to generate diagnostics: ${error.message}`);
    } finally {
      setIsGenerating(false);
    }
  };

  return (
    <ScrollView
      style={[styles.container, {backgroundColor: theme.colors.background}]}
      contentContainerStyle={styles.content}>
      <Card>
        <Text style={[styles.title, {color: theme.colors.text}]}>
          Report an Issue
        </Text>
        <Text style={[styles.description, {color: theme.colors.textSecondary}]}>
          If you're experiencing problems, you can generate a diagnostics report
          to help us troubleshoot. The report includes:
        </Text>
        <View style={styles.list}>
          <Text style={[styles.listItem, {color: theme.colors.textSecondary}]}>
            • App version and device information
          </Text>
          <Text style={[styles.listItem, {color: theme.colors.textSecondary}]}>
            • Connection status and settings (sensitive data redacted)
          </Text>
          <Text style={[styles.listItem, {color: theme.colors.textSecondary}]}>
            • Recent logs (last 200 entries, secrets redacted)
          </Text>
        </View>
        <View style={[styles.warningBox, {backgroundColor: theme.colors.warning + '20'}]}>
          <Text style={[styles.warningText, {color: theme.colors.warning}]}>
            ⚠️ All sensitive information (endpoints, tokens, peer IDs) has been
            automatically redacted from the report.
          </Text>
        </View>
      </Card>

      <View style={styles.actions}>
        <Button
          testID="generate-diagnostics-button"
          title={isGenerating ? 'Generating...' : 'Generate & Share Diagnostics'}
          onPress={generateDiagnostics}
          variant="primary"
          disabled={isGenerating}
        />
      </View>
    </ScrollView>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
  },
  content: {
    padding: 16,
  },
  title: {
    fontSize: 24,
    fontWeight: '600',
    marginBottom: 12,
  },
  description: {
    fontSize: 16,
    lineHeight: 24,
    marginBottom: 16,
  },
  list: {
    marginLeft: 8,
    marginBottom: 16,
  },
  listItem: {
    fontSize: 14,
    lineHeight: 20,
    marginBottom: 8,
  },
  warningBox: {
    padding: 12,
    borderRadius: 8,
    marginTop: 8,
  },
  warningText: {
    fontSize: 12,
    lineHeight: 16,
  },
  actions: {
    marginTop: 24,
  },
});

