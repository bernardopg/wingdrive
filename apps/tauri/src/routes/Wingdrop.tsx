import { Wingdrop } from '@wingdrive/interface';

export function WingdropWindow() {
	return (
		<div className="h-screen w-screen bg-sidebar">
			<Wingdrop onClose={() => window.close()} />
		</div>
	);
}
