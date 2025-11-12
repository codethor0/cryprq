import {renderHook, act} from '@testing-library/react-native';
import {useAppStore} from '@/store/appStore';

describe('AppStore - Connection State', () => {
  beforeEach(() => {
    useAppStore.setState({
      connectionStatus: {status: 'disconnected'},
      peers: [],
      logs: [],
      throughputHistory: [],
    });
  });

  it('should update connection status', () => {
    const {result} = renderHook(() => useAppStore());
    
    act(() => {
      result.current.setConnectionStatus('connecting');
    });
    expect(result.current.connectionStatus.status).toBe('connecting');

    act(() => {
      result.current.setConnectionStatus('connected', {peerId: 'QmTest123'});
    });
    expect(result.current.connectionStatus.status).toBe('connected');
    expect(result.current.connectionStatus.peerId).toBe('QmTest123');
  });

  it('should handle rotation events', () => {
    const {result} = renderHook(() => useAppStore());
    
    act(() => {
      result.current.setConnectionStatus('connected', {peerId: 'QmTest123'});
    });
    
    act(() => {
      result.current.setConnectionStatus('rotating');
    });
    expect(result.current.connectionStatus.status).toBe('rotating');
    
    act(() => {
      result.current.setConnectionStatus('connected');
    });
    expect(result.current.connectionStatus.status).toBe('connected');
  });

  it('should update metrics', () => {
    const {result} = renderHook(() => useAppStore());
    
    act(() => {
      result.current.updateMetrics({
        latency: 25,
        rotationTimer: 300,
        bytesIn: 1024,
        bytesOut: 2048,
      });
    });
    
    expect(result.current.connectionStatus.latency).toBe(25);
    expect(result.current.connectionStatus.rotationTimer).toBe(300);
    expect(result.current.connectionStatus.bytesIn).toBe(1024);
    expect(result.current.connectionStatus.bytesOut).toBe(2048);
  });
});

describe('AppStore - Rotation Events', () => {
  it('should transition from connected to rotating to connected', () => {
    const {result} = renderHook(() => useAppStore());
    
    act(() => {
      result.current.setConnectionStatus('connected');
    });
    
    act(() => {
      result.current.setConnectionStatus('rotating');
    });
    expect(result.current.connectionStatus.status).toBe('rotating');
    
    act(() => {
      result.current.setConnectionStatus('connected');
    });
    expect(result.current.connectionStatus.status).toBe('connected');
  });
});
