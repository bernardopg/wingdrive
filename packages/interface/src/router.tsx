import {createBrowserRouter, Navigate, Outlet, useLocation} from 'react-router-dom';
import {JobsScreen} from './components/JobManager';
import {DaemonManager} from './routes/daemon';
import {ExplorerView} from './routes/explorer';
import {RecentsView} from './routes/explorer/views/RecentsView';
import {FavoritesView} from './routes/favorites';
import {FileKindFiles, FileKindsView} from './routes/file-kinds';
import {Overview} from './routes/overview';
import {RedundancyDashboard} from './routes/redundancy';
import {AtRiskFiles} from './routes/redundancy/at-risk';
import {CompareVolumes} from './routes/redundancy/compare';
import {SourcesHome} from './routes/sources';
import {AdaptersScreen} from './routes/sources/Adapters';
import {SourceDetail} from './routes/sources/SourceDetail';
import {TagView} from './routes/tag';
import {ShellLayout} from './ShellLayout';
import {AutonomyRoute} from './Wingbot/routes/AutonomyRoute';
import {ChatRoute} from './Wingbot/routes/ChatRoute';
import {ConversationRoute} from './Wingbot/routes/ConversationRoute';
import {MemoriesRoute} from './Wingbot/routes/MemoriesRoute';
import {ScheduleRoute} from './Wingbot/routes/ScheduleRoute';
import {TasksRoute} from './Wingbot/routes/TasksRoute';
import {legacyWingbotPath} from './Wingbot/legacyPath';
import {WingbotProvider} from './Wingbot/WingbotContext';
import {WingbotLayout} from './Wingbot/WingbotLayout';

/**
 * Wingbot wrapper component that provides the Wingbot context
 */
function LegacyWingbotRedirect() {
	const {pathname, search, hash} = useLocation();
	return <Navigate to={legacyWingbotPath(pathname) + search + hash} replace />;
}

function WingbotRoutes() {
	return (
		<WingbotProvider>
			<Outlet />
		</WingbotProvider>
	);
}

/**
 * Router routes configuration (without router instance)
 */
export const explorerRoutes = [
	{
		path: '/',
		element: <ShellLayout />,
		children: [
			{
				index: true,
				element: <Overview />
			},
			{
				path: 'explorer',
				element: <ExplorerView />
			},
			{
				path: 'favorites',
				element: <FavoritesView />
			},
			{
				path: 'recents',
				element: <RecentsView />
			},
			{
				path: 'file-kinds',
				element: <FileKindsView />
			},
			{
				path: 'file-kinds/:kindName',
				element: <FileKindFiles />
			},
			{
				path: 'tag/:tagId',
				element: <TagView />
			},
			{
				path: 'sources',
				element: <SourcesHome />
			},
			{
				path: 'sources/adapters',
				element: <AdaptersScreen />
			},
			{
				path: 'sources/:sourceId',
				element: <SourceDetail />
			},
			{
				path: 'redundancy',
				children: [
					{
						index: true,
						element: <RedundancyDashboard />
					},
					{
						path: 'at-risk',
						element: <AtRiskFiles />
					},
					{
						path: 'compare',
						element: <CompareVolumes />
					}
				]
			},
			{
				// Pre-BRAND-002 tabs and deep links.
				path: 'spacebot/*',
				element: <LegacyWingbotRedirect />
			},
			{
				path: 'wingbot',
				element: <WingbotRoutes />,
				children: [
					{
						index: true,
						element: <Navigate to="/wingbot/chat" replace />
					},
					{
						element: <WingbotLayout />,
						children: [
							{
								path: 'chat',
								children: [
									{
										index: true,
										element: <ChatRoute />
									},
									{
										path: 'new',
										element: <ChatRoute />
									},
									{
										path: 'conversation/*',
										element: <ConversationRoute />
									}
								]
							},
							{
								path: 'tasks',
								element: <TasksRoute />
							},
							{
								path: 'memories',
								element: <MemoriesRoute />
							},
							{
								path: 'autonomy',
								element: <AutonomyRoute />
							},
							{
								path: 'schedule',
								element: <ScheduleRoute />
							}
						]
					}
				]
			},
			{
				path: 'jobs',
				element: <JobsScreen />
			},
			{
				path: 'daemon',
				element: <DaemonManager />
			}
		]
	}
];

/**
 * Router for the main Explorer interface
 */
export function createExplorerRouter(): ReturnType<typeof createBrowserRouter> {
	return createBrowserRouter(explorerRoutes);
}
