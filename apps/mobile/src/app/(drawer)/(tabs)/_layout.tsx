import { Platform } from 'react-native';
import { NativeTabs } from 'expo-router/unstable-native-tabs';

export default function TabLayout() {
  return (
    <NativeTabs
      // CRITICAL: sem background custom no iOS preserva o efeito liquid glass
      backgroundColor={Platform.OS === 'ios' ? undefined : 'hsl(235, 10%, 6%)'}
      disableTransparentOnScrollEdge={true}
      iconColor={Platform.OS === 'android' ? 'hsl(235, 10%, 55%)' : undefined}
      labelStyle={Platform.OS === 'android' ? {
        color: 'hsl(235, 10%, 55%)'
      } : undefined}
    >
      <NativeTabs.Trigger name="overview">
        <NativeTabs.Trigger.Label>Overview</NativeTabs.Trigger.Label>
        <NativeTabs.Trigger.Icon sf="square.grid.2x2" md="grid_view" />
      </NativeTabs.Trigger>

      <NativeTabs.Trigger name="browse">
        <NativeTabs.Trigger.Label>Browse</NativeTabs.Trigger.Label>
        <NativeTabs.Trigger.Icon sf="folder" md="folder" />
      </NativeTabs.Trigger>

      <NativeTabs.Trigger name="settings">
        <NativeTabs.Trigger.Label>Settings</NativeTabs.Trigger.Label>
        <NativeTabs.Trigger.Icon sf="gearshape" md="settings" />
      </NativeTabs.Trigger>
    </NativeTabs>
  );
}
