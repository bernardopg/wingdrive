import * as react from 'react';
import react__default, { ComponentProps, ReactNode, ReactElement, PropsWithChildren, ContextType, CSSProperties, FunctionComponent, ForwardRefExoticComponent, HTMLAttributes, RefAttributes, FC, ComponentPropsWithoutRef, ElementRef, Ref } from 'react';
import * as class_variance_authority_types from 'class-variance-authority/types';
import { VariantProps } from 'class-variance-authority';
import { Icon, IconProps } from '@phosphor-icons/react';
import * as CheckboxPrimitive from '@radix-ui/react-checkbox';
import * as SwitchPrimitive from '@radix-ui/react-switch';
import * as SliderPrimitive from '@radix-ui/react-slider';
import * as RadioGroupPrimitive from '@radix-ui/react-radio-group';
import * as RDialog from '@radix-ui/react-dialog';
import { FieldValues, UseFormReturn, UseFormHandleSubmit } from 'react-hook-form';
import * as PopoverPrimitive from '@radix-ui/react-popover';
import * as TooltipPrimitive from '@radix-ui/react-tooltip';
import * as TabsPrimitive from '@radix-ui/react-tabs';
import * as DropdownMenuPrimitive from '@radix-ui/react-dropdown-menu';
import * as RadixCM from '@radix-ui/react-context-menu';
import * as RS from '@radix-ui/react-select';
import { toast as toast$1 } from 'sonner';
export { Toaster } from 'sonner';
import { UseResizableProps, Resizable as Resizable$1 } from 'react-resizable-layout';
import * as CollapsiblePrimitive from '@radix-ui/react-collapsible';
import { VirtualizerOptions, Virtualizer } from '@tanstack/react-virtual';
export { z } from 'zod';

type ButtonBaseProps = VariantProps<typeof buttonStyles>;
type ButtonProps = ButtonBaseProps & React.ButtonHTMLAttributes<HTMLButtonElement> & {
    href?: undefined;
    /** Alias for `disabled` */
    loading?: boolean;
};
type LinkButtonProps = ButtonBaseProps & React.AnchorHTMLAttributes<HTMLAnchorElement> & {
    href?: string;
    /** Alias for `disabled` (applies disabled styling via className) */
    loading?: boolean;
};
declare const buttonStyles: (props?: {
    size?: "icon" | "lg" | "md" | "sm" | "xs";
    variant?: "default" | "subtle" | "outline" | "dotted" | "gray" | "accent" | "colored" | "bare";
    rounding?: "none" | "left" | "right" | "both" | "full";
} & class_variance_authority_types.ClassProp) => string;
declare const Button$1: react.ForwardRefExoticComponent<(ButtonProps | LinkButtonProps) & react.RefAttributes<HTMLButtonElement | HTMLAnchorElement>>;

interface InputBaseProps extends VariantProps<typeof inputStyles> {
    icon?: Icon | React.ReactNode;
    iconPosition?: "left" | "right";
    inputElementClassName?: string;
    right?: React.ReactNode;
}
type InputProps = InputBaseProps & Omit<React.ComponentProps<"input">, "size">;
type TextareaProps = InputBaseProps & React.ComponentProps<"textarea">;
declare const inputSizes: {
    xs: string;
    sm: string;
    md: string;
    lg: string;
    xl: string;
};
declare const inputStyles: (props?: {
    variant?: "default" | "transparent";
    error?: boolean;
    size?: "lg" | "md" | "sm" | "xs" | "xl";
} & class_variance_authority_types.ClassProp) => string;
declare const Input: react.ForwardRefExoticComponent<Omit<InputProps, "ref"> & react.RefAttributes<HTMLInputElement>>;
/** @deprecated Use `SearchBar` instead. */
declare const SearchInput: react.ForwardRefExoticComponent<Omit<InputProps, "ref"> & react.RefAttributes<HTMLInputElement>>;
declare const TextArea: react.ForwardRefExoticComponent<Omit<TextareaProps, "ref"> & react.RefAttributes<HTMLTextAreaElement>>;
interface LabelProps extends Omit<React.ComponentProps<"label">, "htmlFor"> {
    slug?: string;
}
declare function Label$1({ slug, children, className, ...props }: LabelProps): react.JSX.Element;
interface PasswordInputProps extends InputProps {
    buttonClassnames?: string;
}
declare const PasswordInput: react.ForwardRefExoticComponent<Omit<PasswordInputProps, "ref"> & react.RefAttributes<HTMLInputElement>>;

declare const styles: (props?: {} & class_variance_authority_types.ClassProp) => string;
interface CheckBoxProps extends ComponentProps<"input">, VariantProps<typeof styles> {
}
declare const CheckBox: react.ForwardRefExoticComponent<Omit<CheckBoxProps, "ref"> & react.RefAttributes<HTMLInputElement>>;
interface RadixCheckboxProps extends ComponentProps<typeof CheckboxPrimitive.Root> {
    label?: string;
    labelClassName?: string;
}
declare const RadixCheckbox: ({ className, labelClassName, ...props }: RadixCheckboxProps) => react.JSX.Element;
declare const CheckboxRoot: react.ForwardRefExoticComponent<CheckboxPrimitive.CheckboxProps & react.RefAttributes<HTMLButtonElement>>;
declare const CheckboxIndicator: react.ForwardRefExoticComponent<CheckboxPrimitive.CheckboxIndicatorProps & react.RefAttributes<HTMLSpanElement>>;

interface SwitchProps extends VariantProps<typeof switchStyles>, SwitchPrimitive.SwitchProps {
    thumbClassName?: string;
}
declare const switchStyles: (props?: {
    size?: "lg" | "md" | "sm";
} & class_variance_authority_types.ClassProp) => string;
declare const Switch: react.ForwardRefExoticComponent<SwitchProps & react.RefAttributes<HTMLButtonElement>>;

declare const Slider: (props: SliderPrimitive.SliderProps) => react.JSX.Element;

type RootProps = RadioGroupPrimitive.RadioGroupProps;
declare const Root$4: react.ForwardRefExoticComponent<RadioGroupPrimitive.RadioGroupProps & react.RefAttributes<HTMLDivElement>>;
type ItemProps = RadioGroupPrimitive.RadioGroupItemProps;
declare const Item$2: ({ children, ...props }: ItemProps) => react.JSX.Element;

