import { useState } from 'react';
import { Stack } from 'expo-router';
import { GestureHandlerRootView } from 'react-native-gesture-handler';
import { SafeAreaProvider } from 'react-native-safe-area-context';
import { WingDriveProvider } from '../client';
import { AppResetContext } from '../contexts';
import '../global.css';
// Side-effect import: with inlineRequires, a named import of i18n would only run on first use, after
// the first LocalizedText had already rendered without an i18next instance.
import '../i18n';

export default function RootLayout() {
  const [resetKey, setResetKey] = useState(0);

  const resetApp = () => {
    setResetKey((prev) => prev + 1);
  };

  return (
    <GestureHandlerRootView style={{ flex: 1 }} className="bg-sidebar">
      <SafeAreaProvider>
        <AppResetContext.Provider value={{ resetApp }}>
          <WingDriveProvider key={resetKey}>
            <Stack screenOptions={{ headerShown: false }}>
              <Stack.Screen name="(drawer)" />
              <Stack.Screen
                name="search"
                options={{
                  presentation: 'modal',
                  animation: 'slide_from_bottom'
                }}
              />
              <Stack.Screen
                name="jobs"
                options={{
                  presentation: 'modal',
                  animation: 'slide_from_bottom'
                }}
              />
            </Stack>
          </WingDriveProvider>
        </AppResetContext.Provider>
      </SafeAreaProvider>
    </GestureHandlerRootView>
  );
}
