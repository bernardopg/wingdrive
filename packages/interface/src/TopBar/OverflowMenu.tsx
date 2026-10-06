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
					className="min-w-[180px] rounded-lg bg-app border border-app-line shadow-2xl py-1 z-[70]"
					sideOffset={8}
					align="start"
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

						return (
							<DropdownMenu.Sub key={item.id}>
								<DropdownMenu.SubTrigger className="px-3 py-2 text-sm text-menu-ink hover:bg-app-hover/50 transition-colors flex items-center justify-between outline-none cursor-pointer">
									<span>{item.label}</span>
									<span className="text-menu-faint text-xs">▶</span>
								</DropdownMenu.SubTrigger>
								<DropdownMenu.Portal>
									{/* Above the TopBar (z-60) and the inspector, flush with the
									    trigger: a gap closed the submenu before the pointer
									    reached it when it opened to the left. */}
									<DropdownMenu.SubContent
										className="z-[70]"
										sideOffset={0}
									>
										{item.submenuContent || item.element}
									</DropdownMenu.SubContent>
								</DropdownMenu.Portal>
							</DropdownMenu.Sub>
						);
					})}
				</DropdownMenu.Content>
			</DropdownMenu.Portal>
		</DropdownMenu.Root>
	);
}