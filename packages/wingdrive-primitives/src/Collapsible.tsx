import * as CollapsiblePrimitive from '@radix-ui/react-collapsible';
import { clsx } from 'clsx';
import {
  forwardRef,
  type ComponentProps,
  type ComponentPropsWithoutRef,
  type ElementRef,
  type FC,
  type ForwardRefExoticComponent,
  type RefAttributes,
} from 'react';

const Collapsible: FC<
  ComponentProps<typeof CollapsiblePrimitive.Root>
> = CollapsiblePrimitive.Root;

const CollapsibleTrigger = forwardRef<
  ElementRef<typeof CollapsiblePrimitive.CollapsibleTrigger>,
  ComponentPropsWithoutRef<typeof CollapsiblePrimitive.CollapsibleTrigger>
>(({ className, children, ...props }, ref) => (
  <CollapsiblePrimitive.CollapsibleTrigger
    ref={ref}
    className={clsx(
      'flex items-center justify-between w-full',
      'hover:bg-app-hover rounded-md transition-colors',
      'focus:outline-none focus:ring-2 focus:ring-accent',
      className
    )}
    {...props}
  >
    {children}
  </CollapsiblePrimitive.CollapsibleTrigger>
));

CollapsibleTrigger.displayName = CollapsiblePrimitive.CollapsibleTrigger.displayName;

const CollapsibleTriggerExp: ForwardRefExoticComponent<
  ComponentPropsWithoutRef<
    typeof CollapsiblePrimitive.CollapsibleTrigger
  > &
    RefAttributes<
      ElementRef<typeof CollapsiblePrimitive.CollapsibleTrigger>
    >
> = CollapsibleTrigger;

const CollapsibleContent = forwardRef<
  ElementRef<typeof CollapsiblePrimitive.CollapsibleContent>,
  ComponentPropsWithoutRef<typeof CollapsiblePrimitive.CollapsibleContent>
>(({ className, children, ...props }, ref) => (
  <CollapsiblePrimitive.CollapsibleContent
    ref={ref}
    className={clsx(
      'overflow-hidden',
      'data-[state=closed]:animate-collapsible-up data-[state=open]:animate-collapsible-down',
      className
    )}
    {...props}
  >
    {children}
  </CollapsiblePrimitive.CollapsibleContent>
));

CollapsibleContent.displayName = CollapsiblePrimitive.CollapsibleContent.displayName;

const CollapsibleContentExp: ForwardRefExoticComponent<
  ComponentPropsWithoutRef<
    typeof CollapsiblePrimitive.CollapsibleContent
  > &
    RefAttributes<
      ElementRef<typeof CollapsiblePrimitive.CollapsibleContent>
    >
> = CollapsibleContent;

export {
  Collapsible,
  CollapsibleTriggerExp as CollapsibleTrigger,
  CollapsibleContentExp as CollapsibleContent,
};
