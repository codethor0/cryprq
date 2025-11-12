import React, {useState} from 'react';
import {
  View,
  Text,
  ScrollView,
  StyleSheet,
  TextInput,
  Modal,
  TouchableOpacity,
  Alert,
} from 'react-native';
import {useAppStore} from '@/store/appStore';
import {useTheme} from '@/theme';
import {Card} from '@/components/Card';
import {Button} from '@/components/Button';
import {parseAndValidateMultiaddr} from '@/utils/validation';
import {backendService} from '@/services/backend';
import type {Peer} from '@/types';

export function PeersScreen() {
  const theme = useTheme();
  const {peers, addPeer, removePeer, updatePeer, connect, disconnect} =
    useAppStore();
  const [addModalVisible, setAddModalVisible] = useState(false);
  const [peerId, setPeerId] = useState('');
  const [multiaddr, setMultiaddr] = useState('');
  const [testing, setTesting] = useState<string | null>(null);

  const handleAddPeer = () => {
    const validation = parseAndValidateMultiaddr(multiaddr);
    if (!validation.ok) {
      Alert.alert('Invalid Multiaddr', validation.reason);
      return;
    }

    if (!peerId.trim()) {
      Alert.alert('Error', 'Peer ID is required');
      return;
    }

    const newPeer: Peer = {
      id: peerId.trim(),
      multiaddr: multiaddr.trim(),
      status: 'idle',
    };

    addPeer(newPeer);
    setAddModalVisible(false);
    setPeerId('');
    setMultiaddr('');
  };

  const handleTestReachability = async (peer: Peer) => {
    setTesting(peer.id);
    try {
      // Extract endpoint from multiaddr (simplified)
      // For LAN profile, test TCP connection to the host:port
      const parts = peer.multiaddr.split('/');
      const protocolIndex = parts.findIndex(p => p === 'ip4' || p === 'ip6' || p.startsWith('dns'));
      const portIndex = parts.findIndex(p => p === 'udp' || p === 'tcp');
      
      if (protocolIndex === -1 || portIndex === -1) {
        Alert.alert('Error', 'Invalid multiaddr format');
        setTesting(null);
        return;
      }
      
      const host = parts[protocolIndex + 1];
      const port = parts[portIndex + 1];
      const endpoint = `http://${host}:${port || 9464}`;
      
      const result = await backendService.testReachability(endpoint);
      
      if (result.ok && result.latency !== undefined) {
        updatePeer(peer.id, {latency: result.latency});
        Alert.alert('Success', `Reachable (${result.latency}ms)`);
      } else {
        Alert.alert('Unreachable', result.error || 'Connection failed');
      }
    } catch (error: any) {
      Alert.alert('Error', error.message);
    } finally {
      setTesting(null);
    }
  };

  const handleConnect = async (peer: Peer) => {
    updatePeer(peer.id, {status: 'connecting'});
    try {
      await connect(peer.id);
      updatePeer(peer.id, {status: 'connected'});
    } catch (error) {
      updatePeer(peer.id, {status: 'error'});
    }
  };

  const handleDisconnect = async (peer: Peer) => {
    await disconnect();
    updatePeer(peer.id, {status: 'disconnected'});
  };

  return (
    <View style={[styles.container, {backgroundColor: theme.colors.background}]}>
      <ScrollView style={styles.scrollView} contentContainerStyle={styles.content}>
        {peers.length === 0 ? (
          <Card>
            <Text style={[styles.emptyText, {color: theme.colors.textSecondary}]}>
              No peers configured. Add a peer to get started.
            </Text>
          </Card>
        ) : (
          peers.map(peer => (
            <Card key={peer.id} testID={`peer-${peer.id}`}>
              <View style={styles.peerHeader}>
                <View style={styles.peerInfo}>
                  <Text style={[styles.peerId, {color: theme.colors.text}]}>
                    {peer.alias || peer.id.slice(0, 16) + '...'}
                  </Text>
                  <Text
                    style={[styles.peerMultiaddr, {color: theme.colors.textSecondary}]}
                    numberOfLines={1}>
                    {peer.multiaddr}
                  </Text>
                  {peer.latency !== undefined && (
                    <View style={styles.latencyRow}>
                      <Text style={[styles.peerLatency, {color: theme.colors.textSecondary}]}>
                        Latency: {peer.latency}ms
                      </Text>
                    </View>
                  )}
                  {peer.lastHandshake && (
                    <Text style={[styles.lastHandshake, {color: theme.colors.textSecondary}]}>
                      Last: {new Date(peer.lastHandshake).toLocaleString()}
                    </Text>
                  )}
                </View>
              </View>
              <View style={styles.peerActions}>
                {peer.status === 'connected' ? (
                  <Button
                    testID={`disconnect-${peer.id}`}
                    title="Disconnect"
                    onPress={() => handleDisconnect(peer)}
                    variant="danger"
                    style={styles.actionButton}
                  />
                ) : (
                  <Button
                    testID={`connect-${peer.id}`}
                    title="Connect"
                    onPress={() => handleConnect(peer)}
                    variant="primary"
                    style={styles.actionButton}
                  />
                )}
                <Button
                  testID={`test-${peer.id}`}
                  title="Test"
                  onPress={() => handleTestReachability(peer)}
                  variant="secondary"
                  style={styles.actionButton}
                  loading={testing === peer.id}
                />
                <Button
                  testID={`remove-${peer.id}`}
                  title="Remove"
                  onPress={() => removePeer(peer.id)}
                  variant="danger"
                  style={styles.actionButton}
                />
              </View>
            </Card>
          ))
        )}
      </ScrollView>

      <View style={[styles.footer, {backgroundColor: theme.colors.surface}]}>
        <Button
          testID="add-peer-button"
          title="Add Peer"
          onPress={() => setAddModalVisible(true)}
          variant="primary"
        />
      </View>

      <Modal
        visible={addModalVisible}
        animationType="slide"
        transparent={true}
        onRequestClose={() => setAddModalVisible(false)}>
        <View style={styles.modalOverlay}>
          <Card style={styles.modalContent}>
            <Text style={[styles.modalTitle, {color: theme.colors.text}]}>
              Add Peer
            </Text>
            <TextInput
              testID="peer-id-input"
              style={[
                styles.input,
                {
                  backgroundColor: theme.colors.background,
                  color: theme.colors.text,
                  borderColor: theme.colors.border,
                },
              ]}
              placeholder="Peer ID"
              placeholderTextColor={theme.colors.textSecondary}
              value={peerId}
              onChangeText={setPeerId}
            />
            <TextInput
              testID="multiaddr-input"
              style={[
                styles.input,
                {
                  backgroundColor: theme.colors.background,
                  color: theme.colors.text,
                  borderColor: theme.colors.border,
                },
              ]}
              placeholder="/ip4/127.0.0.1/udp/9999/quic-v1/p2p/Qm..."
              placeholderTextColor={theme.colors.textSecondary}
              value={multiaddr}
              onChangeText={setMultiaddr}
            />
            <View style={styles.modalActions}>
              <Button
                testID="cancel-add-peer"
                title="Cancel"
                onPress={() => setAddModalVisible(false)}
                variant="secondary"
                style={styles.modalButton}
              />
              <Button
                testID="confirm-add-peer"
                title="Add"
                onPress={handleAddPeer}
                variant="primary"
                style={styles.modalButton}
              />
            </View>
          </Card>
        </View>
      </Modal>
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
  },
  scrollView: {
    flex: 1,
  },
  content: {
    padding: 16,
  },
  emptyText: {
    textAlign: 'center',
    padding: 20,
  },
  peerHeader: {
    marginBottom: 12,
  },
  peerInfo: {
    marginBottom: 8,
  },
  peerId: {
    fontSize: 16,
    fontWeight: '600',
    marginBottom: 4,
  },
  peerMultiaddr: {
    fontSize: 12,
    marginBottom: 4,
  },
  latencyRow: {
    marginTop: 4,
  },
  peerLatency: {
    fontSize: 12,
  },
  lastHandshake: {
    fontSize: 10,
    marginTop: 2,
  },
  peerActions: {
    flexDirection: 'row',
    gap: 8,
  },
  actionButton: {
    flex: 1,
  },
  footer: {
    padding: 16,
    borderTopWidth: 1,
    borderTopColor: '#E0E0E0',
  },
  modalOverlay: {
    flex: 1,
    backgroundColor: 'rgba(0,0,0,0.5)',
    justifyContent: 'center',
    alignItems: 'center',
  },
  modalContent: {
    width: '90%',
    maxWidth: 400,
  },
  modalTitle: {
    fontSize: 20,
    fontWeight: '600',
    marginBottom: 16,
  },
  input: {
    borderWidth: 1,
    borderRadius: 8,
    padding: 12,
    marginBottom: 12,
    fontSize: 16,
  },
  modalActions: {
    flexDirection: 'row',
    gap: 12,
    marginTop: 8,
  },
  modalButton: {
    flex: 1,
  },
});

