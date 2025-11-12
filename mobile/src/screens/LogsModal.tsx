import React, {useState} from 'react';
import {
  View,
  Text,
  ScrollView,
  StyleSheet,
  TextInput,
  TouchableOpacity,
} from 'react-native';
import {useAppStore} from '@/store/appStore';
import {useTheme} from '@/theme';
import {Card} from '@/components/Card';
import {Button} from '@/components/Button';
import type {LogLine} from '@/types';

export function LogsModal() {
  const theme = useTheme();
  const {logs, clearLogs} = useAppStore();
  const [searchQuery, setSearchQuery] = useState('');
  const [levelFilter, setLevelFilter] = useState<'all' | 'debug' | 'info' | 'warn' | 'error'>('all');
  
  // Ensure we show last 200 lines
  const displayedLogs = logs.slice(-200);

  const filteredLogs = displayedLogs.filter(log => {
    const matchesSearch = !searchQuery || log.msg.toLowerCase().includes(searchQuery.toLowerCase());
    const matchesLevel = levelFilter === 'all' || log.level === levelFilter;
    
    // Special filters for "error" and "rotation"
    if (searchQuery) {
      const lowerQuery = searchQuery.toLowerCase();
      if (lowerQuery === 'error') {
        return matchesLevel && log.level === 'error';
      }
      if (lowerQuery === 'rotation') {
        return (
          log.msg.toLowerCase().includes('rotation') ||
          log.event === 'rotation.completed' ||
          log.event === 'rotation.started' ||
          log.meta?.event === 'rotation.completed' ||
          log.meta?.event === 'rotation.started'
        );
      }
    }
    
    return matchesSearch && matchesLevel;
  });

  const getLevelColor = (level: string) => {
    switch (level) {
      case 'error':
        return theme.colors.error;
      case 'warn':
        return theme.colors.warning;
      case 'info':
        return theme.colors.info;
      default:
        return theme.colors.textSecondary;
    }
  };

  return (
    <View style={[styles.container, {backgroundColor: theme.colors.background}]}>
      <View style={[styles.header, {backgroundColor: theme.colors.surface, borderBottomColor: theme.colors.border}]}>
        <Text style={[styles.title, {color: theme.colors.text}]}>Logs</Text>
        <View style={styles.headerActions}>
          <Button
            testID="clear-logs-button"
            title="Clear"
            onPress={clearLogs}
            variant="secondary"
            style={styles.clearButton}
          />
        </View>
      </View>

      <View style={[styles.filters, {backgroundColor: theme.colors.surface}]}>
        <TextInput
          testID="log-search-input"
          style={[
            styles.searchInput,
            {
              backgroundColor: theme.colors.background,
              color: theme.colors.text,
              borderColor: theme.colors.border,
            },
          ]}
          placeholder="Search logs..."
          placeholderTextColor={theme.colors.textSecondary}
          value={searchQuery}
          onChangeText={setSearchQuery}
        />
        <View style={styles.levelFilters}>
          {(['all', 'error', 'warn', 'info'] as const).map(level => (
            <TouchableOpacity
              key={level}
              testID={`filter-${level}`}
              style={[
                styles.filterButton,
                {
                  backgroundColor:
                    levelFilter === level ? theme.colors.primary : theme.colors.background,
                },
              ]}
              onPress={() => setLevelFilter(level)}>
              <Text
                style={[
                  styles.filterButtonText,
                  {
                    color:
                      levelFilter === level ? '#FFFFFF' : theme.colors.text,
                  },
                ]}>
                {level.toUpperCase()}
              </Text>
            </TouchableOpacity>
          ))}
        </View>
      </View>

      <ScrollView style={styles.logsContainer} contentContainerStyle={styles.logsContent}>
        {filteredLogs.length === 0 ? (
          <Card>
            <Text style={[styles.emptyText, {color: theme.colors.textSecondary}]}>
              No logs available
            </Text>
          </Card>
        ) : (
          filteredLogs.map((log, index) => (
            <View
              key={index}
              style={[
                styles.logLine,
                {
                  backgroundColor: theme.colors.surface,
                  borderLeftColor: getLevelColor(log.level),
                },
              ]}>
              <Text style={[styles.logTimestamp, {color: theme.colors.textSecondary}]}>
                {new Date(log.ts).toLocaleTimeString()}
              </Text>
              <Text style={[styles.logLevel, {color: getLevelColor(log.level)}]}>
                {log.level.toUpperCase()}
              </Text>
              <Text style={[styles.logMessage, {color: theme.colors.text}]}>
                {log.msg}
              </Text>
            </View>
          ))
        )}
      </ScrollView>
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
  },
  header: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    padding: 16,
    borderBottomWidth: 1,
  },
  title: {
    fontSize: 20,
    fontWeight: '600',
  },
  headerActions: {
    flexDirection: 'row',
  },
  clearButton: {
    paddingHorizontal: 12,
    paddingVertical: 6,
  },
  filters: {
    padding: 16,
    borderBottomWidth: 1,
    borderBottomColor: '#E0E0E0',
  },
  searchInput: {
    borderWidth: 1,
    borderRadius: 8,
    padding: 12,
    fontSize: 16,
    marginBottom: 12,
  },
  levelFilters: {
    flexDirection: 'row',
    gap: 8,
  },
  filterButton: {
    paddingHorizontal: 12,
    paddingVertical: 6,
    borderRadius: 16,
  },
  filterButtonText: {
    fontSize: 12,
    fontWeight: '600',
  },
  logsContainer: {
    flex: 1,
  },
  logsContent: {
    padding: 16,
  },
  logLine: {
    padding: 12,
    marginBottom: 8,
    borderRadius: 8,
    borderLeftWidth: 4,
  },
  logTimestamp: {
    fontSize: 10,
    marginBottom: 4,
  },
  logLevel: {
    fontSize: 10,
    fontWeight: '600',
    marginBottom: 4,
  },
  logMessage: {
    fontSize: 14,
  },
  emptyText: {
    textAlign: 'center',
    padding: 20,
  },
});

