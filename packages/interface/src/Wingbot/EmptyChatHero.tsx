import {usePlatform} from '../contexts/PlatformContext';
import {useTranslation} from '../i18n';

export function EmptyChatHero() {
	const {t} = useTranslation('common');
	const platform = usePlatform();
	return (
		<div className="mb-6 text-left">
			<h1 className="text-ink text-[2.65rem] font-semibold tracking-tight">
				{t('wingbot.emptyHero.title')}
			</h1>
			<p className="text-ink-dull mt-2 text-sm">
				{t('wingbot.emptyHero.description')} {''}
				<a
					href="https://github.com/bernardopg/wingdrive/tree/main/docs/core/design"
					target="_blank"
					rel="noreferrer"
					onClick={(event) => {
						event.preventDefault();
						platform.openLink(event.currentTarget.href);
					}}
					className="text-ink-dull hover:text-ink underline underline-offset-4 transition-colors"
				>
					{t('wingbot.emptyHero.readDocs')}
				</a>
			</p>
		</div>
	);
}
