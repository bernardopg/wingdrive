// Type declarations for @wingdrive/assets

declare module "@wingdrive/assets/icons/*.png" {
	const value: number; // React Native uses numeric IDs for local images
	export default value;
}

declare module "@wingdrive/assets/icons/*.jpg" {
	const value: number;
	export default value;
}

declare module "@wingdrive/assets/images/*.png" {
	const value: number;
	export default value;
}

declare module "@wingdrive/assets/images/*.jpg" {
	const value: number;
	export default value;
}

declare module "@wingdrive/assets/svgs/*.svg" {
	import type { FC } from "react";
	const content: FC<Record<string, unknown>>;
	export default content;
}

declare module "*.svg" {
	const src: string;
	export default src;
	export const ReactComponent: React.FC<React.SVGProps<SVGSVGElement>>;
}

declare module "@wingdrive/assets/videos/*.mp4" {
	const value: number;
	export default value;
}

declare module "@wingdrive/assets/sounds/*.mp3" {
	const value: number | string; // number on React Native (asset ID), string on web (URL)
	export default value;
}

declare module "@wingdrive/assets/lottie/*.json" {
	const value: object;
	export default value;
}
