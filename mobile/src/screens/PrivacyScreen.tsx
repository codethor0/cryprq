import React from 'react';
import {View, Text, ScrollView, StyleSheet, Linking, Switch} from 'react-native';
import {useAppStore} from '@/store/appStore';
import {useTheme} from '@/theme';
import {Card} from '@/components/Card';
import {Button} from '@/components/Button';

const PRIVACY_POLICY_URL = 'https://github.com/yourusername/CrypRQ/blob/main/docs/privacy.md';

export function PrivacyScreen() {
  const theme = useTheme();
  const {settings, setSettings} = useAppStore();

  const handleOpenPrivacyPolicy = async () => {
    const supported = await Linking.canOpenURL(PRIVACY_POLICY_URL);
    if (supported) {
      await Linking.openURL(PRIVACY_POLICY_URL);
    } else {
      console.error('Cannot open privacy policy URL');
    }
  };

  return (
    <ScrollView
      style={[styles.container, {backgroundColor: theme.colors.background}]}
      contentContainerStyle={styles.content}>
      <Card>
        <Text style={[styles.sectionTitle, {color: theme.colors.text}]}>
          Privacy Policy
        </Text>
        <Text style={[styles.bodyText, {color: theme.colors.textSecondary}]}>
          CrypRQ is designed with privacy as a core principle. We do not collect,
          store, or transmit any personal data or usage information unless explicitly
          enabled below.
        </Text>
        <Button
          testID="open-privacy-policy-button"
          title="View Full Privacy Policy"
          onPress={handleOpenPrivacyPolicy}
          variant="secondary"
          style={styles.button}
        />
      </Card>

      <Card>
        <Text style={[styles.sectionTitle, {color: theme.colors.text}]}>
          Data Collection
        </Text>
        <View style={styles.switchRow}>
          <View style={styles.switchLabelContainer}>
            <Text style={[styles.switchLabel, {color: theme.colors.text}]}>
              Enable Telemetry (Anonymous)
            </Text>
            <Text style={[styles.switchDescription, {color: theme.colors.textSecondary}]}>
              When enabled, collects anonymous usage data: install ID, OS version,
              app version, and non-PII events (connect/disconnect counts, error types).
              No IP addresses, peer IDs, or network data is collected.
            </Text>
          </View>
          <Switch
            testID="telemetry-switch"
            value={settings.telemetryEnabled || false}
            onValueChange={value => setSettings({telemetryEnabled: value})}
          />
        </View>
      </Card>

      <Card>
        <Text style={[styles.sectionTitle, {color: theme.colors.text}]}>
          Crash Reporting
        </Text>
        <View style={styles.switchRow}>
          <View style={styles.switchLabelContainer}>
            <Text style={[styles.switchLabel, {color: theme.colors.text}]}>
              Enable Crash Reporting
            </Text>
            <Text style={[styles.switchDescription, {color: theme.colors.textSecondary}]}>
              When enabled, crash reports are sent to help improve CrypRQ. Endpoints,
              tokens, and sensitive data are automatically redacted.
            </Text>
          </View>
          <Switch
            testID="crash-reporting-switch"
            value={settings.crashReportingEnabled || false}
            onValueChange={value => {
              setSettings({crashReportingEnabled: value});
              // Initialize crash reporting service
              if (value) {
                import('@/services/crashReporting').then(m => {
                  m.crashReportingService.setEnabled(true);
                });
              }
            }}
          />
        </View>
      </Card>

      <Card>
        <Text style={[styles.sectionTitle, {color: theme.colors.text}]}>
          What We Collect (if telemetry enabled)
        </Text>
        <View style={styles.list}>
          <Text style={[styles.listItem, {color: theme.colors.textSecondary}]}>
            • Anonymous install ID (random UUID, not tied to you)
          </Text>
          <Text style={[styles.listItem, {color: theme.colors.textSecondary}]}>
            • Operating system and version
          </Text>
          <Text style={[styles.listItem, {color: theme.colors.textSecondary}]}>
            • App version and build number
          </Text>
          <Text style={[styles.listItem, {color: theme.colors.textSecondary}]}>
            • Event counts (connections, disconnections, errors)
          </Text>
          <Text style={[styles.listItem, {color: theme.colors.textSecondary}]}>
            • Error types (not messages or logs)
          </Text>
        </View>
        <Text style={[styles.note, {color: theme.colors.textSecondary}]}>
          Note: Telemetry is OFF by default. You must explicitly enable it.
        </Text>
      </Card>
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
  sectionTitle: {
    fontSize: 18,
    fontWeight: '600',
    marginBottom: 12,
  },
  bodyText: {
    fontSize: 14,
    lineHeight: 20,
    marginBottom: 16,
  },
  button: {
    marginTop: 8,
  },
  switchRow: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'flex-start',
    marginTop: 12,
  },
  switchLabelContainer: {
    flex: 1,
    marginRight: 12,
  },
  switchLabel: {
    fontSize: 16,
    fontWeight: '600',
    marginBottom: 4,
  },
  switchDescription: {
    fontSize: 12,
    lineHeight: 16,
  },
  list: {
    marginTop: 8,
  },
  listItem: {
    fontSize: 14,
    lineHeight: 20,
    marginBottom: 8,
  },
  note: {
    fontSize: 12,
    fontStyle: 'italic',
    marginTop: 12,
  },
});

