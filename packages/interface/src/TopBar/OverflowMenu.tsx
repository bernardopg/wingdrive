import { useState } from "react";
import { DotsThree } from "@phosphor-icons/react";
import * as DropdownMenu from "@radix-ui/react-dropdown-menu";
import { CircleButton } from "@wingdrive/primitives";
import { TopBarItem, useTopBar } from "./Context";

interface OverflowButtonProps {
	items: TopBarItem[];
}

export function OverflowButton({ items }: OverflowButtonProps) {
	const [isOpen, setIsOpen] = useState(false);
	// The overflow list holds snapshots taken at layout time; read element and
	// submenu content from the live registry so open menus reflect new state.
	const { items: liveItems } = useTopBar();

	if (items.length === 0) return null;

	return (
		<DropdownMenu.Root open={isOpen} onOpenChange={setIsOpen}>
			<DropdownMenu.Trigger asChild>
				<CircleButton
					icon={DotsThree}
					active={isOpen}
				/>
			</DropdownMenu.Trigger>

			<DropdownMenu.Portal>
				<DropdownMenu.Content
					className="min-w-[180px] max-h-[var(--radix-dropdown-menu-content-available-height)] overflow-y-auto rounded-lg bg-app border border-app-line shadow-2xl py-1 z-[70]"
					sideOffset={8}
					align="end"
					collisionPadding={8}
				>
					{items.map((snapshot) => {
						const item = liveItems.get(snapshot.id) ?? snapshot;
						const isSimpleAction = !!item.onClick;

						if (isSimpleAction) {
							return (
								<DropdownMenu.Item
									key={item.id}
									onClick={() => item.onClick?.()}
									className="px-3 py-2 text-sm text-menu-ink hover:bg-app-hover/50 transition-colors outline-none cursor-pointer"
								>
									{item.label}
								</DropdownMenu.Item>
							);
						}

						// Panels render inline under their label instead of as a flyout.
						// A flyout opened to the left of this right-aligned menu, and the
						// pointer left its safe zone on the way over, closing the menu
						// before a choice could be clicked.
						return (
							<DropdownMenu.Group key={item.id}>
								<DropdownMenu.Label className="px-3 pt-2 pb-1 text-xs font-medium text-menu-faint">
									{item.label}
								</DropdownMenu.Label>
								<div className="px-1 pb-1">
									{item.submenuContent || item.element}
								</div>
							</DropdownMenu.Group>
						);
					})}
				</DropdownMenu.Content>
			</DropdownMenu.Portal>
		</DropdownMenu.Root>
	);
}