/**
 * Raw color values for NativeWind (mobile app)
 * Note: This file is maintained for mobile app compatibility with Tailwind v3.
 * The main spaceui tokens are now CSS-only for Tailwind v4.
 */

const rawColors = {
	black: '0, 0%, 0%',
	white: '0, 0%, 100%',
	accent: {
		DEFAULT: '208, 100%, 57%',
		faint: '208, 100%, 64%',
		deep: '208, 100%, 47%',
	},
	ink: {
		DEFAULT: '235, 35%, 92%',
		dull: '235, 10%, 70%',
		faint: '235, 10%, 55%',
	},
	sidebar: {
		DEFAULT: '235, 15%, 7%',
		box: '235, 15%, 16%',
		line: '235, 15%, 23%',
		ink: '235, 15%, 92%',
		inkDull: '235, 10%, 70%',
		inkFaint: '235, 10%, 55%',
		divider: '235, 15%, 17%',
		button: '235, 15%, 18%',
		selected: '235, 15%, 24%',
		shade: '235, 15%, 23%',
	},
	app: {
		DEFAULT: '235, 15%, 13%',
		box: '235, 15%, 18%',
		darkBox: '235, 15%, 15%',
		darkerBox: '235, 16%, 11%',
		lightBox: '235, 15%, 34%',
		overlay: '235, 15%, 17%',
		input: '235, 15%, 20%',
		focus: '235, 15%, 10%',
		line: '235, 15%, 23%',
		divider: '235, 15%, 5%',
		button: '235, 15%, 23%',
		hover: '235, 15%, 21%',
		selected: '235, 15%, 24%',
		selectedItem: '235, 15%, 18%',
		active: '235, 15%, 30%',
		shade: '235, 15%, 0%',
		frame: '235, 15%, 25%',
		slider: '235, 15%, 20%',
		explorerScrollbar: '235, 20%, 25%',
	},
	menu: {
		DEFAULT: '235, 15%, 10%',
		line: '235, 15%, 14%',
		ink: '235, 25%, 92%',
		faint: '235, 5%, 80%',
		hover: '235, 15%, 30%',
		selected: '235, 5%, 30%',
		shade: '235, 5%, 0%',
	},
	status: {
		success: '142, 76%, 36%',
		warning: '38, 92%, 50%',
		error: '0, 84%, 60%',
		info: '208, 100%, 57%',
	},
};

module.exports = rawColors;
