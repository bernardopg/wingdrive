import {useSelectByPatternDialog} from '../../../components/modals/SelectByPatternModal';
import {useExplorer} from '../context';
import {useSelection} from '../SelectionContext';
import {invertSelection} from './selectionPattern';

/** Invert the selection and select by pattern, over the files the view shows. */
export function useSelectionCommands() {
	const {currentFiles} = useExplorer();
	const {selectedFiles, setSelectedFiles} = useSelection();
	const openSelectByPattern = useSelectByPatternDialog();

	return {
		hasFiles: currentFiles.length > 0,
		invert: () => setSelectedFiles(invertSelection(currentFiles, selectedFiles)),
		selectByPattern: () => openSelectByPattern(currentFiles, setSelectedFiles)
	};
}