interface FormProps<T extends FieldValues> extends Omit<ComponentProps<"form">, "onSubmit"> {
    form: UseFormReturn<T>;
    disabled?: boolean;
    onSubmit?: ReturnType<UseFormHandleSubmit<T>>;
}
declare const Form: <T extends FieldValues>({ form, disabled, onSubmit, children, ...props }: FormProps<T>) => react.JSX.Element;
declare const errorStyles: (props?: {
    variant?: "default" | "none" | "large";
} & class_variance_authority_types.ClassProp) => string;
interface ErrorMessageProps extends VariantProps<typeof errorStyles> {
    name: string;
    className: string;
}
declare const ErrorMessage: ({ name, variant, className, }: ErrorMessageProps) => react.JSX.Element;

interface DialogState {
    open: boolean;
}
interface DialogOptions {
    onSubmit?(): void;
}
interface UseDialogProps extends DialogOptions {
    id: number;
}
declare class DialogManager {
    private idGenerator;
    private listeners;
    private states;
    private components;
    create(dialog: (props: UseDialogProps) => ReactElement, options?: DialogOptions): Promise<void>;
    getId(): number;
    getState(id: number): DialogState | undefined;
    setState(id: number, state: Partial<DialogState>): void;
    subscribe(id: number, listener: (state: DialogState) => void): () => void;
    private globalListeners;
    subscribeGlobal(listener: () => void): () => void;
    private notifyGlobalListeners;
    getComponents(): [number, react.FC<{}>][];
    isAnyDialogOpen(): boolean;
    remove(id: number): void;
}
declare const dialogManager: DialogManager;
declare function useDialog(props: UseDialogProps): {
    state: DialogState;
    id: number;
    onSubmit?(): void;
};
declare function Dialogs(): react.JSX.Element;
interface DialogProps<S extends FieldValues> extends RDialog.DialogProps, Omit<FormProps<S>, "onSubmit"> {
    title?: string;
    dialog: ReturnType<typeof useDialog>;
    loading?: boolean;
    trigger?: ReactNode;
    ctaLabel?: string;
    ctaSecondLabel?: string;
    onSubmit?: ReturnType<UseFormHandleSubmit<S>>;
    onSubmitSecond?: ReturnType<UseFormHandleSubmit<S>>;
    children?: ReactNode;
    ctaDanger?: boolean;
    cancelDanger?: boolean;
    closeLabel?: string;
    cancelLabel?: string;
    cancelBtn?: boolean;
    description?: ReactNode;
    onCancelled?: boolean | (() => void);
    submitDisabled?: boolean;
    transformOrigin?: string;
    buttonsSideContent?: ReactNode;
    invertButtonFocus?: boolean;
    errorMessageException?: string;
    formClassName?: string;
    icon?: ReactNode;
    hideButtons?: boolean;
    hideHeader?: boolean;
    ignoreClickOutside?: boolean;
}
declare function Dialog<S extends FieldValues>({ form, dialog, onSubmit, onSubmitSecond, onCancelled, invertButtonFocus, ...props }: DialogProps<S>): react.JSX.Element;
declare const DialogRoot: react.FC<RDialog.DialogProps>;
declare const DialogTrigger: react.ForwardRefExoticComponent<RDialog.DialogTriggerProps & react.RefAttributes<HTMLButtonElement>>;
declare const DialogPortal: react.FC<RDialog.DialogPortalProps>;
declare const DialogClose: react.ForwardRefExoticComponent<RDialog.DialogCloseProps & react.RefAttributes<HTMLButtonElement>>;
declare const DialogOverlay: {
    ({ className, ...props }: RDialog.DialogOverlayProps): react.JSX.Element;
    displayName: string;
};
declare const DialogContent: {
    ({ className, children, ...props }: RDialog.DialogContentProps): react.JSX.Element;
    displayName: string;
};
declare const DialogHeader: {
    ({ className, ...props }: React.HTMLAttributes<HTMLDivElement>): react.JSX.Element;
    displayName: string;
};
declare const DialogFooter: {
    ({ className, ...props }: React.HTMLAttributes<HTMLDivElement>): react.JSX.Element;
    displayName: string;
};
declare const DialogTitle: {
    ({ className, ...props }: RDialog.DialogTitleProps): react.JSX.Element;
    displayName: string;
};
declare const DialogDescription: {
    ({ className, ...props }: RDialog.DialogDescriptionProps): react.JSX.Element;
    displayName: string;
};

/** Convenience hook for controlled popover state */
declare function usePopover(): {
    open: boolean;
    setOpen: react.Dispatch<react.SetStateAction<boolean>>;
};
declare const Root$3: react.FC<PopoverPrimitive.PopoverProps>;
declare const Trigger$2: react.ForwardRefExoticComponent<PopoverPrimitive.PopoverTriggerProps & react.RefAttributes<HTMLButtonElement>>;
declare const Anchor: react.ForwardRefExoticComponent<PopoverPrimitive.PopoverAnchorProps & react.RefAttributes<HTMLDivElement>>;
declare const Close: react.ForwardRefExoticComponent<PopoverPrimitive.PopoverCloseProps & react.RefAttributes<HTMLButtonElement>>;
declare const Portal$1: react.FC<PopoverPrimitive.PopoverPortalProps>;
declare const Content$2: react.ForwardRefExoticComponent<Omit<PopoverPrimitive.PopoverContentProps & react.RefAttributes<HTMLDivElement>, "ref"> & react.RefAttributes<HTMLDivElement>>;
declare const Popover: {
    Root: react.FC<PopoverPrimitive.PopoverProps>;
    Trigger: react.ForwardRefExoticComponent<PopoverPrimitive.PopoverTriggerProps & react.RefAttributes<HTMLButtonElement>>;
    Content: react.ForwardRefExoticComponent<Omit<PopoverPrimitive.PopoverContentProps & react.RefAttributes<HTMLDivElement>, "ref"> & react.RefAttributes<HTMLDivElement>>;
    Anchor: react.ForwardRefExoticComponent<PopoverPrimitive.PopoverAnchorProps & react.RefAttributes<HTMLDivElement>>;
    Close: react.ForwardRefExoticComponent<PopoverPrimitive.PopoverCloseProps & react.RefAttributes<HTMLButtonElement>>;
    Portal: react.FC<PopoverPrimitive.PopoverPortalProps>;
};

