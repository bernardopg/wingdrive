// Declarations locais do app mobile.
// PNG/JPG/MP4 são numeric IDs no React Native; MP3 é number (RN) ou string (web),
// espelhando as declarações de packages/assets/types.d.ts para não conflitar.
// SVG globais idem (src + ReactComponent).
declare module '*.png' {
	const value: number;
	export default value;
}
declare module '*.jpg' {
	const value: number;
	export default value;
}
declare module '*.jpeg' {
	const value: number;
	export default value;
}
declare module '*.gif' {
	const value: number;
	export default value;
}
declare module '*.mp4' {
	const value: number;
	export default value;
}
declare module '*.mp3' {
	const value: number | string;
	export default value;
}
declare module '*.svg' {
	const src: string;
	export default src;
	export const ReactComponent: React.FC<React.SVGProps<SVGSVGElement>>;
}
declare module '*.ttf';
declare module '*.otf';
// side-effect import do nativewind (src/global.css)
declare module '*.css';
