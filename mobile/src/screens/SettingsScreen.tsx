import React, {useState, useEffect} from 'react';
import {
  View,
  Text,
  ScrollView,
  StyleSheet,
  TextInput,
  Switch,
  TouchableOpacity,
  Alert,
} from 'react-native';
import {Picker} from '@react-native-picker/picker';
import {useAppStore} from '@/store/appStore';
import {useTheme} from '@/theme';
import {Card} from '@/components/Card';
import {Button} from '@/components/Button';
import {isValidEndpoint} from '@/utils/validation';
import {backendService} from '@/services/backend';
import {DeveloperScreen} from './DeveloperScreen';
import {useNavigation} from '@react-navigation/native';
import type {ProfileType} from '@/types';

export function SettingsScreen() {
  const theme = useTheme();
  const navigation = useNavigation();
  const {settings, setSettings} = useAppStore();
  const [localProfile, setLocalProfile] = useState<ProfileType>(settings.profile);
  const [localEndpoint, setLocalEndpoint] = useState(settings.endpoint || '');
  const [localRotationInterval, setLocalRotationInterval] = useState(
    settings.rotationInterval.toString(),
  );
  const [localNotifications, setLocalNotifications] = useState(settings.notifications);
  const [localPostQuantum, setLocalPostQuantum] = useState(settings.postQuantumEnabled !== false);
  const [versionTapCount, setVersionTapCount] = useState(0);
  const [showDeveloper, setShowDeveloper] = useState(false);

  const handleSave = () => {
    // Validate endpoint if LAN/REMOTE
    if (localProfile !== 'LOCAL' && localEndpoint) {
      if (!isValidEndpoint(localEndpoint)) {
        Alert.alert('Invalid Endpoint', 'Please enter a valid HTTP/HTTPS URL');
        return;
      }
    }

    const rotationInterval = parseInt(localRotationInterval, 10);
    if (isNaN(rotationInterval) || rotationInterval < 1) {
      Alert.alert('Invalid Rotation Interval', 'Must be at least 1 minute');
      return;
    }

    setSettings({
      profile: localProfile,
      endpoint: localProfile !== 'LOCAL' ? localEndpoint : undefined,
      rotationInterval,
      notifications: localNotifications,
      postQuantumEnabled: localPostQuantum,
    });

    backendService.setProfile(localProfile, localEndpoint);

    Alert.alert('Success', 'Settings saved');
  };

  return (
    <ScrollView
      style={[styles.container, {backgroundColor: theme.colors.background}]}
      contentContainerStyle={styles.content}>
      <Card testID="profile-card">
        <Text style={[styles.sectionTitle, {color: theme.colors.text}]}>
          Connection Profile
        </Text>
        <View
          style={[
            styles.pickerContainer,
            {backgroundColor: theme.colors.surface, borderColor: theme.colors.border},
          ]}>
          <Picker
            testID="profile-picker"
            selectedValue={localProfile}
            onValueChange={(value: ProfileType) => {
              setLocalProfile(value);
              if (value === 'LOCAL') {
                setLocalEndpoint('http://127.0.0.1:9464');
              }
            }}
            style={{color: theme.colors.text}}>
            <Picker.Item label="Local" value="LOCAL" />
            <Picker.Item label="LAN" value="LAN" />
            <Picker.Item label="Remote" value="REMOTE" />
          </Picker>
        </View>
      </Card>

      {localProfile !== 'LOCAL' && (
        <Card testID="endpoint-card">
          <Text style={[styles.label, {color: theme.colors.text}]}>
            Endpoint URL
          </Text>
          <TextInput
            testID="endpoint-input"
            style={[
              styles.input,
              {
                backgroundColor: theme.colors.surface,
                color: theme.colors.text,
                borderColor: theme.colors.border,
              },
            ]}
            placeholder="http://192.168.1.100:9464"
            placeholderTextColor={theme.colors.textSecondary}
            value={localEndpoint}
            onChangeText={setLocalEndpoint}
            autoCapitalize="none"
            autoCorrect={false}
          />
        </Card>
      )}

      <Card testID="rotation-card">
        <Text style={[styles.label, {color: theme.colors.text}]}>
          Rotation Interval (minutes)
        </Text>
        <TextInput
          testID="rotation-input"
          style={[
            styles.input,
            {
              backgroundColor: theme.colors.surface,
              color: theme.colors.text,
              borderColor: theme.colors.border,
            },
          ]}
          placeholder="5"
          placeholderTextColor={theme.colors.textSecondary}
          value={localRotationInterval}
          onChangeText={setLocalRotationInterval}
          keyboardType="numeric"
        />
      </Card>

      <Card testID="notifications-card">
        <Text style={[styles.sectionTitle, {color: theme.colors.text}]}>
          Notifications
        </Text>
        <View style={styles.switchRow}>
          <Text style={[styles.switchLabel, {color: theme.colors.text}]}>
            Connect/Disconnect
          </Text>
          <Switch
            testID="notify-connect-switch"
            value={localNotifications.connectDisconnect}
            onValueChange={value =>
              setLocalNotifications({...localNotifications, connectDisconnect: value})
            }
          />
        </View>
        <View style={styles.switchRow}>
          <Text style={[styles.switchLabel, {color: theme.colors.text}]}>
            Key Rotations
          </Text>
          <Switch
            testID="notify-rotation-switch"
            value={localNotifications.rotations}
            onValueChange={value =>
              setLocalNotifications({...localNotifications, rotations: value})
            }
          />
        </View>
      </Card>

      <Card testID="security-card">
        <Text style={[styles.sectionTitle, {color: theme.colors.text}]}>
          Security
        </Text>
        <View style={styles.switchRow}>
          <View style={{flex: 1}}>
            <Text style={[styles.switchLabel, {color: theme.colors.text}]}>
              Post-Quantum Encryption
            </Text>
            <Text style={[styles.switchDescription, {color: theme.colors.textSecondary}]}>
              {localPostQuantum
                ? '✅ ML-KEM (Kyber768) + X25519 hybrid enabled'
                : '⚠️ X25519-only (not recommended)'}
            </Text>
          </View>
          <Switch
            testID="post-quantum-switch"
            value={localPostQuantum}
            onValueChange={value => {
              setLocalPostQuantum(value);
              if (!value) {
                Alert.alert(
                  'Post-Quantum Encryption Disabled',
                  'You are using X25519-only encryption. This is not recommended for future-proof security.',
                  [{text: 'OK'}]
                );
              }
            }}
          />
        </View>
      </Card>

      <Card testID="mode-card">
        <Text style={[styles.sectionTitle, {color: theme.colors.text}]}>
          Mode
        </Text>
        <View style={styles.pickerContainer}>
          <Picker
            testID="mode-picker"
            selectedValue={settings.mode || 'controller'}
            onValueChange={(value: 'controller' | 'on-device') => {
              if (value === 'on-device') {
                Alert.alert(
                  'On-Device Mode (Beta)',
                  'On-Device mode is not yet available. This feature will allow running CrypRQ core directly on your device. See mobile/docs/on-device-plan.md for details.',
                  [{text: 'OK'}]
                );
                return;
              }
              setSettings({mode: value});
            }}
            style={{color: theme.colors.text}}>
            <Picker.Item label="Controller (Default)" value="controller" />
            <Picker.Item label="On-Device (Beta - Coming Soon)" value="on-device" enabled={false} />
          </Picker>
        </View>
        {settings.mode === 'on-device' && (
          <View style={styles.warningBox}>
            <Text style={[styles.warningText, {color: theme.colors.warning}]}>
              ⚠️ On-Device mode requires VPN permissions and may impact battery life.
            </Text>
          </View>
        )}
      </Card>

      <Card testID="links-card">
        <Text style={[styles.sectionTitle, {color: theme.colors.text}]}>
          Information
        </Text>
        <View style={styles.linkGroup}>
          <TouchableOpacity
            testID="privacy-link"
            onPress={() => navigation.navigate('Privacy' as never)}
            style={styles.linkRow}>
            <Text style={[styles.linkText, {color: theme.colors.primary}]}>
              Privacy Policy
            </Text>
          </TouchableOpacity>
          <TouchableOpacity
            testID="about-link"
            onPress={() => navigation.navigate('About' as never)}
            style={styles.linkRow}>
            <Text style={[styles.linkText, {color: theme.colors.primary}]}>
              About
            </Text>
          </TouchableOpacity>
          <TouchableOpacity
            testID="report-issue-link"
            onPress={() => navigation.navigate('ReportIssue' as never)}
            style={styles.linkRow}>
            <Text style={[styles.linkText, {color: theme.colors.primary}]}>
              Report Issue
            </Text>
          </TouchableOpacity>
        </View>
      </Card>

      <View style={styles.actions}>
        <Button
          testID="save-settings-button"
          title="Save Settings"
          onPress={handleSave}
          variant="primary"
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
  sectionTitle: {
    fontSize: 18,
    fontWeight: '600',
    marginBottom: 12,
  },
  label: {
    fontSize: 16,
    fontWeight: '600',
    marginBottom: 8,
  },
  pickerContainer: {
    borderWidth: 1,
    borderRadius: 8,
    overflow: 'hidden',
  },
  input: {
    borderWidth: 1,
    borderRadius: 8,
    padding: 12,
    fontSize: 16,
    marginTop: 8,
  },
  switchRow: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginTop: 12,
  },
  switchLabel: {
    fontSize: 16,
  },
  switchDescription: {
    fontSize: 12,
    marginTop: 4,
  },
  actions: {
    marginTop: 16,
  },
  versionArea: {
    marginTop: 32,
    padding: 16,
    alignItems: 'center',
  },
  versionText: {
    fontSize: 12,
  },
  modalOverlay: {
    position: 'absolute',
    top: 0,
    left: 0,
    right: 0,
    bottom: 0,
    backgroundColor: 'rgba(0,0,0,0.7)',
    justifyContent: 'center',
    alignItems: 'center',
  },
  modalContent: {
    width: '90%',
    maxHeight: '80%',
    borderRadius: 12,
    overflow: 'hidden',
  },
  modalHeader: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    padding: 16,
    borderBottomWidth: 1,
    borderBottomColor: '#333',
  },
  modalTitle: {
    fontSize: 20,
    fontWeight: '600',
  },
  closeButton: {
    fontSize: 24,
    fontWeight: '600',
  },
  modalScroll: {
    maxHeight: 500,
  },
  linkGroup: {
    marginTop: 8,
  },
  linkRow: {
    paddingVertical: 12,
    borderBottomWidth: 1,
    borderBottomColor: '#333',
  },
  linkText: {
    fontSize: 16,
  },
  warningBox: {
    marginTop: 12,
    padding: 12,
    backgroundColor: 'rgba(255, 193, 7, 0.1)',
    borderRadius: 8,
  },
  warningText: {
    fontSize: 12,
    lineHeight: 16,
  },
});