declare const Kbd: ({ children, className, }: {
    children: ReactNode;
    className?: string;
}) => react.JSX.Element;
interface TooltipProps extends PropsWithChildren, Pick<TooltipPrimitive.TooltipProps, "disableHoverableContent">, Pick<TooltipPrimitive.TooltipContentProps, "alignOffset" | "sideOffset" | "align"> {
    label: ReactNode;
    position?: "top" | "right" | "bottom" | "left";
    className?: string;
    tooltipClassName?: string;
    labelClassName?: string;
    asChild?: boolean;
    keybinds?: Array<string>;
}
declare const Tooltip: ({ position, ...props }: TooltipProps) => react.JSX.Element;
declare const TooltipProvider: react.FC<TooltipPrimitive.TooltipProviderProps>;
declare const TooltipRoot: react.FC<TooltipPrimitive.TooltipProps>;
declare const TooltipTrigger: react.ForwardRefExoticComponent<TooltipPrimitive.TooltipTriggerProps & react.RefAttributes<HTMLButtonElement>>;
declare const TooltipPortal: react.FC<TooltipPrimitive.TooltipPortalProps>;
declare const TooltipContent: {
    ({ className, sideOffset, ...props }: TooltipPrimitive.TooltipContentProps): react.JSX.Element;
    displayName: string;
};

declare const Root$2: ({ className, ...props }: TabsPrimitive.TabsProps) => react.JSX.Element;
declare const Content$1: ({ className, ...props }: TabsPrimitive.TabsContentProps) => react.JSX.Element;
declare const List: ({ className, ...props }: TabsPrimitive.TabsListProps) => react.JSX.Element;
declare const Trigger$1: ({ className, ...props }: TabsPrimitive.TabsTriggerProps) => react.JSX.Element;

declare const Root$1: react.FC<DropdownMenuPrimitive.DropdownMenuProps>;
declare const Trigger: react.ForwardRefExoticComponent<DropdownMenuPrimitive.DropdownMenuTriggerProps & react.RefAttributes<HTMLButtonElement>>;
declare const Group: react.ForwardRefExoticComponent<DropdownMenuPrimitive.DropdownMenuGroupProps & react.RefAttributes<HTMLDivElement>>;
declare const Portal: react.FC<DropdownMenuPrimitive.DropdownMenuPortalProps>;
declare const Sub: react.FC<DropdownMenuPrimitive.DropdownMenuSubProps>;
declare const RadioGroup: react.ForwardRefExoticComponent<DropdownMenuPrimitive.DropdownMenuRadioGroupProps & react.RefAttributes<HTMLDivElement>>;
declare const Content: react.ForwardRefExoticComponent<Omit<DropdownMenuPrimitive.DropdownMenuContentProps & react.RefAttributes<HTMLDivElement>, "ref"> & react.RefAttributes<HTMLDivElement>>;
declare const Item$1: react.ForwardRefExoticComponent<Omit<DropdownMenuPrimitive.DropdownMenuItemProps & react.RefAttributes<HTMLDivElement>, "ref"> & {
    inset?: boolean;
} & react.RefAttributes<HTMLDivElement>>;
declare const CheckboxItem: react.ForwardRefExoticComponent<Omit<DropdownMenuPrimitive.DropdownMenuCheckboxItemProps & react.RefAttributes<HTMLDivElement>, "ref"> & react.RefAttributes<HTMLDivElement>>;
declare const RadioItem: react.ForwardRefExoticComponent<Omit<DropdownMenuPrimitive.DropdownMenuRadioItemProps & react.RefAttributes<HTMLDivElement>, "ref"> & react.RefAttributes<HTMLDivElement>>;
declare const Label: react.ForwardRefExoticComponent<Omit<DropdownMenuPrimitive.DropdownMenuLabelProps & react.RefAttributes<HTMLDivElement>, "ref"> & {
    inset?: boolean;
} & react.RefAttributes<HTMLDivElement>>;
declare const Separator$1: react.ForwardRefExoticComponent<Omit<DropdownMenuPrimitive.DropdownMenuSeparatorProps & react.RefAttributes<HTMLDivElement>, "ref"> & react.RefAttributes<HTMLDivElement>>;
declare const SubTrigger: react.ForwardRefExoticComponent<Omit<DropdownMenuPrimitive.DropdownMenuSubTriggerProps & react.RefAttributes<HTMLDivElement>, "ref"> & {
    inset?: boolean;
} & react.RefAttributes<HTMLDivElement>>;
declare const SubContent: react.ForwardRefExoticComponent<Omit<DropdownMenuPrimitive.DropdownMenuSubContentProps & react.RefAttributes<HTMLDivElement>, "ref"> & react.RefAttributes<HTMLDivElement>>;
declare const DropdownMenu: {
    Root: react.FC<DropdownMenuPrimitive.DropdownMenuProps>;
    Trigger: react.ForwardRefExoticComponent<DropdownMenuPrimitive.DropdownMenuTriggerProps & react.RefAttributes<HTMLButtonElement>>;
    Content: react.ForwardRefExoticComponent<Omit<DropdownMenuPrimitive.DropdownMenuContentProps & react.RefAttributes<HTMLDivElement>, "ref"> & react.RefAttributes<HTMLDivElement>>;
    Item: react.ForwardRefExoticComponent<Omit<DropdownMenuPrimitive.DropdownMenuItemProps & react.RefAttributes<HTMLDivElement>, "ref"> & {
        inset?: boolean;
    } & react.RefAttributes<HTMLDivElement>>;
    CheckboxItem: react.ForwardRefExoticComponent<Omit<DropdownMenuPrimitive.DropdownMenuCheckboxItemProps & react.RefAttributes<HTMLDivElement>, "ref"> & react.RefAttributes<HTMLDivElement>>;
    RadioItem: react.ForwardRefExoticComponent<Omit<DropdownMenuPrimitive.DropdownMenuRadioItemProps & react.RefAttributes<HTMLDivElement>, "ref"> & react.RefAttributes<HTMLDivElement>>;
    Label: react.ForwardRefExoticComponent<Omit<DropdownMenuPrimitive.DropdownMenuLabelProps & react.RefAttributes<HTMLDivElement>, "ref"> & {
        inset?: boolean;
    } & react.RefAttributes<HTMLDivElement>>;
    Separator: react.ForwardRefExoticComponent<Omit<DropdownMenuPrimitive.DropdownMenuSeparatorProps & react.RefAttributes<HTMLDivElement>, "ref"> & react.RefAttributes<HTMLDivElement>>;
    Group: react.ForwardRefExoticComponent<DropdownMenuPrimitive.DropdownMenuGroupProps & react.RefAttributes<HTMLDivElement>>;
    Portal: react.FC<DropdownMenuPrimitive.DropdownMenuPortalProps>;
    Sub: react.FC<DropdownMenuPrimitive.DropdownMenuSubProps>;
    SubTrigger: react.ForwardRefExoticComponent<Omit<DropdownMenuPrimitive.DropdownMenuSubTriggerProps & react.RefAttributes<HTMLDivElement>, "ref"> & {
        inset?: boolean;
    } & react.RefAttributes<HTMLDivElement>>;
    SubContent: react.ForwardRefExoticComponent<Omit<DropdownMenuPrimitive.DropdownMenuSubContentProps & react.RefAttributes<HTMLDivElement>, "ref"> & react.RefAttributes<HTMLDivElement>>;
    RadioGroup: react.ForwardRefExoticComponent<DropdownMenuPrimitive.DropdownMenuRadioGroupProps & react.RefAttributes<HTMLDivElement>>;
};

