import { splitSnippet } from "./sourceSearch";

/** A hit's snippet with the matched terms highlighted. */
export function SnippetText({ snippet }: { snippet: string }) {
	return (
		<>
			{splitSnippet(snippet).map((part, index) =>
				part.match ? (
					<mark key={index} className="bg-accent/25 text-ink rounded-sm px-0.5">
						{part.text}
					</mark>
				) : (
					<span key={index}>{part.text.replace(/[\r\n\t]+/g, " ")}</span>
				),
			)}
		</>
	);
}
