import {expect, test} from 'bun:test';
import {useSortPreferencesStore} from './sortPreferences';

test('sort preferences store sets and gets by path', () => {
	useSortPreferencesStore.setState({preferences: {}});
	expect(useSortPreferencesStore.getState().getPreferences('/a')).toBeUndefined();
	useSortPreferencesStore.getState().setPreferences('/a', 'name');
	expect(useSortPreferencesStore.getState().getPreferences('/a')).toBe('name');
	expect(useSortPreferencesStore.getState().preferences['/a']).toBe('name');
});
