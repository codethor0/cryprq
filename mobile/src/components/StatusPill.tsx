import React from 'react';
import {View, Text, StyleSheet} from 'react-native';
import {useTheme} from '@/theme';
import type {ConnectionStatus} from '@/types';

interface StatusPillProps {
  status: ConnectionStatus;
  testID?: string;
}

export function StatusPill({status, testID}: StatusPillProps) {
  const theme = useTheme();

  const getStatusColor = () => {
    switch (status) {
      case 'connected':
        return theme.colors.statusConnected;
      case 'connecting':
        return theme.colors.statusConnecting;
      case 'rotating':
        return theme.colors.statusRotating;
      case 'errored':
        return theme.colors.error;
      default:
        return theme.colors.statusDisconnected;
    }
  };

  const getStatusText = () => {
    switch (status) {
      case 'connected':
        return 'Connected';
      case 'connecting':
        return 'Connecting';
      case 'rotating':
        return 'Rotating Keys';
      case 'errored':
        return 'Error';
      default:
        return 'Disconnected';
    }
  };

  return (
    <View
      style={[styles.pill, {backgroundColor: getStatusColor()}]}
      testID={testID}>
      <Text style={styles.text}>{getStatusText()}</Text>
    </View>
  );
}

const styles = StyleSheet.create({
  pill: {
    paddingHorizontal: 12,
    paddingVertical: 6,
    borderRadius: 16,
    alignSelf: 'flex-start',
  },
  text: {
    color: '#FFFFFF',
    fontSize: 12,
    fontWeight: '600',
  },
});