interface ContextMenuProps extends RadixCM.ContextMenuContentProps {
    trigger: React.ReactNode;
    onOpenChange?: (open: boolean) => void;
    disabled?: boolean;
}
declare const contextMenuClassNames: string;
declare const ContextMenuContext: react.Context<boolean>;
declare const useContextMenuContext: <T extends boolean>({ suspense, }?: {
    suspense?: T;
}) => T extends true ? NonNullable<ContextType<typeof ContextMenuContext>> : NonNullable<ContextType<typeof ContextMenuContext>> | undefined;
declare const contextMenuSeparatorClassNames = "border-b-menu-line mx-1 my-0.5 border-b";
declare const contextMenuItemStyles: (props?: {
    variant?: "default" | "dull" | "danger";
} & class_variance_authority_types.ClassProp) => string;
interface ContextMenuItemProps extends RadixCM.ContextMenuItemProps, VariantProps<typeof contextMenuItemStyles>, Pick<ContextMenuInnerItemProps, "label" | "keybind" | "icon" | "iconProps"> {
}
declare const contextMenuItemClassNames = "group py-0.5 outline-none px-1";
interface ContextMenuCheckboxItemProps extends RadixCM.ContextMenuCheckboxItemProps, VariantProps<typeof contextMenuItemStyles>, Pick<ContextMenuInnerItemProps, "label" | "keybind"> {
}
interface ContextMenuInnerItemProps {
    icon?: Icon;
    iconProps?: IconProps;
    label?: string;
    keybind?: string;
    rightArrow?: boolean;
}
declare const ContextMenuDivItem: ({ variant, children, className, ...props }: ContextMenuInnerItemProps & VariantProps<typeof contextMenuItemStyles> & PropsWithChildren<{
    className?: string;
}>) => react.JSX.Element;
declare const ContextMenu: {
    Root: ({ trigger, children, className, onOpenChange, disabled, ...props }: ContextMenuProps) => react.JSX.Element;
    Item: ({ icon, label, children, keybind, variant, iconProps, onClick, ...props }: ContextMenuItemProps) => react.JSX.Element;
    CheckboxItem: ({ variant, className, label, keybind, children, ...props }: ContextMenuCheckboxItemProps) => react.JSX.Element;
    Separator: (props: {
        className?: string;
    }) => react.JSX.Element;
    SubMenu: ({ label, icon, className, ...props }: RadixCM.ContextMenuSubContentProps & ContextMenuItemProps) => react.JSX.Element;
};

declare const Section: ({ className, ...props }: React.HTMLAttributes<HTMLDivElement>) => react.JSX.Element;
declare const itemStyles: (props?: {
    selected?: boolean | "undefined";
    active?: boolean;
} & class_variance_authority_types.ClassProp) => string;
type DropdownItemProps = PropsWithChildren<{
    to?: string;
    className?: string;
    icon?: any;
    iconClassName?: string;
    onClick?: () => void;
}> & VariantProps<typeof itemStyles>;
declare const Item: ({ to, className, icon: Icon, children, ...props }: DropdownItemProps) => react.JSX.Element;
declare const Button: react.ForwardRefExoticComponent<ButtonBaseProps & react.ButtonHTMLAttributes<HTMLButtonElement> & {
    href?: undefined;
    loading?: boolean;
} & react.RefAttributes<HTMLButtonElement>>;
interface DropdownRootProps {
    button: React.ReactNode;
    className?: string;
    itemsClassName?: string;
    align?: "left" | "right";
}
declare const Root: (props: PropsWithChildren<DropdownRootProps>) => react.JSX.Element;
declare const Separator: ({ className, ...props }: React.HTMLAttributes<HTMLDivElement>) => react.JSX.Element;

declare const Dropdown_Button: typeof Button;
type Dropdown_DropdownRootProps = DropdownRootProps;
declare const Dropdown_Item: typeof Item;
declare const Dropdown_Root: typeof Root;
declare const Dropdown_Section: typeof Section;
declare const Dropdown_Separator: typeof Separator;
declare namespace Dropdown {
  export { Dropdown_Button as Button, type Dropdown_DropdownRootProps as DropdownRootProps, Dropdown_Item as Item, Dropdown_Root as Root, Dropdown_Section as Section, Dropdown_Separator as Separator };
}

