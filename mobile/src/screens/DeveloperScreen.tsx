import React, {useState} from 'react';
import {
  View,
  Text,
  ScrollView,
  StyleSheet,
  TouchableOpacity,
  Alert,
} from 'react-native';
import {useAppStore} from '@/store/appStore';
import {useTheme} from '@/theme';
import {Card} from '@/components/Card';
import {Button} from '@/components/Button';
import {backendService} from '@/services/backend';
import {notificationService} from '@/services/notifications';

export function DeveloperScreen() {
  const theme = useTheme();
  const {setConnectionStatus, clearLogs, setSettings, settings} = useAppStore();
  const [endpoint, setEndpoint] = useState(settings.endpoint || '');

  const handleSimulateRotation = async () => {
    // Simulate rotation event
    setConnectionStatus('rotating');
    await notificationService.notifyRotation();
    
    setTimeout(() => {
      setConnectionStatus('connected');
    }, 1000);
    
    Alert.alert('Success', 'Rotation simulated');
  };

  const handleSimulateDisconnect = async () => {
    setConnectionStatus('disconnected');
    await notificationService.notifyDisconnected();
    Alert.alert('Success', 'Disconnect simulated');
  };

  const handleFlushLogs = () => {
    clearLogs();
    Alert.alert('Success', 'Logs flushed');
  };

  const handleSwitchEndpoint = (profile: 'LOCAL' | 'LAN' | 'REMOTE') => {
    const endpoints = {
      LOCAL: 'http://127.0.0.1:9464',
      LAN: 'http://192.168.1.100:9464',
      REMOTE: 'https://gateway.example.com',
    };
    
    setSettings({profile, endpoint: endpoints[profile]});
    backendService.setProfile(profile, endpoints[profile]);
    Alert.alert('Success', `Switched to ${profile} profile`);
  };

  return (
    <ScrollView
      style={[styles.container, {backgroundColor: theme.colors.background}]}
      contentContainerStyle={styles.content}>
      <Card testID="developer-card">
        <Text style={[styles.sectionTitle, {color: theme.colors.text}]}>
          Developer Tools
        </Text>
        <Text style={[styles.warning, {color: theme.colors.warning}]}>
          ⚠️ These tools are for testing only
        </Text>
      </Card>

      <Card testID="simulation-card">
        <Text style={[styles.sectionTitle, {color: theme.colors.text}]}>
          Simulations
        </Text>
        <View style={styles.buttonGroup}>
          <Button
            testID="simulate-rotation-button"
            title="Simulate Rotation"
            onPress={handleSimulateRotation}
            variant="secondary"
            style={styles.button}
          />
          <Button
            testID="simulate-disconnect-button"
            title="Simulate Disconnect"
            onPress={handleSimulateDisconnect}
            variant="secondary"
            style={styles.button}
          />
          <Button
            testID="flush-logs-button"
            title="Flush Logs"
            onPress={handleFlushLogs}
            variant="secondary"
            style={styles.button}
          />
        </View>
      </Card>

      <Card testID="endpoint-switch-card">
        <Text style={[styles.sectionTitle, {color: theme.colors.text}]}>
          Quick Endpoint Switch
        </Text>
        <View style={styles.buttonGroup}>
          <Button
            testID="switch-local-button"
            title="LOCAL"
            onPress={() => handleSwitchEndpoint('LOCAL')}
            variant={settings.profile === 'LOCAL' ? 'primary' : 'secondary'}
            style={styles.button}
          />
          <Button
            testID="switch-lan-button"
            title="LAN"
            onPress={() => handleSwitchEndpoint('LAN')}
            variant={settings.profile === 'LAN' ? 'primary' : 'secondary'}
            style={styles.button}
          />
          <Button
            testID="switch-remote-button"
            title="REMOTE"
            onPress={() => handleSwitchEndpoint('REMOTE')}
            variant={settings.profile === 'REMOTE' ? 'primary' : 'secondary'}
            style={styles.button}
          />
        </View>
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
  warning: {
    fontSize: 14,
    marginTop: 8,
  },
  buttonGroup: {
    gap: 12,
    marginTop: 8,
  },
  button: {
    width: '100%',
  },
});

