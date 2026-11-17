import { LocalizedText as Text } from "../../i18n/LocalizedText";
import { useState, useRef, useCallback } from "react";
import { Alert, Pressable, View, ScrollView, Dimensions, type NativeScrollEvent, type NativeSyntheticEvent, type ScrollViewInstance } from "react-native";
import { useSafeAreaInsets, type EdgeInsets } from "react-native-safe-area-context";
import Animated, {
	useSharedValue,
	useAnimatedScrollHandler,
	useAnimatedStyle,
	interpolate,
	Extrapolation,
} from "react-native-reanimated";
import { useLibraryAction, useNormalizedQuery } from "../../client";
import { useTranslation } from "react-i18next";
import { RenameDialog } from "../../components/RenameDialog";
import { PageIndicator } from "../../components/PageIndicator";
import { GlassSearchBar } from "../../components/GlassSearchBar";
import { useRouter } from "expo-router";
import { WingItem as WingItemComponent, WingGroupComponent } from "./components";
import { SettingsGroup } from "../../components/primitive";
import { StorageAccessCard } from "../../components/StorageAccessCard";

interface Wing {
	id: string;
	name: string;
	color: string;
}

const SCREEN_WIDTH = Dimensions.get("window").width;

function WingContent({
	wing,
	insets
}: {
	wing: Wing;
	insets: EdgeInsets;
}) {
	const router = useRouter();
	const scrollY = useSharedValue(0);

	const scrollHandler = useAnimatedScrollHandler({
		onScroll: (event) => {
			scrollY.value = event.contentOffset.y;
		},
	});

	const handleSearchPress = () => {
		router.push("/search");
	};

	// Fetch wing layout
	const { data: layout } = useNormalizedQuery({
		query: "wings.get_layout",
		input: { wing_id: wing.id },
		resourceType: "wing_layout",
		resourceId: wing.id,
		enabled: !!wing.id,
	});

	// Wing name scale on overscroll (anchored left)
	const wingNameScale = useAnimatedStyle(() => {
		const scale = interpolate(
			scrollY.value,
			[-200, 0],
			[1.3, 1],
			Extrapolation.CLAMP
		);

		return {
			transform: [{ scale }],
			transformOrigin: 'left center',
		};
	});

	// Filter out Overview items (mobile doesn't show Overview in browse tab)
	const wingItems = (layout?.wing_items || []).filter(
		(item) => item.item_type !== "Overview"
	);
	const groups = layout?.groups || [];

	return (
		<Animated.ScrollView
			style={{ width: SCREEN_WIDTH }}
			contentContainerStyle={{
				paddingTop: insets.top + 45,
				paddingHorizontal: 16,
				paddingBottom: insets.bottom + 60,
			}}
			showsVerticalScrollIndicator={false}
			onScroll={scrollHandler}
			scrollEventThrottle={16}
		>
			{/* Header */}
			<View className="mb-6">
				<View className="flex-row items-center gap-2">
					<View
						className="w-4 h-4 mx-1 rounded-full"
						style={{ backgroundColor: wing.color }}
					/>
					<Animated.Text
						style={[wingNameScale]}
						className="text-ink text-[30px] font-bold"
					>
						{wing.name}
					</Animated.Text>
				</View>
			</View>

			{/* Search Bar */}
			<View className="mb-6">
				<GlassSearchBar onPress={handleSearchPress} editable={false} />
			</View>

			<StorageAccessCard />

			{/* Wing Items (pinned shortcuts) */}
			{wingItems.length > 0 && (
				<View className="mb-6">
					<SettingsGroup>
						{wingItems.map((item) => (
							<WingItemComponent key={item.id} item={item} />
						))}
					</SettingsGroup>
				</View>
			)}

			{/* Groups */}
			{groups.map(({ group, items }) => (
				<WingGroupComponent key={group.id} group={group} items={items} />
			))}

			{/* A new wing has nothing in it, and items are added on the desktop for now (MOB-017). */}
			{layout && wingItems.length === 0 && groups.length === 0 && (
				<View className="items-center px-6 py-10">
					<Text className="text-ink font-semibold text-base mb-2 text-center">This Wing is empty</Text>
					<Text className="text-ink-dull text-sm text-center">
						Add folders, devices and shortcuts to it from the sidebar of WingDrive on your computer. They show up here once the devices sync.
					</Text>
				</View>
			)}
		</Animated.ScrollView>
	);
}

