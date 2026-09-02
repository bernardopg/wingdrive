import type {
	ForwardRefExoticComponent,
	HTMLAttributes,
	RefAttributes,
} from "react";

import { tw } from "./utils";

const CategoryHeadingImpl = tw.h3`text-xs font-semibold text-ink-dull`;

export const CategoryHeading =
	CategoryHeadingImpl as unknown as ForwardRefExoticComponent<
		HTMLAttributes<HTMLHeadingElement> &
			RefAttributes<HTMLHeadingElement>
	>;

const ScreenHeadingImpl = tw.h3`text-xl font-bold`;

export const ScreenHeading =
	ScreenHeadingImpl as unknown as ForwardRefExoticComponent<
		HTMLAttributes<HTMLHeadingElement> &
			RefAttributes<HTMLHeadingElement>
	>;
