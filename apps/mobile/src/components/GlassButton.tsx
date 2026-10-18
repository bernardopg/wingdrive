import type { ReactNode } from "react";
import { Pressable, View } from "react-native";
import {
	LiquidGlassView,
	isLiquidGlassSupported,
} from "@callstack/liquid-glass";

interface GlassButtonProps {
	onPress?: () => void;
	icon?: ReactNode;
	active?: boolean;
	className?: string;
	size?: number;
}

export function GlassButton({
	onPress,
	icon,
	active,
	className,
	size = 40,
}: GlassButtonProps) {
	const iconSize = Math.round(size * 0.5);

	const content = (
		<View
			style={{
				width: size,
				height: size,
				alignItems: "center",
				justifyContent: "center",
			}}
		>
			<View
				style={{
					width: iconSize,
					height: iconSize,
					alignItems: "center",
					justifyContent: "center",
				}}
			>
				{icon}
			</View>
		</View>
	);

	if (isLiquidGlassSupported) {
		return (
			<Pressable
				onPress={onPress}
				className={className}
				style={({ pressed }) => ({
					opacity: pressed ? 0.9 : 1,
					transform: [{ scale: pressed ? 0.95 : 1 }],
				})}
			>
				<LiquidGlassView
					interactive
					effect="regular"
					colorScheme="dark"
					tintColor={active ? "rgba(40, 130, 250, 0.15)" : undefined}
					style={{
						width: size,
						height: size,
						borderRadius: size / 2,
						alignItems: "center",
						justifyContent: "center",
					}}
				>
					{content}
				</LiquidGlassView>
			</Pressable>
		);
	}

	// Fallback for older iOS and Android. One rounded, clipped view: expo-blur on Android does not
	// clip to borderRadius, so the old BlurView plus an inset square showed as a dark box.
	return (
		<Pressable
			onPress={onPress}
			className={className}
			android_ripple={{ color: "rgba(255, 255, 255, 0.12)", foreground: true }}
			hitSlop={8}
			style={({ pressed }) => ({
				width: size,
				height: size,
				borderRadius: size / 2,
				overflow: "hidden",
				alignItems: "center",
				justifyContent: "center",
				backgroundColor: active ? "rgba(40, 130, 250, 0.2)" : "rgba(30, 31, 38, 0.75)",
				borderWidth: 1,
				borderColor: "rgba(128, 128, 128, 0.25)",
				opacity: pressed ? 0.8 : 1,
			})}
		>
			<View
				style={{
					width: iconSize,
					height: iconSize,
					alignItems: "center",
					justifyContent: "center",
				}}
			>
				{icon}
			</View>
		</Pressable>
	);
}
