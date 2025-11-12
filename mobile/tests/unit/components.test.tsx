import React from 'react';
import {render} from '@testing-library/react-native';
import {StatusPill} from '@/components/StatusPill';
import {Card} from '@/components/Card';
import {Button} from '@/components/Button';

describe('Components', () => {
  describe('StatusPill', () => {
    it('should render connected status', () => {
      const {getByTestId, getByText} = render(
        <StatusPill status="connected" testID="status-pill" />,
      );
      expect(getByTestId('status-pill')).toBeTruthy();
      expect(getByText('Connected')).toBeTruthy();
    });

    it('should render disconnected status', () => {
      const {getByText} = render(<StatusPill status="disconnected" />);
      expect(getByText('Disconnected')).toBeTruthy();
    });
  });

  describe('Card', () => {
    it('should render children', () => {
      const {getByText} = render(
        <Card testID="test-card">
          <Text>Test Content</Text>
        </Card>,
      );
      expect(getByText('Test Content')).toBeTruthy();
    });
  });

  describe('Button', () => {
    it('should render button with title', () => {
      const {getByText} = render(
        <Button title="Test Button" onPress={() => {}} testID="test-button" />,
      );
      expect(getByText('Test Button')).toBeTruthy();
    });

    it('should show loading state', () => {
      const {getByTestId} = render(
        <Button title="Test" onPress={() => {}} loading={true} testID="test-button" />,
      );
      // ActivityIndicator should be present when loading
      expect(getByTestId('test-button')).toBeTruthy();
    });
  });
});

