import { useLayoutEffect, useRef, type RefObject } from "react";
import { useExplorer } from "../context";

export function useTabScroll(ref: RefObject<HTMLDivElement | null>, ready: number) {
	const {scrollPosition, setScrollPosition} = useExplorer();
	const initial = useRef(scrollPosition);
	const restored = useRef(false);
	useLayoutEffect(() => {
		if (!ready || restored.current || !ref.current) return;
		if (initial.current.left || initial.current.top) {
			ref.current.scrollTo(initial.current.left, initial.current.top);
		}
		restored.current = true;
	}, [ready, ref]);
	useLayoutEffect(() => {
		if (!ready) return;
		const element = ref.current;
		return () => {
			if (element) setScrollPosition({top: element.scrollTop, left: element.scrollLeft});
		};
	}, [ready, ref, setScrollPosition]);
}
