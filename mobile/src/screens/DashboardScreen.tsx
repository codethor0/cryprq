import React, {useEffect, useState} from 'react';
import {
  View,
  Text,
  ScrollView,
  StyleSheet,
  TouchableOpacity,
} from 'react-native';
import {useNavigation} from '@react-navigation/native';
import {useAppStore} from '@/store/appStore';
import {useTheme} from '@/theme';
import {StatusPill} from '@/components/StatusPill';
import {Card} from '@/components/Card';
import {Button} from '@/components/Button';
import {VictoryChart, VictoryLine, VictoryAxis, VictoryArea} from 'victory-native';
import type {NativeStackNavigationProp} from '@react-navigation/native-stack';

type NavigationProp = NativeStackNavigationProp<any>;

export function DashboardScreen() {
  const navigation = useNavigation<NavigationProp>();
  const theme = useTheme();
  const {
    connectionStatus,
    throughputHistory,
    connect,
    disconnect,
  } = useAppStore();
  const [rotationCountdown, setRotationCountdown] = useState<number | null>(null);
  const [isRotating, setIsRotating] = useState(false);
  const [lastRotationToast, setLastRotationToast] = useState<number>(0);

  useEffect(() => {
    if (connectionStatus.rotationTimer !== undefined) {
      setRotationCountdown(Math.floor(connectionStatus.rotationTimer));
    }
  }, [connectionStatus.rotationTimer]);

  useEffect(() => {
    if (rotationCountdown !== null && rotationCountdown > 0 && connectionStatus.status === 'connected' && !isRotating) {
      const timer = setInterval(() => {
        setRotationCountdown(prev => (prev !== null ? Math.max(0, prev - 1) : null));
      }, 1000);
      return () => clearInterval(timer);
    }
  }, [rotationCountdown, connectionStatus.status, isRotating]);

  // Listen to rotation events from backend service
  useEffect(() => {
    const handleRotation = (event: { type: string; nextInSeconds?: number }) => {
      if (event.type === 'rotation.started') {
        setIsRotating(true);
      } else if (event.type === 'rotation.completed') {
        setIsRotating(false);
        if (event.nextInSeconds !== undefined) {
          setRotationCountdown(event.nextInSeconds);
        }
        const now = Date.now();
        if (now - lastRotationToast > 2000) {
          // Toast notification would go here
          setLastRotationToast(now);
        }
      } else if (event.type === 'rotation.scheduled' && event.nextInSeconds !== undefined) {
        setRotationCountdown(event.nextInSeconds);
      }
    };

    backendService.on('rotation', handleRotation);

    // Also resync from metrics every 2s to handle jitter
    const metricsInterval = setInterval(() => {
      const {connectionStatus: currentStatus} = useAppStore.getState();
      if (currentStatus.status === 'connected' && currentStatus.rotationTimer !== undefined) {
        setRotationCountdown(currentStatus.rotationTimer);
      }
    }, 2000);

    return () => {
      backendService.off('rotation', handleRotation);
      clearInterval(metricsInterval);
    };
  }, [lastRotationToast]);

  const formatTime = (seconds: number) => {
    const mins = Math.floor(seconds / 60);
    const secs = seconds % 60;
    return `${mins}:${secs.toString().padStart(2, '0')}`;
  };

  const formatBytes = (bytes: number) => {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  };

  const chartData = throughputHistory.map((point, index) => ({
    x: index,
    y: point.bytesIn + point.bytesOut,
  }));

  return (
    <ScrollView
      style={[styles.container, {backgroundColor: theme.colors.background}]}
      contentContainerStyle={styles.content}>
      <Card testID="status-card">
        <View style={styles.statusRow}>
          <Text style={[styles.label, {color: theme.colors.text}]}>
            Status
          </Text>
          <StatusPill 
            status={isRotating ? 'rotating' : connectionStatus.status} 
            testID="status-pill" 
          />
        </View>
        
        {/* Ensure status updates within 2s */}
        {connectionStatus.status === 'connecting' && (
          <View style={styles.connectingIndicator}>
            <Text style={[styles.connectingText, {color: theme.colors.textSecondary}]}>
              Connecting...
            </Text>
          </View>
        )}

        {connectionStatus.peerId && (
          <View style={styles.infoRow}>
            <Text style={[styles.label, {color: theme.colors.text}]}>
              Peer ID:
            </Text>
            <Text
              style={[styles.value, {color: theme.colors.textSecondary}]}
              numberOfLines={1}
              ellipsizeMode="middle">
              {connectionStatus.peerId}
            </Text>
          </View>
        )}

        {connectionStatus.latency !== undefined && (
          <View style={styles.infoRow}>
            <Text style={[styles.label, {color: theme.colors.text}]}>
              Latency:
            </Text>
            <Text style={[styles.value, {color: theme.colors.textSecondary}]}>
              {connectionStatus.latency.toFixed(0)} ms
            </Text>
          </View>
        )}

        {rotationCountdown !== null && (
          <View style={styles.infoRow}>
            <Text style={[styles.label, {color: theme.colors.text}]}>
              Rotation in:
            </Text>
            <Text style={[styles.value, {color: theme.colors.textSecondary}]}>
              {formatTime(rotationCountdown)}
            </Text>
          </View>
        )}
      </Card>

      <Card testID="throughput-card">
        <Text style={[styles.sectionTitle, {color: theme.colors.text}]}>
          Throughput (60s)
        </Text>
        {chartData.length > 0 ? (
          <View style={styles.chartContainer}>
            <VictoryChart
              height={200}
              padding={{left: 50, right: 20, top: 20, bottom: 40}}>
              <VictoryAxis
                style={{
                  axis: {stroke: theme.colors.border},
                  tickLabels: {fill: theme.colors.textSecondary, fontSize: 10},
                }}
              />
              <VictoryAxis
                dependentAxis
                style={{
                  axis: {stroke: theme.colors.border},
                  tickLabels: {fill: theme.colors.textSecondary, fontSize: 10},
                }}
              />
              <VictoryArea
                data={chartData}
                style={{
                  data: {
                    fill: theme.colors.primary,
                    fillOpacity: 0.3,
                    stroke: theme.colors.primary,
                    strokeWidth: 2,
                  },
                }}
              />
            </VictoryChart>
          </View>
        ) : (
          <Text style={[styles.emptyText, {color: theme.colors.textSecondary}]}>
            No data yet
          </Text>
        )}
      </Card>

      <Card testID="stats-card">
        <Text style={[styles.sectionTitle, {color: theme.colors.text}]}>
          Statistics
        </Text>
        <View style={styles.statsRow}>
          <View style={styles.statItem}>
            <Text style={[styles.statLabel, {color: theme.colors.textSecondary}]}>
              Bytes In
            </Text>
            <Text style={[styles.statValue, {color: theme.colors.text}]}>
              {formatBytes(connectionStatus.bytesIn || 0)}
            </Text>
          </View>
          <View style={styles.statItem}>
            <Text style={[styles.statLabel, {color: theme.colors.textSecondary}]}>
              Bytes Out
            </Text>
            <Text style={[styles.statValue, {color: theme.colors.text}]}>
              {formatBytes(connectionStatus.bytesOut || 0)}
            </Text>
          </View>
        </View>
      </Card>

      <View style={styles.actions}>
        {connectionStatus.status === 'disconnected' ||
        connectionStatus.status === 'errored' ? (
          <Button
            testID="connect-button"
            title="Connect"
            onPress={() => connect()}
            variant="primary"
          />
        ) : (
          <Button
            testID="disconnect-button"
            title="Disconnect"
            onPress={() => disconnect()}
            variant="danger"
          />
        )}
      </View>

      <TouchableOpacity
        testID="view-logs-button"
        onPress={() => navigation.navigate('Logs')}
        style={styles.logsButton}>
        <Text style={[styles.logsButtonText, {color: theme.colors.primary}]}>
          View Logs
        </Text>
      </TouchableOpacity>
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
  statusRow: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: 12,
  },
  infoRow: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    marginTop: 8,
  },
  label: {
    fontSize: 16,
    fontWeight: '600',
  },
  value: {
    fontSize: 16,
    flex: 1,
    textAlign: 'right',
    marginLeft: 16,
  },
  sectionTitle: {
    fontSize: 18,
    fontWeight: '600',
    marginBottom: 12,
  },
  chartContainer: {
    marginTop: 8,
  },
  emptyText: {
    textAlign: 'center',
    padding: 20,
    fontSize: 14,
  },
  statsRow: {
    flexDirection: 'row',
    justifyContent: 'space-around',
    marginTop: 8,
  },
  statItem: {
    alignItems: 'center',
  },
  statLabel: {
    fontSize: 12,
    marginBottom: 4,
  },
  statValue: {
    fontSize: 18,
    fontWeight: '600',
  },
  actions: {
    marginTop: 16,
  },
  logsButton: {
    marginTop: 16,
    padding: 12,
    alignItems: 'center',
  },
  logsButtonText: {
    fontSize: 16,
    fontWeight: '600',
  },
});

