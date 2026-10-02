import {RouterProvider} from 'react-router-dom';
import {useTabManager} from './useTabManager';

export function TabView() {
	const {router, activeTabId} = useTabManager();
	return <RouterProvider key={activeTabId} router={router} />;
}