declare const selectStyles: (props?: {
    variant?: "default";
    size?: "lg" | "md" | "sm";
} & class_variance_authority_types.ClassProp) => string;
interface SelectProps<TValue extends string = string> extends VariantProps<typeof selectStyles> {
    value: TValue;
    onChange: (value: TValue) => void;
    placeholder?: string;
    className?: string;
    disabled?: boolean;
    containerClassName?: string;
}
declare const Select: <TValue extends string = string>(props: PropsWithChildren<SelectProps<TValue>> & {
    ref?: React.ForwardedRef<HTMLDivElement>;
}) => React.ReactElement;
declare function SelectOption(props: PropsWithChildren<{
    value: string;
    default?: boolean;
}>): react.JSX.Element;
declare const SelectRoot: react.FC<RS.SelectProps>;
declare const SelectGroup: react.ForwardRefExoticComponent<RS.SelectGroupProps & react.RefAttributes<HTMLDivElement>>;
declare const SelectValue: react.ForwardRefExoticComponent<RS.SelectValueProps & react.RefAttributes<HTMLSpanElement>>;
declare const SelectTrigger: {
    ({ className, children, ...props }: RS.SelectTriggerProps): react.JSX.Element;
    displayName: string;
};
declare const SelectScrollUpButton: {
    ({ className, ...props }: RS.SelectScrollUpButtonProps): react.JSX.Element;
    displayName: string;
};
declare const SelectScrollDownButton: {
    ({ className, ...props }: RS.SelectScrollDownButtonProps): react.JSX.Element;
    displayName: string;
};
declare const SelectContent: {
    ({ className, children, position, ...props }: RS.SelectContentProps): react.JSX.Element;
    displayName: string;
};
declare const SelectLabel: {
    ({ className, ...props }: RS.SelectLabelProps): react.JSX.Element;
    displayName: string;
};
declare const SelectItem: {
    ({ className, children, ...props }: RS.SelectItemProps): react.JSX.Element;
    displayName: string;
};
declare const SelectSeparator: {
    ({ className, ...props }: RS.SelectSeparatorProps): react.JSX.Element;
    displayName: string;
};

declare const selectPillStyles: (props?: {
    variant?: "default" | "sidebar" | "ghost";
    size?: "lg" | "md" | "sm";
} & class_variance_authority_types.ClassProp) => string;
interface SelectPillProps extends React.ButtonHTMLAttributes<HTMLButtonElement>, VariantProps<typeof selectPillStyles> {
    /** Hide the caret icon */
    hideCaret?: boolean;
}
declare const SelectPill: react.ForwardRefExoticComponent<SelectPillProps & react.RefAttributes<HTMLButtonElement>>;

declare const TOAST_TIMEOUT = 4000;
type ToastId<T = string | number> = T;
type ToastType = "info" | "success" | "error" | "warning";
type ToastMessage = ReactNode | {
    title: ReactNode;
    body?: ReactNode;
};
type ToastPromiseData = unknown;
type ToastPromise<T = ToastPromiseData> = Promise<T> | (() => Promise<T>);
type ToastAction = {
    label: string;
    onClick: () => void;
    className?: string;
};
type ToastCloseEvent = "on-dismiss" | "on-auto-close" | "on-action" | "on-cancel";
interface ToastOptions {
    id?: ToastId;
    ref?: React.Ref<HTMLDivElement>;
    type?: ToastType;
    icon?: ReactNode;
    duration?: number;
    action?: ToastAction;
    cancel?: (Omit<ToastAction, "onClick"> & {
        onClick?: ToastAction["onClick"];
    }) | string;
    onClose?: (data: {
        id: ToastId;
        event: ToastCloseEvent;
    }) => void;
    onDismiss?: (id: ToastId) => void;
    onAutoClose?: (id: ToastId) => void;
    important?: boolean;
    style?: CSSProperties;
    className?: string;
}
interface PromiseToastOptions<T = ToastPromiseData> extends Omit<ToastOptions, "ref" | "icon" | "type" | "action" | "cancel" | "onClose"> {
    loading: ToastMessage;
    success: ToastMessage | ((data: T) => ToastMessage);
    error: ToastMessage | ((error: unknown) => ToastMessage);
    loader?: ReactNode;
    showLoader?: boolean;
    onClose?: (data: {
        id: ToastId;
        event: Extract<ToastCloseEvent, "on-dismiss" | "on-auto-close">;
    }) => void;
}
type CustomToastOptions = Omit<ToastOptions, "type" | "icon" | "action" | "cancel" | "onDismiss" | "onClose">;
declare const toast: ((message: ToastMessage | ((id: ToastId) => ToastMessage), { ref, type, icon, action, cancel, onDismiss, onClose, onAutoClose, className, ...options }?: ToastOptions) => string | number) & {
    info: (message: ToastMessage | ((id: ToastId) => ToastMessage), options?: Omit<ToastOptions, "type">) => string | number;
    success: (message: ToastMessage | ((id: ToastId) => ToastMessage), options?: Omit<ToastOptions, "type">) => string | number;
    error: (message: ToastMessage | ((id: ToastId) => ToastMessage), options?: Omit<ToastOptions, "type">) => string | number;
    warning: (message: ToastMessage | ((id: ToastId) => ToastMessage), options?: Omit<ToastOptions, "type">) => string | number;
    custom: (jsx: Parameters<typeof toast$1.custom>[0], { onAutoClose, className, ...options }?: CustomToastOptions) => string | number;
    promise: <T extends ToastPromiseData>(promise: ToastPromise<T>, { loading, success, error, onDismiss, onAutoClose, onClose, duration, className, loader, showLoader, ...options }: PromiseToastOptions<T>) => string | number;
    dismiss: (id?: number | string) => string | number;
};

declare function Loader(props: {
    className?: string;
    color?: string;
}): react.JSX.Element;

declare const Divider: ({ className, ...props }: React.HTMLAttributes<HTMLDivElement>) => react.JSX.Element;

type ProgressBarProps = {
    pending?: boolean;
} & ({
    value: number;
    total: number;
} | {
    percent: number;
});
declare const ProgressBar: react.MemoExoticComponent<(props: ProgressBarProps) => react.JSX.Element>;

type CircularProgressProps = {
    radius: number;
    progress: number;
    steps?: number;
    cut?: number;
    rotate?: number;
    strokeWidth?: number;
    strokeColor?: string;
    fillColor?: string;
    strokeLinecap?: "round" | "inherit" | "butt" | "square";
    transition?: string;
    pointerRadius?: number;
    pointerStrokeWidth?: number;
    pointerStrokeColor?: string;
    pointerFillColor?: string;
    trackStrokeColor?: string;
    trackStrokeWidth?: number;
    trackStrokeLinecap?: "round" | "inherit" | "butt" | "square";
    trackTransition?: string;
    counterClockwise?: boolean;
    inverse?: boolean;
    initialAnimation?: boolean;
    initialAnimationDelay?: number;
    className?: string;
    children?: react__default.ReactNode;
};
declare const CircularProgress: FunctionComponent<CircularProgressProps>;

interface SearchBarProps extends Omit<React.InputHTMLAttributes<HTMLInputElement>, "value" | "onChange"> {
    value?: string;
    onChange?: (value: string) => void;
    onClear?: () => void;
}
declare const SearchBar: react.ForwardRefExoticComponent<SearchBarProps & react.RefAttributes<HTMLInputElement>>;

