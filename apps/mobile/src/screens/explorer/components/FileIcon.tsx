import { Image } from "react-native";
import { Image as ExpoImage } from "expo-image";
import type { File } from "@wingdrive/ts-client";
import { getFileKindForIcon, getVirtualMetadata } from "@wingdrive/ts-client";
import { getIcon } from "@wingdrive/assets/util/mobile";
import { imageSource } from "../../../utils/imageSource";
import { previewKind } from "../fileLogic";

/**
 * A file's kind icon, or a preview of an image or video stored on this phone. The mobile core
 * builds no thumbnails, so local files are decoded by the image loader, downsampled and cached.
 */
export function FileIcon({
	file,
	localPath,
	size,
}: {
	file: File;
	/** The file's path when it is on this phone */
	localPath: string | null;
	/** Edge in points */
	size: number;
}) {
	const virtualMetadata = getVirtualMetadata(file);
	const iconSource =
		virtualMetadata?.iconUrl ??
		getIcon(getFileKindForIcon(file), true, file.extension, file.kind === "Directory");

	if (localPath && file.kind !== "Directory" && previewKind(file.extension)) {
		return (
			<ExpoImage
				source={{ uri: `file://${localPath}` }}
				// The kind icon: an asset number from Metro or a URL.
				placeholder={typeof iconSource === "number" ? iconSource : { uri: iconSource }}
				placeholderContentFit="contain"
				contentFit="cover"
				recyclingKey={localPath}
				cachePolicy="memory-disk"
				style={{ width: size, height: size, borderRadius: 6 }}
			/>
		);
	}

	return (
		<Image
			source={imageSource(iconSource)}
			style={{ width: size, height: size, resizeMode: "contain" }}
		/>
	);
}