/** Colors offered to new wings, the same palette as the desktop's "New Wing". */
const WING_COLORS = ["#3B82F6", "#F97316", "#22C55E", "#A855F7", "#EF4444", "#EAB308"];

function CreateWingScreen({ wingCount }: { wingCount: number }) {
	const { t } = useTranslation("common");
	const [naming, setNaming] = useState(false);
	const createWing = useLibraryAction("wings.create");
	return (
		<View
			style={{ width: SCREEN_WIDTH, paddingHorizontal: 16 }}
			className="flex-1 items-center justify-center"
		>
			<Pressable
				onPress={() => setNaming(true)}
				accessibilityRole="button"
				className="items-center"
			>
				<View className="w-16 h-16 rounded-full bg-accent/10 items-center justify-center mb-4">
					<Text className="text-accent text-3xl">+</Text>
				</View>
				<Text className="text-xl font-bold text-ink mb-2">
					Create New Wing
				</Text>
				<Text className="text-ink-dull text-center max-w-xs">
					Organize your files, devices, and locations into separate Wings
				</Text>
			</Pressable>
			{naming && (
				<RenameDialog
					title={t("Create New Wing")}
					initialValue=""
					onClose={() => setNaming(false)}
					onSubmit={(name) => {
						setNaming(false);
						createWing.mutate(
							{ name, icon: "Planet", color: WING_COLORS[wingCount % WING_COLORS.length] },
							{ onError: (error) => Alert.alert(t("Error"), error.message) },
						);
					}}
				/>
			)}
		</View>
	);
}

export function BrowseScreen() {
	const insets = useSafeAreaInsets();
	const { data: wingsData } = useNormalizedQuery({
		query: "wings.list",
		input: null,
		resourceType: "wing",
	});
	const [currentPage, setCurrentPage] = useState(0);
	const scrollViewRef = useRef<ScrollViewInstance>(null);

	const wingsList = (wingsData?.wings || []) as Wing[];
	const totalPages = wingsList.length + 1; // +1 for create wing page

	const handleScroll = useCallback(
		(event: NativeSyntheticEvent<NativeScrollEvent>) => {
			const offsetX = event.nativeEvent.contentOffset.x;
			const page = Math.round(offsetX / SCREEN_WIDTH);
			setCurrentPage(page);
		},
		[]
	);

	// Build page colors array - wing colors for wing pages, accent for create page
	const pageColors = [
		...wingsList.map((wing) => wing.color),
		"hsl(208, 100%, 57%)", // Create page uses accent color
	];

	return (
		<View className="flex-1 bg-app">
			<ScrollView
				ref={scrollViewRef}
				horizontal
				pagingEnabled
				showsHorizontalScrollIndicator={false}
				onScroll={handleScroll}
				scrollEventThrottle={16}
				decelerationRate="fast"
			>
				{wingsList.map((wing) => (
					<WingContent
						key={wing.id}
						wing={wing}
						insets={insets}
					/>
				))}
				<CreateWingScreen wingCount={wingsList.length} />
			</ScrollView>

			{/* Floating Wing Indicator */}
			<View
				className="absolute left-0 right-0"
				style={{
					top: insets.top + 16,
				}}
				pointerEvents="none"
			>
				<View className="mb-4">
					<PageIndicator
						currentIndex={currentPage}
						totalPages={totalPages}
						pageColors={pageColors}
					/>
				</View>
			</View>
		</View>
	);
}
