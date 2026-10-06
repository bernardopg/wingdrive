import { Navigate, Outlet, type RouteObject } from 'react-router-dom';
import { WingbotLayout } from './WingbotLayout';
import { ChatRoute } from './routes/ChatRoute';
import { ConversationRoute } from './routes/ConversationRoute';
import { TasksRoute } from './routes/TasksRoute';

/**
 * Wingbot nested route configuration
 * These routes are mounted under /wingbot in the main router
 */
export const wingbotRoutes: RouteObject[] = [
	{
		path: 'wingbot',
		element: <WingbotLayout />,
		children: [
			{
				index: true,
				element: <Navigate to="/wingbot/chat" replace />,
			},
			{
				path: 'chat',
				children: [
					{
						index: true,
						element: <ChatRoute />,
					},
					{
						path: 'new',
						element: <ChatRoute />,
					},
					{
						path: 'conversation/:conversationId',
						element: <ConversationRoute />,
					},
				],
			},
			{
				path: 'tasks',
				element: <TasksRoute />,
			},
		],
	},
];

/**
 * Wingbot Router Provider component that wraps the Outlet
 * Used when Wingbot routes are mounted within the main router
 */
export function WingbotRouter() {
	return <Outlet />;
}
