import React, {useState} from 'react';
import {
  View,
  Text,
  ScrollView,
  StyleSheet,
  Switch,
  TouchableOpacity,
} from 'react-native';
import {useAppStore} from '@/store/appStore';
import {useTheme} from '@/theme';
import {Card} from '@/components/Card';
import {Button} from '@/components/Button';

export function FirstRunScreen({onComplete}: {onComplete: () => void}) {
  const theme = useTheme();
  const {setSettings} = useAppStore();
  const [eulaAccepted, setEulaAccepted] = useState(false);
  const [privacyAccepted, setPrivacyAccepted] = useState(false);
  const [telemetryEnabled, setTelemetryEnabled] = useState(false);

  const handleAccept = () => {
    if (!eulaAccepted || !privacyAccepted) {
      return;
    }

    setSettings({
      eulaAccepted: true,
      privacyAccepted: true,
      telemetryEnabled,
    });

    onComplete();
  };

  return (
    <ScrollView
      style={[styles.container, {backgroundColor: theme.colors.background}]}
      contentContainerStyle={styles.content}>
      <Card>
        <Text style={[styles.title, {color: theme.colors.text}]}>
          Welcome to CrypRQ Mobile
        </Text>
        <Text style={[styles.subtitle, {color: theme.colors.textSecondary}]}>
          Please review and accept the terms to continue
        </Text>
      </Card>

      <Card>
        <View style={styles.checkboxRow}>
          <Switch
            testID="eula-switch"
            value={eulaAccepted}
            onValueChange={setEulaAccepted}
          />
          <View style={styles.checkboxLabel}>
            <Text style={[styles.checkboxText, {color: theme.colors.text}]}>
              I accept the End User License Agreement
            </Text>
            <TouchableOpacity onPress={() => {/* Open EULA */}}>
              <Text style={[styles.link, {color: theme.colors.primary}]}>
                View EULA
              </Text>
            </TouchableOpacity>
          </View>
        </View>
      </Card>

      <Card>
        <View style={styles.checkboxRow}>
          <Switch
            testID="privacy-switch"
            value={privacyAccepted}
            onValueChange={setPrivacyAccepted}
          />
          <View style={styles.checkboxLabel}>
            <Text style={[styles.checkboxText, {color: theme.colors.text}]}>
              I accept the Privacy Policy
            </Text>
            <TouchableOpacity onPress={() => {/* Open Privacy */}}>
              <Text style={[styles.link, {color: theme.colors.primary}]}>
                View Privacy Policy
              </Text>
            </TouchableOpacity>
          </View>
        </View>
      </Card>

      <Card>
        <View style={styles.checkboxRow}>
          <Switch
            testID="telemetry-switch"
            value={telemetryEnabled}
            onValueChange={setTelemetryEnabled}
          />
          <View style={styles.checkboxLabel}>
            <Text style={[styles.checkboxText, {color: theme.colors.text}]}>
              Enable anonymous telemetry (optional)
            </Text>
            <Text style={[styles.description, {color: theme.colors.textSecondary}]}>
              Help improve CrypRQ by sharing anonymous usage data. You can change
              this later in Settings.
            </Text>
          </View>
        </View>
      </Card>

      <View style={styles.actions}>
        <Button
          testID="accept-button"
          title="Accept and Continue"
          onPress={handleAccept}
          variant="primary"
          disabled={!eulaAccepted || !privacyAccepted}
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
    fontWeight: '700',
    marginBottom: 8,
  },
  subtitle: {
    fontSize: 14,
  },
  checkboxRow: {
    flexDirection: 'row',
    alignItems: 'flex-start',
    gap: 12,
  },
  checkboxLabel: {
    flex: 1,
  },
  checkboxText: {
    fontSize: 16,
    fontWeight: '600',
    marginBottom: 4,
  },
  link: {
    fontSize: 14,
    textDecorationLine: 'underline',
  },
  description: {
    fontSize: 12,
    marginTop: 4,
    lineHeight: 16,
  },
  actions: {
    marginTop: 24,
  },
});

