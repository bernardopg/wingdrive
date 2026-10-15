import { LocalizedText as Text } from "../../i18n/LocalizedText";
import React from "react";

import { SettingsRow, SettingsRowProps } from "./SettingsRow";

interface SettingsOptionProps extends SettingsRowProps {
	value?: string;
}

export function SettingsOption({ value, ...props }: SettingsOptionProps) {
	return (
		<SettingsRow
			{...props}
			trailing={
				value ? (
					<Text className="text-ink-dull mr-2">{value}</Text>
				) : undefined
			}
		/>
	);
}