interface ShortcutProps extends ComponentProps<"div"> {
    chars: string;
}
declare const Shortcut: (props: ShortcutProps) => react.JSX.Element;

declare const circleButtonStyles: (props?: {
    variant?: "default" | "accent" | "solid" | "active";
    size?: "lg" | "md" | "sm";
} & class_variance_authority_types.ClassProp) => string;
interface CircleButtonProps extends Omit<React.ButtonHTMLAttributes<HTMLButtonElement>, "size">, VariantProps<typeof circleButtonStyles> {
    icon?: React.ElementType;
    /** @deprecated Use variant="active" or variant="accent" instead */
    active?: boolean;
    /** @deprecated Use variant="accent" instead */
    activeAccent?: boolean;
}
declare const CircleButton: react.ForwardRefExoticComponent<CircleButtonProps & react.RefAttributes<HTMLButtonElement>>;

interface CircleButtonGroupProps {
    children: React.ReactNode;
    className?: string;
}
declare function CircleButtonGroup({ children, className, }: CircleButtonGroupProps): react.JSX.Element;

interface TabBarProps extends React.HTMLAttributes<HTMLDivElement> {
    /** Unique layout group ID for framer-motion animation coordination */
    layoutId?: string;
    /** Element rendered after the tab list (e.g. a "new tab" button) */
    trailing?: React.ReactNode;
}
declare const TabBar: react.ForwardRefExoticComponent<TabBarProps & react.RefAttributes<HTMLDivElement>>;
interface TabBarItemProps extends Omit<React.ButtonHTMLAttributes<HTMLButtonElement>, "children"> {
    /** Whether this tab is currently active */
    active?: boolean;
    /** Tab label */
    label: string;
    /** Called when the close button is clicked */
    onClose?: () => void;
    /** Show close button (default: true when onClose is provided) */
    closable?: boolean;
    /** Close button element — override to provide your own icon */
    closeIcon?: React.ReactNode;
}
declare const TabBarItem: react.ForwardRefExoticComponent<TabBarItemProps & react.RefAttributes<HTMLButtonElement>>;

declare const shinyButtonVariants: (props?: {
    size?: "default" | "small";
    glow?: "lg" | "sm" | "none";
} & class_variance_authority_types.ClassProp) => string;
type BaseShinyButtonProps = VariantProps<typeof shinyButtonVariants> & {
    icon?: Icon | ReactNode;
    children: ReactNode;
    className?: string;
};
type ShinyButtonAsButton = BaseShinyButtonProps & Omit<ComponentProps<"button">, "size"> & {
    href?: never;
};
type ShinyButtonAsLink = BaseShinyButtonProps & Omit<ComponentProps<"a">, "size"> & {
    href: string;
};
type ShinyButtonProps = ShinyButtonAsButton | ShinyButtonAsLink;
declare function ShinyButton({ icon: IconComponent, size, glow, children, className, href, ...props }: ShinyButtonProps): react.JSX.Element;

interface ShinyToggleOption<T extends string> {
    value: T;
    label: ReactNode;
    count?: number;
}
interface ShinyToggleProps<T extends string> {
    value: T;
    onChange: (value: T) => void;
    options: ShinyToggleOption<T>[];
    className?: string;
}
declare function ShinyToggle<T extends string>({ value, onChange, options, className, }: ShinyToggleProps<T>): react.JSX.Element;

declare const infoBannerStyles: (props?: {
    variant?: "gray" | "danger" | "info" | "warning";
} & class_variance_authority_types.ClassProp) => string;
interface InfoBannerProps extends VariantProps<typeof infoBannerStyles> {
    icon: ReactNode;
    children: ReactNode;
}
declare function InfoBanner({ icon, children, variant }: InfoBannerProps): react.JSX.Element;
declare function InfoBannerText({ children }: {
    children: ReactNode;
}): react.JSX.Element;
declare function InfoBannerSubtext({ children }: {
    children: ReactNode;
}): react.JSX.Element;

declare const Card$1: ForwardRefExoticComponent<HTMLAttributes<HTMLDivElement> & RefAttributes<HTMLDivElement>>;
declare const GridLayout: ForwardRefExoticComponent<HTMLAttributes<HTMLDivElement> & RefAttributes<HTMLDivElement>>;

type CardVariant = "default" | "dark";
interface CardProps extends React.HTMLAttributes<HTMLDivElement> {
    variant?: CardVariant;
}
declare const Card: react.ForwardRefExoticComponent<CardProps & react.RefAttributes<HTMLDivElement>>;
declare const CardHeader: react.ForwardRefExoticComponent<react.HTMLAttributes<HTMLDivElement> & react.RefAttributes<HTMLDivElement>>;
declare const CardTitle: react.ForwardRefExoticComponent<react.HTMLAttributes<HTMLHeadingElement> & react.RefAttributes<HTMLParagraphElement>>;
declare const CardDescription: react.ForwardRefExoticComponent<react.HTMLAttributes<HTMLParagraphElement> & react.RefAttributes<HTMLParagraphElement>>;
declare const CardContent: react.ForwardRefExoticComponent<react.HTMLAttributes<HTMLDivElement> & react.RefAttributes<HTMLDivElement>>;
declare const CardFooter: react.ForwardRefExoticComponent<react.HTMLAttributes<HTMLDivElement> & react.RefAttributes<HTMLDivElement>>;

declare const CategoryHeading: ForwardRefExoticComponent<HTMLAttributes<HTMLHeadingElement> & RefAttributes<HTMLHeadingElement>>;
declare const ScreenHeading: ForwardRefExoticComponent<HTMLAttributes<HTMLHeadingElement> & RefAttributes<HTMLHeadingElement>>;

type ResizableContextProps = Resizable$1 & {
    size: number;
    collapsed: boolean;
};
declare const useResizableContext: () => ResizableContextProps;
interface ResizableProps extends Omit<PropsWithChildren<UseResizableProps>, "axis"> {
    axis?: UseResizableProps["axis"];
    collapsed?: boolean;
    onCollapseChange?: (val: boolean) => void;
}
declare const Resizable: ({ axis, ...props }: ResizableProps) => react.JSX.Element;
declare const ResizablePanel: (props: HTMLAttributes<HTMLDivElement>) => react.JSX.Element;
declare const ResizableHandle: ({ className, ...props }: HTMLAttributes<HTMLDivElement>) => react.JSX.Element;

