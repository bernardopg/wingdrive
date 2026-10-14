/** Keep Linux WebKit's compositor available for folder navigation. */
export function configureRendering(
	root: Pick<Element, 'toggleAttribute'>,
	platform: string,
	windowLabel: string
): void {
	root.toggleAttribute(
		'data-reduced-transparency',
		platform.startsWith('Linux') && windowLabel === 'main'
	);
}
