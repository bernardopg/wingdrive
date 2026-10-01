import { useCoreQuery } from '../contexts/WingDriveContext';

/**
 * Hook to get all libraries using auto-generated types
 */
export function useLibraries(includeStats = false) {
	return useCoreQuery({
		type: 'libraries.list',
		input: {
			include_stats: includeStats,
		},
	});
}