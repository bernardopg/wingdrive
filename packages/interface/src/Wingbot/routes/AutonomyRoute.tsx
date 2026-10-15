import {useTranslation} from '../../i18n';

export function AutonomyRoute() {
	const {t} = useTranslation('common');
	return (
		<div className="border-app-line bg-app-box/90 mx-auto w-full max-w-3xl rounded-[28px] border p-6 text-left shadow-[0_30px_80px_rgba(0,0,0,0.25)] backdrop-blur-xl">
			<h1 className="text-ink text-3xl font-semibold">{t('wingbot.autonomy.title')}</h1>
			<p className="text-ink-dull mt-2 text-sm">
				{t('wingbot.autonomy.description')}
			</p>
		</div>
	);
}
