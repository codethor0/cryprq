import React from 'react';
import {View, Text, ScrollView, StyleSheet, Linking} from 'react-native';
import {useTheme} from '@/theme';
import {Card} from '@/components/Card';
import {Button} from '@/components/Button';

const LICENSE_URL = 'https://github.com/yourusername/CrypRQ/blob/main/LICENSE';
const REPO_URL = 'https://github.com/yourusername/CrypRQ';

export function AboutScreen() {
  const theme = useTheme();
  const version = '1.0.0';
  const buildNumber = '1';
  const commitHash = process.env.COMMIT_HASH || 'dev';

  return (
    <ScrollView
      style={[styles.container, {backgroundColor: theme.colors.background}]}
      contentContainerStyle={styles.content}>
      <Card>
        <Text style={[styles.appName, {color: theme.colors.text}]}>CrypRQ Mobile</Text>
        <Text style={[styles.version, {color: theme.colors.textSecondary}]}>
          Version {version} (Build {buildNumber})
        </Text>
        <Text style={[styles.commit, {color: theme.colors.textSecondary}]}>
          Commit: {commitHash.slice(0, 7)}
        </Text>
      </Card>

      <Card>
        <Text style={[styles.sectionTitle, {color: theme.colors.text}]}>
          Description
        </Text>
        <Text style={[styles.bodyText, {color: theme.colors.textSecondary}]}>
          CrypRQ Mobile is a controller app for managing CrypRQ nodes. It provides
          real-time monitoring, peer management, and secure connection handling.
        </Text>
      </Card>

      <Card>
        <Text style={[styles.sectionTitle, {color: theme.colors.text}]}>
          Open Source
        </Text>
        <Text style={[styles.bodyText, {color: theme.colors.textSecondary}]}>
          CrypRQ is open source software. View the source code, report issues, or
          contribute on GitHub.
        </Text>
        <Button
          testID="open-repo-button"
          title="View on GitHub"
          onPress={() => Linking.openURL(REPO_URL)}
          variant="secondary"
          style={styles.button}
        />
      </Card>

      <Card>
        <Text style={[styles.sectionTitle, {color: theme.colors.text}]}>
          License
        </Text>
        <Text style={[styles.bodyText, {color: theme.colors.textSecondary}]}>
          Licensed under the MIT License.
        </Text>
        <Button
          testID="open-license-button"
          title="View License"
          onPress={() => Linking.openURL(LICENSE_URL)}
          variant="secondary"
          style={styles.button}
        />
      </Card>

      <Card>
        <Text style={[styles.sectionTitle, {color: theme.colors.text}]}>
          Acknowledgments
        </Text>
        <Text style={[styles.bodyText, {color: theme.colors.textSecondary}]}>
          Built with React Native, Zustand, and other open source libraries.
          See LICENSE file for full attribution.
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
  appName: {
    fontSize: 24,
    fontWeight: '700',
    marginBottom: 4,
  },
  version: {
    fontSize: 14,
    marginBottom: 2,
  },
  commit: {
    fontSize: 12,
    fontFamily: 'monospace',
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
});

