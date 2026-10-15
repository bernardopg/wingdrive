import type { ImageSourcePropType } from "react-native";

/**
 * Turns a shared icon into an Image source.
 *
 * The shared icon helpers are typed for the web, where an imported image is a
 * URL string; Metro resolves the same import to an asset number.
 */
export function imageSource(src: string | number): ImageSourcePropType {
	return typeof src === "number" ? src : { uri: src };
}