declare enum ModifierKeys {
    Alt = "Alt",
    Shift = "Shift",
    AltGraph = "AltGraph",
    CapsLock = "CapsLock",
    Control = "Control",
    Fn = "Fn",
    FnLock = "FnLock",
    Meta = "Meta",
    NumLock = "NumLock",
    ScrollLock = "ScrollLock",
    Symbol = "Symbol",
    SymbolLock = "SymbolLock"
}
declare enum EditingKeys {
    Backspace = "Backspace",
    Delete = "Delete"
}
declare enum UIKeys {
    Escape = "Escape"
}
declare enum NavigationKeys {
    ArrowUp = "ArrowUp",
    ArrowDown = "ArrowDown",
    ArrowLeft = "ArrowLeft",
    ArrowRight = "ArrowRight"
}
type OSforKeys = "macOS" | "Windows" | "Other";
declare const modifierSymbols: Record<ModifierKeys | EditingKeys | UIKeys | NavigationKeys, {
    macOS?: string;
    Windows?: string;
    Other: string;
}>;
declare const keySymbols: Record<string, {
    macOS?: string;
    Windows?: string;
    Other: string;
}>;

type ClassnameFactory<T> = (s: TemplateStringsArray) => T;
type TailwindFactory = {
    [K in keyof react__default.JSX.IntrinsicElements]: ClassnameFactory<react__default.ForwardRefExoticComponent<react__default.JSX.IntrinsicElements[K]>>;
} & {
    <T>(c: T): ClassnameFactory<T>;
};
declare const tw: TailwindFactory;

interface SpaceItemProps {
    /** Phosphor icon component, image URL string, or any React element type */
    icon?: React.ElementType | string;
    /** Display label */
    label: string;
    /** Active/selected state */
    active?: boolean;
    /** Colored dot instead of icon (e.g., tag color) */
    color?: string;
    /** Custom icon element (overrides icon prop — use for thumbnails, composed icons, etc.) */
    iconElement?: React.ReactNode;
    /** Right-side content (badge, count, indicator) */
    right?: React.ReactNode;
    onClick?: (e?: React.MouseEvent) => void;
    onContextMenu?: (e: React.MouseEvent) => void;
    className?: string;
}
declare const SpaceItem: react.ForwardRefExoticComponent<SpaceItemProps & react.RefAttributes<HTMLButtonElement>>;

declare const badgeVariants: (props?: {
    variant?: "default" | "outline" | "accent" | "error" | "info" | "success" | "warning" | "secondary" | "destructive";
    size?: "md" | "sm" | "default";
} & class_variance_authority_types.ClassProp) => string;
interface BadgeProps extends React.HTMLAttributes<HTMLDivElement>, VariantProps<typeof badgeVariants> {
}
declare function Badge({ className, variant, size, ...props }: BadgeProps): react.JSX.Element;

declare const bannerVariants: (props?: {
    variant?: "default" | "error" | "info" | "success" | "warning";
} & class_variance_authority_types.ClassProp) => string;
interface BannerProps extends React.HTMLAttributes<HTMLDivElement>, VariantProps<typeof bannerVariants> {
    showDot?: boolean;
}
declare const Banner: react.ForwardRefExoticComponent<BannerProps & react.RefAttributes<HTMLDivElement>>;

interface ToggleOption {
    value: string;
    label: string;
    icon?: React.ComponentType<{
        className?: string;
    }>;
    title?: string;
    className?: string;
}
interface ToggleGroupProps {
    options: ToggleOption[];
    value: string;
    onChange: (value: string) => void;
    className?: string;
    disabled?: boolean;
    itemClassName?: string;
}
declare const ToggleGroup: react.ForwardRefExoticComponent<ToggleGroupProps & react.RefAttributes<HTMLDivElement>>;

declare const Collapsible: FC<ComponentProps<typeof CollapsiblePrimitive.Root>>;
declare const CollapsibleTriggerExp: ForwardRefExoticComponent<ComponentPropsWithoutRef<typeof CollapsiblePrimitive.CollapsibleTrigger> & RefAttributes<ElementRef<typeof CollapsiblePrimitive.CollapsibleTrigger>>>;
declare const CollapsibleContentExp: ForwardRefExoticComponent<ComponentPropsWithoutRef<typeof CollapsiblePrimitive.CollapsibleContent> & RefAttributes<ElementRef<typeof CollapsiblePrimitive.CollapsibleContent>>>;

interface NumberStepperProps {
    value: number;
    onChange: (value: number) => void;
    min?: number;
    max?: number;
    step?: number;
    allowFloat?: boolean;
    disabled?: boolean;
    className?: string;
    showProgress?: boolean;
    label?: string;
    description?: string;
    suffix?: string;
}
declare const NumberStepper: react.ForwardRefExoticComponent<NumberStepperProps & react.RefAttributes<HTMLDivElement>>;

interface FilterButtonProps extends React.ButtonHTMLAttributes<HTMLButtonElement> {
    active?: boolean;
    label?: string;
}
declare const FilterButton: react.ForwardRefExoticComponent<FilterButtonProps & react.RefAttributes<HTMLButtonElement>>;

interface OptionListProps extends React.HTMLAttributes<HTMLDivElement> {
}
declare const optionListItemStyles: (props?: {
    size?: "lg" | "md" | "sm";
    selected?: boolean;
} & class_variance_authority_types.ClassProp) => string;
interface OptionListItemProps extends Omit<React.ButtonHTMLAttributes<HTMLButtonElement>, "size">, VariantProps<typeof optionListItemStyles> {
}
declare const OptionList: react.ForwardRefExoticComponent<OptionListProps & react.RefAttributes<HTMLDivElement>>;
declare const OptionListItem: react.ForwardRefExoticComponent<OptionListItemProps & react.RefAttributes<HTMLButtonElement>>;

interface SelectTriggerButtonProps extends React.ButtonHTMLAttributes<HTMLButtonElement> {
    placeholder?: string;
}
declare const SelectTriggerButton: react.ForwardRefExoticComponent<SelectTriggerButtonProps & react.RefAttributes<HTMLButtonElement>>;

