import { LocalizedText as Text } from "../../../i18n/LocalizedText";
import React from 'react';
import { View } from "react-native";

export default function NetworkScreen() {
  return (
    <View className="flex-1 bg-sidebar items-center justify-center">
      <Text className="text-ink text-xl">Network</Text>
      <Text className="text-ink-dull text-sm mt-2">Coming soon</Text>
    </View>
  );
}
