/// <reference types="vite/client" />

// Ambient types for interface code, shared by every app that bundles it.
// apps/tauri and apps/web reference this file instead of copying it.

declare module '*.svg' {
	const src: string;
	export default src;
	export const ReactComponent: React.FC<React.SVGProps<SVGSVGElement>>;
}

declare module '@mkkellogg/gaussian-splats-3d' {
	const GaussianSplats3D: any;
	export default GaussianSplats3D;
	export const Viewer: any;
	export const DropInViewer: any;
	export const SceneFormat: any;
}

declare module 'qrcode' {
	export function toCanvas(canvas: HTMLCanvasElement, text: string, options?: any): Promise<void>;
	export function toDataURL(text: string, options?: any): Promise<string>;
	export function toString(text: string, options?: any): Promise<string>;
}