type PartialKeys<T, K extends keyof T> = Omit<T, K> & Partial<Pick<T, K>>;
type EndAnchoredVirtualizerOptions<TScrollElement extends Element, TItemElement extends Element> = PartialKeys<VirtualizerOptions<TScrollElement, TItemElement>, "observeElementRect" | "observeElementOffset" | "scrollToFn">;
declare function useEndAnchoredVirtualizer<TScrollElement extends Element, TItemElement extends Element = Element>(options: EndAnchoredVirtualizerOptions<TScrollElement, TItemElement>): Virtualizer<TScrollElement, TItemElement>;

interface VirtualListHandle {
    virtualizer: Virtualizer<HTMLDivElement, Element> | null;
    scrollToEnd: (opts?: {
        behavior?: ScrollBehavior;
    }) => void;
    scrollToIndex: (index: number, opts?: {
        align?: "start" | "center" | "end" | "auto";
        behavior?: ScrollBehavior;
    }) => void;
    isAtEnd: () => boolean;
    getDistanceFromEnd: () => number;
}
interface VirtualListProps<T> {
    items: ReadonlyArray<T>;
    getItemKey: (index: number) => string | number;
    estimateSize: (index: number) => number;
    renderItem: (item: T, index: number) => ReactNode;
    overscan?: number;
    anchorTo?: "start" | "end";
    followOnAppend?: boolean | ScrollBehavior;
    scrollEndThreshold?: number;
    onReachStart?: () => void;
    handleRef?: Ref<VirtualListHandle>;
    className?: string;
    innerClassName?: string;
}
declare function VirtualList<T>({ items, getItemKey, estimateSize, renderItem, overscan, anchorTo, followOnAppend, scrollEndThreshold, onReachStart, handleRef, className, innerClassName, }: VirtualListProps<T>): react.JSX.Element;

interface JumpToEndButtonProps<TScrollElement extends Element = Element, TItemElement extends Element = Element> {
    virtualizer: Virtualizer<TScrollElement, TItemElement> | null | undefined;
    className?: string;
    children?: ReactNode;
    "aria-label"?: string;
}
declare function JumpToEndButton<TScrollElement extends Element = Element, TItemElement extends Element = Element>({ virtualizer, className, children, "aria-label": ariaLabel, }: JumpToEndButtonProps<TScrollElement, TItemElement>): react.JSX.Element;

export { Badge, Banner, Button$1 as Button, type ButtonBaseProps, type ButtonProps, Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle, CategoryHeading, CheckBox, type CheckBoxProps, CheckboxIndicator, CheckboxRoot, CircleButton, CircleButtonGroup, type CircleButtonGroupProps, type CircleButtonProps, CircularProgress, type CircularProgressProps, Collapsible, CollapsibleContentExp as CollapsibleContent, CollapsibleTriggerExp as CollapsibleTrigger, ContextMenu, type ContextMenuCheckboxItemProps, ContextMenuDivItem, type ContextMenuItemProps, Dialog, DialogClose, DialogContent, DialogDescription, DialogFooter, DialogHeader, type DialogOptions, DialogOverlay, DialogPortal, type DialogProps, DialogRoot, type DialogState, DialogTitle, DialogTrigger, Dialogs, Divider, Dropdown, DropdownMenu, CheckboxItem as DropdownMenuCheckboxItem, Content as DropdownMenuContent, Group as DropdownMenuGroup, Item$1 as DropdownMenuItem, Label as DropdownMenuLabel, Portal as DropdownMenuPortal, RadioGroup as DropdownMenuRadioGroup, RadioItem as DropdownMenuRadioItem, Root$1 as DropdownMenuRoot, Separator$1 as DropdownMenuSeparator, Sub as DropdownMenuSub, SubContent as DropdownMenuSubContent, SubTrigger as DropdownMenuSubTrigger, Trigger as DropdownMenuTrigger, EditingKeys, type EndAnchoredVirtualizerOptions, ErrorMessage, type ErrorMessageProps, FilterButton, Form, type FormProps, GridLayout, InfoBanner, InfoBannerSubtext, InfoBannerText, Input, type InputBaseProps, type InputProps, JumpToEndButton, type JumpToEndButtonProps, Kbd, Label$1 as Label, type LabelProps, Card$1 as LayoutCard, type LinkButtonProps, Loader, ModifierKeys, NavigationKeys, NumberStepper, type OSforKeys, OptionList, OptionListItem, PasswordInput, Popover, Anchor as PopoverAnchor, Close as PopoverClose, Content$2 as PopoverContent, Portal$1 as PopoverPortal, Root$3 as PopoverRoot, Trigger$2 as PopoverTrigger, ProgressBar, type ProgressBarProps, Item$2 as RadioGroupItem, type ItemProps as RadioGroupItemProps, Root$4 as RadioGroupRoot, type RootProps as RadioGroupRootProps, RadixCheckbox, type RadixCheckboxProps, Resizable, ResizableHandle, ResizablePanel, ScreenHeading, SearchBar, SearchInput, Select, SelectContent, SelectGroup, SelectItem, SelectLabel, SelectOption, SelectPill, type SelectPillProps, type SelectProps, SelectRoot, SelectScrollDownButton, SelectScrollUpButton, SelectSeparator, SelectTrigger, SelectTriggerButton, SelectValue, ShinyButton, type ShinyButtonProps, ShinyToggle, Shortcut, type ShortcutProps, Slider, SpaceItem, type SpaceItemProps, Switch, type SwitchProps, TOAST_TIMEOUT, TabBar, TabBarItem, type TabBarItemProps, type TabBarProps, Content$1 as TabsContent, List as TabsList, Root$2 as TabsRoot, Trigger$1 as TabsTrigger, TextArea, type TextareaProps, type ToastId, type ToastMessage, ToggleGroup, Tooltip, TooltipContent, TooltipPortal, type TooltipProps, TooltipProvider, TooltipRoot, TooltipTrigger, CircleButton as TopBarButton, CircleButtonGroup as TopBarButtonGroup, UIKeys, type UseDialogProps, VirtualList, type VirtualListHandle, type VirtualListProps, badgeVariants, bannerVariants, buttonStyles, buttonStyles as buttonVariants, circleButtonStyles, contextMenuClassNames, contextMenuItemClassNames, contextMenuSeparatorClassNames, dialogManager, errorStyles, inputSizes, inputStyles, keySymbols, modifierSymbols, selectPillStyles, selectStyles, toast, tw, useContextMenuContext, useDialog, useEndAnchoredVirtualizer, usePopover, useResizableContext };
