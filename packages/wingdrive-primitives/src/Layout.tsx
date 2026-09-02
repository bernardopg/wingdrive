import type {
	ForwardRefExoticComponent,
	HTMLAttributes,
	RefAttributes,
} from "react";

import {tw} from "./utils";

const CardImpl = tw.div`flex px-4 py-2 text-sm border rounded-md shadow-sm border-app-line bg-app-box`;

export const Card = CardImpl as unknown as ForwardRefExoticComponent<
	HTMLAttributes<HTMLDivElement> & RefAttributes<HTMLDivElement>
>;

const GridLayoutImpl = tw.div`grid grid-cols-2 gap-3 lg:grid-cols-3`;

export const GridLayout =
	GridLayoutImpl as unknown as ForwardRefExoticComponent<
		HTMLAttributes<HTMLDivElement> & RefAttributes<HTMLDivElement>
	>;
