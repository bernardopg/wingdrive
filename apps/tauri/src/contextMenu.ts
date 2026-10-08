import {
	IconMenuItem,
	Menu,
	MenuItem,
	PredefinedMenuItem,
	Submenu
} from '@tauri-apps/api/menu';
import type { ContextMenuItem } from '@wingdrive/interface';

/**
 * Convert platform-agnostic menu items to Tauri's native Menu API
 */
export async function showNativeContextMenu(
	items: ContextMenuItem[],
	position: { x: number; y: number }
) {
	console.log('[Tauri ContextMenu] Building native menu from items:', items);

	const menuItems = await buildMenuItems(items);
	const menu = await Menu.new({ items: menuItems });

	console.log('[Tauri ContextMenu] Showing menu at position:', position);
	await menu.popup();
}

/**
 * Recursively build Tauri menu items from platform-agnostic definitions
 */
async function buildMenuItems(items: ContextMenuItem[]): Promise<any[]> {
	const menuItems = [];

	for (const item of items) {
		if (item.type === 'separator') {
			// Add separator
			menuItems.push(await PredefinedMenuItem.new({ item: 'Separator' }));
		} else if (item.submenu) {
			// Add submenu
			const subItems = await buildMenuItems(item.submenu);
			const submenu = await Submenu.new({
				text: item.label || 'Submenu',
				items: subItems,
			});
			menuItems.push(submenu);
		} else {
			const options = {
				text: item.label || '',
				enabled: !item.disabled,
				accelerator: item.keybind,
				action: item.onClick,
			};
			const icon = item.iconUrl ? await menuIconBytes(item.iconUrl) : null;
			menuItems.push(
				icon
					? await IconMenuItem.new({ ...options, icon })
					: await MenuItem.new(options)
			);
		}
	}

	return menuItems;
}

/** Native menu icons rasterized from `data:` URLs, kept for later menus. */
const iconCache = new Map<string, Promise<Uint8Array | null>>();

/**
 * Native menus take PNG bytes only, so the webview draws the icon (PNG or
 * SVG) to a canvas and exports it as PNG.
 */
function menuIconBytes(url: string): Promise<Uint8Array | null> {
	let cached = iconCache.get(url);
	if (!cached) {
		cached = rasterize(url).catch(() => null);
		iconCache.set(url, cached);
	}
	return cached;
}

async function rasterize(url: string): Promise<Uint8Array | null> {
	const size = 32;
	const image = new Image(size, size);
	image.src = url;
	await image.decode();
	const canvas = document.createElement('canvas');
	canvas.width = size;
	canvas.height = size;
	canvas.getContext('2d')?.drawImage(image, 0, 0, size, size);
	const blob = await new Promise<Blob | null>((resolve) =>
		canvas.toBlob(resolve, 'image/png')
	);
	return blob ? new Uint8Array(await blob.arrayBuffer()) : null;
}

/**
 * Initialize the context menu handler on the window global
 */
export function initializeContextMenuHandler() {
	window.__WINGDRIVE__ ??= {};
	window.__WINGDRIVE__.showContextMenu = showNativeContextMenu;
	console.log('[Tauri ContextMenu] Handler initialized');
}
