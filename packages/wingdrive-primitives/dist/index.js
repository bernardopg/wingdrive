var __defProp = Object.defineProperty;
var __export = (target, all) => {
  for (var name in all)
    __defProp(target, name, { get: all[name], enumerable: true });
};

// src/Button.tsx
import { cva, cx } from "class-variance-authority";
import { forwardRef } from "react";
import { jsx } from "react/jsx-runtime";
var hasHref = (props) => "href" in props;
var buttonStyles = cva(
  [
    "inline-flex cursor-default items-center justify-center gap-1.5 border font-medium tracking-wide outline-none transition-colors duration-100",
    "disabled:pointer-events-none disabled:cursor-not-allowed disabled:opacity-70",
    "cursor-pointer ring-offset-app-box focus-visible:ring-2 focus-visible:ring-accent focus-visible:ring-offset-2"
  ],
  {
    variants: {
      size: {
        icon: "!p-1",
        lg: "text-md px-3 py-1.5 font-medium",
        md: "px-2.5 py-1.5 text-sm font-medium",
        sm: "px-2 py-0.5 text-sm font-medium",
        xs: "px-1.5 py-0.5 text-xs font-normal"
      },
      variant: {
        default: [
          "bg-transparent hover:bg-app-hover active:bg-app-selected",
          "border border-app-line/80 hover:border-app-line active:border-app-line"
        ],
        subtle: [
          "border-transparent hover:border-app-line/50 active:border-app-line active:bg-app-box/30"
        ],
        outline: [
          "border-sidebar-line/60 hover:border-sidebar-line active:border-sidebar-line/30"
        ],
        dotted: [
          "rounded border border-dashed border-sidebar-line/70 text-center text-xs font-medium text-ink-faint transition hover:border-sidebar-line hover:bg-sidebar-selected/5"
        ],
        gray: [
          "bg-app-button hover:bg-app-hover focus:bg-app-selected text-ink",
          "border border-app-line/50 hover:border-app-line/70 focus:ring-1 focus:ring-accent"
        ],
        accent: [
          "border-accent bg-accent text-white shadow-md shadow-app-shade/10 hover:brightness-110 focus:outline-none",
          "focus:ring-1 focus:ring-accent focus:ring-offset-2 focus:ring-offset-app-selected"
        ],
        colored: [
          "text-white shadow-sm hover:bg-opacity-90 active:bg-opacity-100"
        ],
        bare: ""
      },
      rounding: {
        none: "rounded-none",
        left: "rounded-l-md rounded-r-none",
        right: "rounded-l-none rounded-r-md",
        both: "rounded-xl",
        full: "rounded-full"
      }
    },
    defaultVariants: {
      size: "sm",
      variant: "default",
      rounding: "both"
    }
  }
);
var Button = forwardRef(({ className, loading, ...props }, ref) => {
  if (loading && !hasHref(props))
    props.disabled = true;
  className = cx(buttonStyles(props), loading && "pointer-events-none opacity-70", className);
  return hasHref(props) ? /* @__PURE__ */ jsx(
    "a",
    {
      ...props,
      ref,
      className: cx(className, "inline-block no-underline")
    }
  ) : /* @__PURE__ */ jsx(
    "button",
    {
      type: "button",
      ...props,
      ref,
      className
    }
  );
});
Button.displayName = "Button";

// src/Input.tsx
import {
  Eye,
  EyeSlash,
  MagnifyingGlass
} from "@phosphor-icons/react";
import { cva as cva2 } from "class-variance-authority";
import clsx from "clsx";
import { createElement, forwardRef as forwardRef2, isValidElement, useState } from "react";
import { jsx as jsx2, jsxs } from "react/jsx-runtime";
var inputSizes = {
  xs: "h-[25px]",
  sm: "h-[30px]",
  md: "h-[36px]",
  lg: "h-[42px]",
  xl: "h-[48px]"
};
var inputStyles = cva2(
  [
    "rounded-lg border text-sm leading-4",
    "outline-none transition-[background-color,border-color,box-shadow] focus-within:ring-2 focus-within:ring-accent",
    "text-ink"
  ],
  {
    variants: {
      variant: {
        default: [
          "border-app-line bg-app-input/50 border-app-line active:border-app-line placeholder-ink-faint"
        ],
        transparent: [
          "border-transparent bg-transparent placeholder-ink-dull focus-within:bg-transparent",
          "focus-within:border-transparent focus-within:ring-transparent"
        ]
      },
      error: {
        true: "border-red-500 focus-within:border-red-500 focus-within:ring-red-400/30"
      },
      size: inputSizes
    },
    defaultVariants: {
      variant: "default",
      size: "md"
    }
  }
);
var Input = forwardRef2(
  ({
    variant,
    size,
    right,
    icon,
    iconPosition = "left",
    className,
    error,
    ...props
  }, ref) => /* @__PURE__ */ jsxs(
    "div",
    {
      className: clsx(
        "group flex",
        inputStyles({
          variant,
          size: right && !size ? "md" : size,
          error,
          className
        })
      ),
      children: [
        /* @__PURE__ */ jsxs(
          "div",
          {
            className: clsx(
              "flex h-full flex-1 overflow-hidden",
              iconPosition === "right" && "flex-row-reverse"
            ),
            children: [
              icon && /* @__PURE__ */ jsx2(
                "div",
                {
                  className: clsx(
                    "flex h-full items-center",
                    iconPosition === "left" ? "pl-[10px] pr-2" : "pl-2 pr-[10px]"
                  ),
                  children: isValidElement(icon) ? icon : createElement(icon, {
                    size: 18,
                    className: "text-gray-350"
                  })
                }
              ),
              /* @__PURE__ */ jsx2(
                "input",
                {
                  className: clsx(
                    "flex-1 truncate border-none bg-transparent px-3 text-sm outline-none placeholder:text-ink-faint focus:!ring-0",
                    (right || icon && iconPosition === "right") && "pr-0",
                    icon && iconPosition === "left" && "pl-0",
                    size === "xs" && "!py-0",
                    props.inputElementClassName
                  ),
                  onKeyDown: (e) => {
                    e.stopPropagation();
                  },
                  ref,
                  autoComplete: props.autoComplete || "off",
                  ...props
                }
              )
            ]
          }
        ),
        right && /* @__PURE__ */ jsx2(
          "div",
          {
            className: clsx(
              "flex h-full min-w-[12px] items-center",
              size === "lg" ? "px-[5px]" : "px-1"
            ),
            children: right
          }
        )
      ]
    }
  )
);
Input.displayName = "Input";
var SearchInput = forwardRef2(
  (props, ref) => /* @__PURE__ */ jsx2(Input, { ...props, ref, icon: MagnifyingGlass })
);
SearchInput.displayName = "SearchInput";
var TextArea = forwardRef2(
  ({ size, variant, error, ...props }, ref) => {
    return /* @__PURE__ */ jsx2(
      "textarea",
      {
        ...props,
        ref,
        onKeyDown: (e) => {
          e.stopPropagation();
        },
        className: clsx(
          "h-auto p-2",
          inputStyles({ size, variant, error }),
          props.className
        )
      }
    );
  }
);
TextArea.displayName = "TextArea";
function Label({ slug, children, className, ...props }) {
  return /* @__PURE__ */ jsx2(
    "label",
    {
      htmlFor: slug,
      className: clsx("text-sm font-bold", className),
      ...props,
      children
    }
  );
}
var PasswordInput = forwardRef2(
  (props, ref) => {
    const [showPassword, setShowPassword] = useState(false);
    const CurrentEyeIcon = showPassword ? EyeSlash : Eye;
    return /* @__PURE__ */ jsx2(
      Input,
      {
        ...props,
        type: showPassword ? "text" : "password",
        ref,
        onKeyDown: (e) => {
          e.stopPropagation();
        },
        right: /* @__PURE__ */ jsx2(
          Button,
          {
            tabIndex: 0,
            onClick: () => setShowPassword(!showPassword),
            size: "icon",
            "aria-label": showPassword ? "Hide Password" : "Show Password",
            className: clsx(props.buttonClassnames),
            children: /* @__PURE__ */ jsx2(CurrentEyeIcon, { className: "!pointer-events-none size-4" })
          }
        )
      }
    );
  }
);
PasswordInput.displayName = "PasswordInput";

// src/Checkbox.tsx
import { Check } from "@phosphor-icons/react";
import * as CheckboxPrimitive from "@radix-ui/react-checkbox";
import { cva as cva3 } from "class-variance-authority";
import clsx2 from "clsx";
import { forwardRef as forwardRef3 } from "react";
import { jsx as jsx3, jsxs as jsxs2 } from "react/jsx-runtime";
var styles = cva3(
  [
    "form-check-input float-left mr-2 mt-1 size-4 appearance-none rounded-sm border border-gray-300 bg-white bg-contain bg-center bg-no-repeat align-top transition duration-200",
    "checked:border-accent checked:bg-accent checked:hover:bg-accent/80 focus:outline-none"
  ],
  { variants: {} }
);
var CheckBox = forwardRef3(
  ({ className, ...props }, ref) => /* @__PURE__ */ jsx3(
    "input",
    {
      ...props,
      type: "checkbox",
      ref,
      className: styles({ className })
    }
  )
);
CheckBox.displayName = "CheckBox";
var RadixCheckbox = ({
  className,
  labelClassName,
  ...props
}) => /* @__PURE__ */ jsxs2("div", { className: clsx2("flex items-center", className), children: [
  /* @__PURE__ */ jsx3(
    CheckboxPrimitive.Root,
    {
      className: "flex size-[15px] shrink-0 items-center justify-center rounded-[4px] border border-gray-300/10 bg-app-selected radix-state-checked:bg-accent",
      id: props.name,
      ...props,
      children: /* @__PURE__ */ jsx3(CheckboxPrimitive.Indicator, { className: "text-white", children: /* @__PURE__ */ jsx3(Check, { weight: "bold", size: 12 }) })
    }
  ),
  props.label && /* @__PURE__ */ jsx3(
    "label",
    {
      className: clsx2("ml-2 text-sm font-medium", labelClassName),
      htmlFor: props.name,
      children: props.label
    }
  )
] });
var CheckboxRoot = CheckboxPrimitive.Root;
var CheckboxIndicator = CheckboxPrimitive.Indicator;

// src/Switch.tsx
import * as SwitchPrimitive from "@radix-ui/react-switch";
import { cva as cva4 } from "class-variance-authority";
import { forwardRef as forwardRef4 } from "react";
import { jsx as jsx4 } from "react/jsx-runtime";
var switchStyles = cva4(
  [
    "relative inline-flex shrink-0 transition",
    "items-center rounded-full p-1",
    "bg-app-line focus:outline-none focus:ring-1 focus:ring-accent focus:ring-offset-2 focus:ring-offset-app-selected radix-state-checked:bg-accent"
  ],
  {
    variants: {
      size: {
        sm: "h-[20px] w-[34px]",
        md: "h-[25px] w-[47px]",
        lg: "h-[30px] w-[55px]"
      }
    },
    defaultVariants: {
      size: "md"
    }
  }
);
var thumbStyles = cva4(
  [
    "inline-block size-4 transition",
    "rounded-full bg-white",
    "shadow-sm shadow-app-shade/40"
  ],
  {
    variants: {
      size: {
        sm: "size-[12px] radix-state-checked:translate-x-[14px]",
        md: "size-[19px] radix-state-checked:translate-x-[20px]",
        lg: "size-6 radix-state-checked:translate-x-[23px]"
      }
    },
    defaultVariants: {
      size: "md"
    }
  }
);
var Switch = forwardRef4(
  ({ size, className, thumbClassName, ...props }, ref) => /* @__PURE__ */ jsx4(
    SwitchPrimitive.Root,
    {
      ...props,
      ref,
      className: switchStyles({ size, className }),
      children: /* @__PURE__ */ jsx4(
        SwitchPrimitive.Thumb,
        {
          className: thumbStyles({ size, className: thumbClassName })
        }
      )
    }
  )
);
Switch.displayName = "Switch";

// src/Slider.tsx
import * as SliderPrimitive from "@radix-ui/react-slider";
import clsx3 from "clsx";
import { jsx as jsx5, jsxs as jsxs3 } from "react/jsx-runtime";
var Slider = (props) => /* @__PURE__ */ jsxs3(
  SliderPrimitive.Root,
  {
    ...props,
    className: clsx3(
      "relative flex h-6 w-full select-none items-center",
      props.className
    ),
    children: [
      /* @__PURE__ */ jsx5(SliderPrimitive.Track, { className: "relative h-2 grow rounded-full bg-app-slider outline-none", children: /* @__PURE__ */ jsx5(SliderPrimitive.Range, { className: "absolute h-full rounded-full bg-accent outline-none" }) }),
      /* @__PURE__ */ jsx5(
        SliderPrimitive.Thumb,
        {
          className: "z-50 block size-5 rounded-full bg-accent font-bold shadow-lg shadow-black/20 outline-none ring-accent/30 transition focus:ring-4",
          "data-tip": "1.0"
        }
      )
    ]
  }
);

// src/RadioGroup.tsx
import * as RadioGroupPrimitive from "@radix-ui/react-radio-group";
import clsx4 from "clsx";
import { forwardRef as forwardRef5 } from "react";
import { jsx as jsx6, jsxs as jsxs4 } from "react/jsx-runtime";
var Root5 = forwardRef5(
  ({ children, className, ...props }, ref) => {
    return /* @__PURE__ */ jsx6(RadioGroupPrimitive.Root, { ...props, ref, children: /* @__PURE__ */ jsx6("div", { className: clsx4("space-y-3", className), children }) });
  }
);
Root5.displayName = "RadioGroupRoot";
var Item2 = ({ children, ...props }) => {
  return /* @__PURE__ */ jsxs4(
    "div",
    {
      className: clsx4(
        "flex max-w-sm space-x-2 rounded-md border border-app-line bg-app-box/50 px-4 py-3",
        props.disabled && "opacity-30"
      ),
      children: [
        /* @__PURE__ */ jsx6(
          RadioGroupPrimitive.Item,
          {
            id: "radio" + props.value,
            className: clsx4(
              "peer relative mr-1 mt-1 size-4 shrink-0 rounded-full border border-app-line",
              "radix-state-checked:bg-accent",
              "radix-state-unchecked:bg-app-input",
              "focus:outline-none focus:ring-0 focus:ring-offset-0 focus-visible:ring focus-visible:ring-accent focus-visible:ring-opacity-75 focus-visible:ring-offset-2"
            ),
            ...props,
            children: /* @__PURE__ */ jsx6(RadioGroupPrimitive.Indicator, { className: "leading-0 absolute inset-0 flex items-center justify-center", children: /* @__PURE__ */ jsx6("div", { className: "size-1.5 rounded-full bg-white" }) })
          }
        ),
        /* @__PURE__ */ jsx6("label", { htmlFor: "radio" + props.value, children })
      ]
    }
  );
};

// src/Dialog.tsx
import * as RDialog from "@radix-ui/react-dialog";
import { X } from "@phosphor-icons/react";
import { animated as animated2, useTransition as useTransition2 } from "@react-spring/web";
import clsx6 from "clsx";
import { useEffect, useState as useState2 } from "react";

// src/Loader.tsx
import clsx5 from "clsx";
import { Puff } from "react-loading-icons";
import { jsx as jsx7 } from "react/jsx-runtime";
function Loader(props) {
  return /* @__PURE__ */ jsx7(
    Puff,
    {
      stroke: props.color || "#2599FF",
      strokeOpacity: 4,
      strokeWidth: 5,
      speed: 1,
      className: clsx5("size-7", props.className)
    }
  );
}

// src/forms/Form.tsx
import { Warning } from "@phosphor-icons/react";
import { animated, useTransition } from "@react-spring/web";
import { cva as cva5 } from "class-variance-authority";
import {
  FormProvider,
  get,
  useFormContext
} from "react-hook-form";
import { z } from "zod";
import { Fragment, jsx as jsx8, jsxs as jsxs5 } from "react/jsx-runtime";
var Form = ({
  form,
  disabled,
  onSubmit,
  children,
  ...props
}) => {
  return /* @__PURE__ */ jsx8(FormProvider, { ...form, children: /* @__PURE__ */ jsx8(
    "form",
    {
      onSubmit: (e) => {
        e.stopPropagation();
        e.preventDefault();
        return onSubmit?.(e);
      },
      ...props,
      children: /* @__PURE__ */ jsx8(
        "fieldset",
        {
          disabled: disabled || form.formState.isSubmitting,
          className: "min-w-0",
          children
        }
      )
    }
  ) });
};
var errorStyles = cva5(
  "flex justify-center gap-2 whitespace-normal break-words rounded border border-red-500/40 bg-red-800/40 px-3 py-2 text-white",
  {
    variants: {
      variant: {
        none: "",
        default: "w-full text-xs",
        large: "text-left text-xs font-semibold"
      }
    },
    defaultVariants: {
      variant: "default"
    }
  }
);
var AnimatedDiv = animated.div;
var ErrorMessage = ({
  name,
  variant,
  className
}) => {
  const methods = useFormContext();
  const error = get(methods.formState.errors, name);
  const transitions = useTransition(error, {
    from: { opacity: 0 },
    enter: { opacity: 1 },
    leave: { opacity: 0 },
    clamp: true,
    config: { mass: 0.4, tension: 200, friction: 10, bounce: 0 },
    exitBeforeEnter: true
  });
  return /* @__PURE__ */ jsx8(Fragment, { children: transitions((styles2, error2) => {
    const message = error2?.message;
    return typeof message === "string" ? /* @__PURE__ */ jsxs5(
      AnimatedDiv,
      {
        style: styles2,
        className: errorStyles({ variant, className }),
        children: [
          /* @__PURE__ */ jsx8(Warning, { className: "size-4" }),
          /* @__PURE__ */ jsx8("p", { className: "whitespace-normal", children: message })
        ]
      }
    ) : null;
  }) });
};

// src/Dialog.tsx
import { Fragment as Fragment2, jsx as jsx9, jsxs as jsxs6 } from "react/jsx-runtime";
var DialogManager = class {
  idGenerator = 0;
  listeners = /* @__PURE__ */ new Map();
  states = /* @__PURE__ */ new Map();
  components = /* @__PURE__ */ new Map();
  create(dialog, options) {
    const id = this.getId();
    this.components.set(id, () => dialog({ id, ...options }));
    this.states.set(id, { open: true });
    this.listeners.set(id, /* @__PURE__ */ new Set());
    this.notifyGlobalListeners();
    return new Promise((res) => {
      const checkInterval = setInterval(() => {
        if (!this.components.has(id)) {
          clearInterval(checkInterval);
          res();
        }
      }, 100);
    });
  }
  getId() {
    return ++this.idGenerator;
  }
  getState(id) {
    return this.states.get(id);
  }
  setState(id, state) {
    const current = this.states.get(id);
    if (!current) return;
    const newState = { ...current, ...state };
    this.states.set(id, newState);
    const listeners = this.listeners.get(id);
    if (listeners) {
      listeners.forEach((listener) => listener(newState));
    }
  }
  subscribe(id, listener) {
    const listeners = this.listeners.get(id);
    if (listeners) {
      listeners.add(listener);
    }
    return () => {
      const listeners2 = this.listeners.get(id);
      if (listeners2) {
        listeners2.delete(listener);
      }
    };
  }
  globalListeners = /* @__PURE__ */ new Set();
  subscribeGlobal(listener) {
    this.globalListeners.add(listener);
    return () => {
      this.globalListeners.delete(listener);
    };
  }
  notifyGlobalListeners() {
    this.globalListeners.forEach((listener) => listener());
  }
  getComponents() {
    return Array.from(this.components.entries());
  }
  isAnyDialogOpen() {
    return Array.from(this.states.values()).some((s) => s.open);
  }
  remove(id) {
    const state = this.getState(id);
    if (!state) {
      console.error(new Error(`Dialog ${id} not registered!`));
    } else if (state.open === false) {
      this.components.delete(id);
      this.states.delete(id);
      this.listeners.delete(id);
      this.notifyGlobalListeners();
    }
  }
};
var dialogManager = new DialogManager();
function Remover({ id }) {
  useEffect(
    () => () => {
      dialogManager.remove(id);
    },
    [id]
  );
  return null;
}
function useDialog(props) {
  const [state, setState] = useState2(() => {
    const initialState = dialogManager.getState(props.id);
    if (!initialState) throw new Error(`Dialog ${props.id} does not exist!`);
    return initialState;
  });
  useEffect(() => {
    return dialogManager.subscribe(props.id, setState);
  }, [props.id]);
  return {
    ...props,
    state
  };
}
function Dialogs() {
  const [, forceUpdate] = useState2({});
  useEffect(() => {
    return dialogManager.subscribeGlobal(() => {
      forceUpdate({});
    });
  }, []);
  const dialogs = dialogManager.getComponents();
  return /* @__PURE__ */ jsx9(Fragment2, { children: dialogs.map(([id, Dialog2]) => /* @__PURE__ */ jsx9(Dialog2, {}, id)) });
}
var AnimatedDialogContent = animated2(RDialog.Content);
var AnimatedDialogOverlay = animated2(RDialog.Overlay);
function Dialog({
  form,
  dialog,
  onSubmit,
  onSubmitSecond,
  onCancelled = true,
  invertButtonFocus,
  ...props
}) {
  const transitions = useTransition2(dialog.state.open, {
    from: {
      opacity: 0,
      transform: "translateY(20px)",
      transformOrigin: props.transformOrigin || "bottom"
    },
    enter: { opacity: 1, transform: "translateY(0px)" },
    leave: { opacity: 0, transform: "translateY(20px)" },
    config: { mass: 0.4, tension: 200, friction: 10, bounce: 0 }
  });
  const setOpen = (v) => dialogManager.setState(dialog.id, { open: v });
  const cancelButton = /* @__PURE__ */ jsx9(RDialog.Close, { asChild: true, children: /* @__PURE__ */ jsx9(
    Button,
    {
      size: "sm",
      variant: props.cancelDanger ? "colored" : "gray",
      onClick: typeof onCancelled === "function" ? onCancelled : void 0,
      className: clsx6(props.cancelDanger && "border-red-500 bg-red-500"),
      children: props.cancelLabel || "Cancel"
    }
  ) });
  const closeButton = /* @__PURE__ */ jsx9(RDialog.Close, { asChild: true, children: /* @__PURE__ */ jsx9(
    Button,
    {
      disabled: props.loading,
      size: "sm",
      variant: "gray",
      onClick: typeof onCancelled === "function" ? onCancelled : void 0,
      children: props.closeLabel || "Close"
    }
  ) });
  const disableCheck = props.errorMessageException ? !form.formState.isValid && !form.formState.errors.root?.serverError?.message?.startsWith(
    props.errorMessageException
  ) : !form.formState.isValid;
  const submitButton = props.ctaLabel ? !props.ctaSecondLabel ? /* @__PURE__ */ jsx9(
    Button,
    {
      type: "submit",
      size: "sm",
      disabled: form.formState.isSubmitting || props.submitDisabled || disableCheck,
      variant: props.ctaDanger ? "colored" : "accent",
      onClick: async (e) => {
        e.preventDefault();
        await onSubmit?.(e);
        dialog.onSubmit?.();
      },
      children: props.ctaLabel
    }
  ) : /* @__PURE__ */ jsxs6("div", { className: "flex flex-row gap-x-2", children: [
    /* @__PURE__ */ jsx9(
      Button,
      {
        type: "submit",
        size: "sm",
        disabled: form.formState.isSubmitting || props.submitDisabled || disableCheck,
        variant: props.ctaDanger ? "colored" : "accent",
        className: clsx6(props.ctaDanger && "border-red-500 bg-red-500"),
        onClick: async (e) => {
          e.preventDefault();
          await onSubmit?.(e);
          dialog.onSubmit?.();
        },
        children: props.ctaLabel
      }
    ),
    /* @__PURE__ */ jsx9(
      Button,
      {
        type: "submit",
        size: "sm",
        disabled: form.formState.isSubmitting || props.submitDisabled || disableCheck,
        variant: "accent",
        onClick: async (e) => {
          e.preventDefault();
          await onSubmitSecond?.(e);
          dialog.onSubmit?.();
        },
        children: props.ctaSecondLabel
      }
    )
  ] }) : null;
  return /* @__PURE__ */ jsxs6(RDialog.Root, { open: dialog.state.open, onOpenChange: setOpen, children: [
    props.trigger && /* @__PURE__ */ jsx9(RDialog.Trigger, { asChild: true, children: props.trigger }),
    transitions(
      (styles2, show) => show ? /* @__PURE__ */ jsxs6(RDialog.Portal, { forceMount: true, children: [
        /* @__PURE__ */ jsx9(
          AnimatedDialogOverlay,
          {
            className: "fixed inset-0 z-[102] m-px grid place-items-center overflow-y-auto rounded-xl bg-app/50",
            style: {
              opacity: styles2.opacity
            }
          }
        ),
        /* @__PURE__ */ jsxs6(
          AnimatedDialogContent,
          {
            className: "!pointer-events-none fixed inset-0 z-[103] grid place-items-center overflow-y-auto",
            style: styles2,
            onInteractOutside: (e) => props.ignoreClickOutside && e.preventDefault(),
            children: [
              /* @__PURE__ */ jsxs6(
                Form,
                {
                  form,
                  onSubmit: async (e) => {
                    e?.preventDefault();
                    if (onSubmit) {
                      await onSubmit(e);
                    }
                  },
                  className: clsx6(
                    "!pointer-events-auto my-8 min-w-[300px] max-w-[400px] rounded-xl",
                    "border border-app-line bg-app-box text-ink shadow-app-shade",
                    props.formClassName
                  ),
                  children: [
                    !props.hideHeader && /* @__PURE__ */ jsxs6(RDialog.Title, { className: "flex items-center gap-2.5 border-b border-app-line bg-app-input/60 p-3 font-bold", children: [
                      props.icon && props.icon,
                      props.title
                    ] }),
                    /* @__PURE__ */ jsxs6("div", { className: "flex-1 overflow-auto p-5", children: [
                      props.description && /* @__PURE__ */ jsx9(RDialog.Description, { className: "mb-2 text-sm text-ink-dull", children: props.description }),
                      props.children
                    ] }),
                    (props.buttonsSideContent || !props.hideButtons && (submitButton || props.cancelBtn || onCancelled)) && /* @__PURE__ */ jsxs6(
                      "div",
                      {
                        className: clsx6(
                          "flex items-center justify-end space-x-2 border-t border-app-line bg-app-input/60 p-3"
                        ),
                        children: [
                          form.formState.isSubmitting && /* @__PURE__ */ jsx9(Loader, {}),
                          props.buttonsSideContent && /* @__PURE__ */ jsx9("div", { children: props.buttonsSideContent }),
                          /* @__PURE__ */ jsx9("div", { className: "grow" }),
                          !props.hideButtons && /* @__PURE__ */ jsx9(
                            "div",
                            {
                              className: clsx6(
                                invertButtonFocus ? "flex-row-reverse" : "flex-row",
                                "flex gap-2"
                              ),
                              children: invertButtonFocus ? /* @__PURE__ */ jsxs6(Fragment2, { children: [
                                submitButton,
                                props.cancelBtn && cancelButton,
                                onCancelled && closeButton
                              ] }) : /* @__PURE__ */ jsxs6(Fragment2, { children: [
                                onCancelled && closeButton,
                                props.cancelBtn && cancelButton,
                                submitButton
                              ] })
                            }
                          )
                        ]
                      }
                    )
                  ]
                }
              ),
              /* @__PURE__ */ jsx9(Remover, { id: dialog.id })
            ]
          }
        )
      ] }) : null
    )
  ] });
}
var DialogRoot = RDialog.Root;
var DialogTrigger = RDialog.Trigger;
var DialogPortal = RDialog.Portal;
var DialogClose = RDialog.Close;
var DialogOverlay = ({
  className,
  ...props
}) => /* @__PURE__ */ jsx9(
  RDialog.Overlay,
  {
    className: clsx6(
      "fixed inset-0 z-50 bg-black/50 data-[state=open]:animate-in data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=open]:fade-in-0",
      className
    ),
    ...props
  }
);
DialogOverlay.displayName = "DialogOverlay";
var DialogContent = ({
  className,
  children,
  ...props
}) => /* @__PURE__ */ jsxs6(DialogPortal, { children: [
  /* @__PURE__ */ jsx9(DialogOverlay, {}),
  /* @__PURE__ */ jsxs6(
    RDialog.Content,
    {
      className: clsx6(
        "fixed left-[50%] top-[50%] z-50 grid w-full max-w-lg translate-x-[-50%] translate-y-[-50%] gap-4 border border-app-line bg-app-dark-box p-6 shadow-lg duration-200 data-[state=open]:animate-in data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=open]:fade-in-0 data-[state=closed]:zoom-out-95 data-[state=open]:zoom-in-95 data-[state=closed]:slide-out-to-left-1/2 data-[state=closed]:slide-out-to-top-[48%] data-[state=open]:slide-in-from-left-1/2 data-[state=open]:slide-in-from-top-[48%] sm:rounded-lg",
        className
      ),
      ...props,
      children: [
        children,
        /* @__PURE__ */ jsxs6(RDialog.Close, { className: "absolute right-4 top-4 rounded-sm opacity-70 ring-offset-app-dark-box transition-opacity hover:opacity-100 focus:outline-none focus:ring-2 focus:ring-accent focus:ring-offset-2 disabled:pointer-events-none data-[state=open]:bg-app-button data-[state=open]:text-ink-dull", children: [
          /* @__PURE__ */ jsx9(X, { className: "size-4" }),
          /* @__PURE__ */ jsx9("span", { className: "sr-only", children: "Close" })
        ] })
      ]
    }
  )
] });
DialogContent.displayName = "DialogContent";
var DialogHeader = ({
  className,
  ...props
}) => /* @__PURE__ */ jsx9(
  "div",
  {
    className: clsx6(
      "flex flex-col space-y-1.5 text-center sm:text-left",
      className
    ),
    ...props
  }
);
DialogHeader.displayName = "DialogHeader";
var DialogFooter = ({
  className,
  ...props
}) => /* @__PURE__ */ jsx9(
  "div",
  {
    className: clsx6(
      "flex flex-col-reverse sm:flex-row sm:justify-end sm:space-x-2",
      className
    ),
    ...props
  }
);
DialogFooter.displayName = "DialogFooter";
var DialogTitle = ({
  className,
  ...props
}) => /* @__PURE__ */ jsx9(
  RDialog.Title,
  {
    className: clsx6(
      "text-lg font-semibold leading-none tracking-tight text-ink",
      className
    ),
    ...props
  }
);
DialogTitle.displayName = "DialogTitle";
var DialogDescription = ({
  className,
  ...props
}) => /* @__PURE__ */ jsx9(
  RDialog.Description,
  {
    className: clsx6("text-sm text-ink-dull", className),
    ...props
  }
);
DialogDescription.displayName = "DialogDescription";

// src/Popover.tsx
import * as PopoverPrimitive from "@radix-ui/react-popover";
import clsx7 from "clsx";
import { forwardRef as forwardRef6, useState as useState3 } from "react";
import { jsx as jsx10 } from "react/jsx-runtime";
function usePopover() {
  const [open, setOpen] = useState3(false);
  return { open, setOpen };
}
var Root8 = PopoverPrimitive.Root;
var Trigger3 = PopoverPrimitive.Trigger;
var Anchor2 = PopoverPrimitive.Anchor;
var Close3 = PopoverPrimitive.Close;
var Portal3 = PopoverPrimitive.Portal;
var Content3 = forwardRef6(({ className, sideOffset = 8, style, children, ...props }, ref) => /* @__PURE__ */ jsx10(Portal3, { children: /* @__PURE__ */ jsx10(
  PopoverPrimitive.Content,
  {
    ref,
    sideOffset,
    onOpenAutoFocus: (event) => event.preventDefault(),
    onCloseAutoFocus: (event) => event.preventDefault(),
    className: "z-[9999]",
    ...props,
    children: /* @__PURE__ */ jsx10(
      "div",
      {
        className: clsx7(
          "flex flex-col",
          "cursor-default select-none rounded-2xl",
          "text-left text-sm text-ink",
          "bg-app-overlay",
          "border border-app-line",
          "shadow-2xl",
          className
        ),
        style,
        children
      }
    )
  }
) }));
Content3.displayName = PopoverPrimitive.Content.displayName;
var Popover = {
  Root: Root8,
  Trigger: Trigger3,
  Content: Content3,
  Anchor: Anchor2,
  Close: Close3,
  Portal: Portal3
};

// src/Tooltip.tsx
import * as TooltipPrimitive from "@radix-ui/react-tooltip";
import clsx8 from "clsx";
import { jsx as jsx11, jsxs as jsxs7 } from "react/jsx-runtime";
var Kbd = ({
  children,
  className
}) => /* @__PURE__ */ jsx11(
  "kbd",
  {
    className: clsx8(
      "flex h-4.5 items-center justify-center rounded-md border border-app-selected bg-app-selected/50 px-1.5 py-0.5 text-[10px] text-ink",
      className
    ),
    children
  }
);
var separateKeybinds = (keybinds) => {
  if (!keybinds) return;
  const arr = [];
  for (const i of keybinds) {
    if (i.length >= 2) {
      arr.push(i);
      continue;
    }
    for (const j of i) {
      arr.push(j);
    }
  }
  return arr;
};
var Tooltip = ({ position = "bottom", ...props }) => {
  return /* @__PURE__ */ jsxs7(
    TooltipPrimitive.Root,
    {
      disableHoverableContent: props.disableHoverableContent,
      children: [
        /* @__PURE__ */ jsx11(TooltipPrimitive.Trigger, { asChild: true, children: props.asChild ? props.children : /* @__PURE__ */ jsx11("span", { className: props.className, children: props.children }) }),
        /* @__PURE__ */ jsx11(TooltipPrimitive.Portal, { children: /* @__PURE__ */ jsxs7(
          TooltipPrimitive.Content,
          {
            side: position,
            align: props.align,
            sideOffset: props.sideOffset,
            alignOffset: props.alignOffset,
            className: clsx8(
              "TooltipContent z-[101] m-2 mt-1 flex max-w-[200px] select-text items-center gap-2 break-words rounded border border-app-line bg-app-box px-2 py-1 text-center text-xs text-ink",
              props.tooltipClassName,
              !props.label && "hidden"
            ),
            children: [
              /* @__PURE__ */ jsx11("div", { className: props.labelClassName, children: props.label }),
              props.keybinds && /* @__PURE__ */ jsx11("div", { className: "flex items-center justify-center gap-1", children: separateKeybinds(props.keybinds)?.map((k) => /* @__PURE__ */ jsx11(Kbd, { children: /* @__PURE__ */ jsx11("p", { children: k }) }, k.toString())) })
            ]
          }
        ) })
      ]
    }
  );
};
var TooltipProvider = TooltipPrimitive.Provider;
var TooltipRoot = TooltipPrimitive.Root;
var TooltipTrigger = TooltipPrimitive.Trigger;
var TooltipPortal = TooltipPrimitive.Portal;
var TooltipContent = ({
  className,
  sideOffset = 4,
  ...props
}) => /* @__PURE__ */ jsx11(
  TooltipPrimitive.Content,
  {
    sideOffset,
    className: clsx8(
      "z-50 overflow-hidden rounded-md border border-app-line bg-app-box px-3 py-1.5 text-sm text-ink shadow-md animate-in fade-in-0 zoom-in-95 data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=closed]:zoom-out-95 data-[side=bottom]:slide-in-from-top-2 data-[side=left]:slide-in-from-right-2 data-[side=right]:slide-in-from-left-2 data-[side=top]:slide-in-from-bottom-2",
      className
    ),
    ...props
  }
);
TooltipContent.displayName = "TooltipContent";

// src/Tabs.tsx
import * as TabsPrimitive from "@radix-ui/react-tabs";
import clsx9 from "clsx";
import { jsx as jsx12 } from "react/jsx-runtime";
var Root11 = ({
  className,
  ...props
}) => /* @__PURE__ */ jsx12(TabsPrimitive.Root, { className: clsx9("flex flex-col", className), ...props });
var Content6 = ({
  className,
  ...props
}) => /* @__PURE__ */ jsx12(TabsPrimitive.Content, { className: clsx9("outline-none", className), ...props });
var List2 = ({
  className,
  ...props
}) => /* @__PURE__ */ jsx12(
  TabsPrimitive.List,
  {
    className: clsx9(
      "flex flex-row items-center space-x-1 border-b border-app-line/70 p-2",
      className
    ),
    ...props
  }
);
var Trigger6 = ({
  className,
  ...props
}) => /* @__PURE__ */ jsx12(
  TabsPrimitive.Trigger,
  {
    className: clsx9(
      "rounded-full px-2 py-0.5 text-sm font-medium radix-state-active:bg-app-selected",
      className
    ),
    ...props
  }
);

// src/DropdownMenu.tsx
import * as DropdownMenuPrimitive from "@radix-ui/react-dropdown-menu";
import { Check as Check2, CaretRight } from "@phosphor-icons/react";
import clsx10 from "clsx";
import { forwardRef as forwardRef7 } from "react";
import { jsx as jsx13, jsxs as jsxs8 } from "react/jsx-runtime";
var Root13 = DropdownMenuPrimitive.Root;
var Trigger8 = DropdownMenuPrimitive.Trigger;
var Group2 = DropdownMenuPrimitive.Group;
var Portal6 = DropdownMenuPrimitive.Portal;
var Sub2 = DropdownMenuPrimitive.Sub;
var RadioGroup2 = DropdownMenuPrimitive.RadioGroup;
var Content8 = forwardRef7(({ className, sideOffset = 4, ...props }, ref) => /* @__PURE__ */ jsx13(DropdownMenuPrimitive.Portal, { children: /* @__PURE__ */ jsx13(
  DropdownMenuPrimitive.Content,
  {
    ref,
    sideOffset,
    className: clsx10(
      "z-50 min-w-[8rem] overflow-hidden rounded-md p-1",
      "border border-menu-line bg-menu/95 backdrop-blur-lg",
      "text-sm text-menu-ink shadow-xl shadow-menu-shade/30",
      "animate-in fade-in-0 zoom-in-95",
      className
    ),
    ...props
  }
) }));
Content8.displayName = DropdownMenuPrimitive.Content.displayName;
var Item4 = forwardRef7(({ className, inset, ...props }, ref) => /* @__PURE__ */ jsx13(
  DropdownMenuPrimitive.Item,
  {
    ref,
    className: clsx10(
      "relative flex cursor-default select-none items-center rounded-sm px-2 py-1.5 text-sm outline-none",
      "text-menu-ink transition-colors",
      "focus:bg-accent focus:text-white",
      "data-[disabled]:pointer-events-none data-[disabled]:opacity-50",
      inset && "pl-8",
      className
    ),
    ...props
  }
));
Item4.displayName = DropdownMenuPrimitive.Item.displayName;
var CheckboxItem2 = forwardRef7(({ className, children, checked, ...props }, ref) => /* @__PURE__ */ jsxs8(
  DropdownMenuPrimitive.CheckboxItem,
  {
    ref,
    className: clsx10(
      "relative flex cursor-default select-none items-center rounded-sm py-1.5 pl-8 pr-2 text-sm outline-none",
      "text-menu-ink transition-colors",
      "focus:bg-accent focus:text-white",
      "data-[disabled]:pointer-events-none data-[disabled]:opacity-50",
      className
    ),
    checked,
    ...props,
    children: [
      /* @__PURE__ */ jsx13("span", { className: "absolute left-2 flex size-3.5 items-center justify-center", children: /* @__PURE__ */ jsx13(DropdownMenuPrimitive.ItemIndicator, { children: /* @__PURE__ */ jsx13(Check2, { weight: "bold", className: "size-4" }) }) }),
      children
    ]
  }
));
CheckboxItem2.displayName = DropdownMenuPrimitive.CheckboxItem.displayName;
var RadioItem2 = forwardRef7(({ className, children, ...props }, ref) => /* @__PURE__ */ jsxs8(
  DropdownMenuPrimitive.RadioItem,
  {
    ref,
    className: clsx10(
      "relative flex cursor-default select-none items-center rounded-sm py-1.5 pl-8 pr-2 text-sm outline-none",
      "text-menu-ink transition-colors",
      "focus:bg-accent focus:text-white",
      "data-[disabled]:pointer-events-none data-[disabled]:opacity-50",
      className
    ),
    ...props,
    children: [
      /* @__PURE__ */ jsx13("span", { className: "absolute left-2 flex size-3.5 items-center justify-center", children: /* @__PURE__ */ jsx13(DropdownMenuPrimitive.ItemIndicator, { children: /* @__PURE__ */ jsx13("div", { className: "size-2 rounded-full bg-current" }) }) }),
      children
    ]
  }
));
RadioItem2.displayName = DropdownMenuPrimitive.RadioItem.displayName;
var Label3 = forwardRef7(({ className, inset, ...props }, ref) => /* @__PURE__ */ jsx13(
  DropdownMenuPrimitive.Label,
  {
    ref,
    className: clsx10(
      "px-2 py-1.5 text-xs font-semibold text-menu-ink",
      inset && "pl-8",
      className
    ),
    ...props
  }
));
Label3.displayName = DropdownMenuPrimitive.Label.displayName;
var Separator2 = forwardRef7(({ className, ...props }, ref) => /* @__PURE__ */ jsx13(
  DropdownMenuPrimitive.Separator,
  {
    ref,
    className: clsx10("mx-1 my-0.5 border-b border-menu-line", className),
    ...props
  }
));
Separator2.displayName = DropdownMenuPrimitive.Separator.displayName;
var SubTrigger2 = forwardRef7(({ className, inset, children, ...props }, ref) => /* @__PURE__ */ jsxs8(
  DropdownMenuPrimitive.SubTrigger,
  {
    ref,
    className: clsx10(
      "flex cursor-default select-none items-center rounded-sm px-2 py-1.5 text-sm outline-none",
      "text-menu-ink focus:bg-accent focus:text-white",
      "data-[state=open]:bg-accent data-[state=open]:text-white",
      inset && "pl-8",
      className
    ),
    ...props,
    children: [
      children,
      /* @__PURE__ */ jsx13(CaretRight, { weight: "fill", size: 12, className: "ml-auto" })
    ]
  }
));
SubTrigger2.displayName = DropdownMenuPrimitive.SubTrigger.displayName;
var SubContent2 = forwardRef7(({ className, ...props }, ref) => /* @__PURE__ */ jsx13(
  DropdownMenuPrimitive.SubContent,
  {
    ref,
    className: clsx10(
      "z-50 min-w-[8rem] overflow-hidden rounded-md p-1",
      "border border-menu-line bg-menu/95 backdrop-blur-lg",
      "text-sm text-menu-ink shadow-xl shadow-menu-shade/30",
      "animate-in fade-in-0 zoom-in-95",
      className
    ),
    ...props
  }
));
SubContent2.displayName = DropdownMenuPrimitive.SubContent.displayName;
var DropdownMenu = {
  Root: Root13,
  Trigger: Trigger8,
  Content: Content8,
  Item: Item4,
  CheckboxItem: CheckboxItem2,
  RadioItem: RadioItem2,
  Label: Label3,
  Separator: Separator2,
  Group: Group2,
  Portal: Portal6,
  Sub: Sub2,
  SubTrigger: SubTrigger2,
  SubContent: SubContent2,
  RadioGroup: RadioGroup2
};

// src/ContextMenu.tsx
import { CaretRight as CaretRight2, Check as Check3 } from "@phosphor-icons/react";
import * as RadixCM from "@radix-ui/react-context-menu";
import { cva as cva6 } from "class-variance-authority";
import clsx11 from "clsx";
import {
  createContext,
  Suspense,
  useContext
} from "react";
import { Fragment as Fragment3, jsx as jsx14, jsxs as jsxs9 } from "react/jsx-runtime";
var contextMenuClassNames = clsx11(
  "z-50 max-h-[calc(100vh-20px)] overflow-y-auto",
  "my-2 min-w-48 max-w-64 py-0.5",
  "cool-shadow bg-menu/95 backdrop-blur-lg",
  "border border-menu-line",
  "cursor-default select-none rounded-md",
  "animate-in fade-in"
);
var ContextMenuContext = createContext(null);
var useContextMenuContext = ({
  suspense
} = {}) => {
  const ctx = useContext(ContextMenuContext);
  if (suspense && ctx === null)
    throw new Error("ContextMenuContext.Provider not found!");
  return ctx;
};
var Root15 = ({
  trigger,
  children,
  className,
  onOpenChange,
  disabled,
  ...props
}) => {
  return /* @__PURE__ */ jsxs9(RadixCM.Root, { onOpenChange, children: [
    /* @__PURE__ */ jsx14(
      RadixCM.Trigger,
      {
        asChild: true,
        onContextMenu: (e) => disabled && e.preventDefault(),
        children: trigger
      }
    ),
    /* @__PURE__ */ jsx14(RadixCM.Portal, { children: /* @__PURE__ */ jsx14(
      RadixCM.Content,
      {
        className: clsx11(contextMenuClassNames, className),
        ...props,
        children: /* @__PURE__ */ jsx14(ContextMenuContext.Provider, { value: true, children })
      }
    ) })
  ] });
};
var contextMenuSeparatorClassNames = "border-b-menu-line mx-1 my-0.5 border-b";
var Separator4 = (props) => /* @__PURE__ */ jsx14(
  RadixCM.Separator,
  {
    className: clsx11(contextMenuSeparatorClassNames, props.className)
  }
);
var SubMenu = ({
  label,
  icon,
  className,
  ...props
}) => {
  return /* @__PURE__ */ jsxs9(RadixCM.Sub, { children: [
    /* @__PURE__ */ jsx14(RadixCM.SubTrigger, { className: contextMenuItemClassNames, children: /* @__PURE__ */ jsx14(ContextMenuDivItem, { rightArrow: true, ...{ label, icon } }) }),
    /* @__PURE__ */ jsx14(RadixCM.Portal, { children: /* @__PURE__ */ jsx14(Suspense, { fallback: null, children: /* @__PURE__ */ jsx14(
      RadixCM.SubContent,
      {
        className: clsx11(
          contextMenuClassNames,
          "-mt-2",
          className
        ),
        ...props
      }
    ) }) })
  ] });
};
var contextMenuItemStyles = cva6(
  [
    "flex max-h-fit min-h-[26px] items-center space-x-2 overflow-hidden rounded px-2",
    "text-sm text-menu-ink",
    "group-radix-highlighted:text-white",
    "group-radix-disabled:pointer-events-none group-radix-disabled:text-menu-ink/50",
    "group-radix-state-open:bg-accent group-radix-state-open:text-white"
  ],
  {
    variants: {
      variant: {
        default: "group-radix-highlighted:bg-accent",
        dull: "group-radix-highlighted:bg-app-selected/50 group-radix-highlighted:!text-menu-ink group-radix-state-open:bg-app-selected/50 group-radix-state-open:!text-ink",
        danger: [
          "text-red-600 dark:text-red-400",
          "group-radix-highlighted:text-white",
          "group-radix-highlighted:bg-red-500"
        ]
      }
    },
    defaultVariants: {
      variant: "default"
    }
  }
);
var contextMenuItemClassNames = "group py-0.5 outline-none px-1";
var Item6 = ({
  icon,
  label,
  children,
  keybind,
  variant,
  iconProps,
  onClick,
  ...props
}) => {
  return /* @__PURE__ */ jsx14(
    RadixCM.Item,
    {
      ...props,
      className: clsx11(contextMenuItemClassNames, props.className),
      onClick: (e) => !props.disabled && onClick?.(e),
      children: /* @__PURE__ */ jsx14(
        ContextMenuDivItem,
        {
          ...{ icon, iconProps, label, keybind, variant, children }
        }
      )
    }
  );
};
var CheckboxItem4 = ({
  variant,
  className,
  label,
  keybind,
  children,
  ...props
}) => {
  return /* @__PURE__ */ jsx14(
    RadixCM.CheckboxItem,
    {
      className: contextMenuItemClassNames,
      ...props,
      children: /* @__PURE__ */ jsxs9(ContextMenuDivItem, { variant, className, children: [
        /* @__PURE__ */ jsx14("span", { className: "flex size-3.5 items-center justify-center", children: /* @__PURE__ */ jsx14(RadixCM.ItemIndicator, { children: /* @__PURE__ */ jsx14(Check3, { weight: "bold" }) }) }),
        /* @__PURE__ */ jsx14(ItemInternals, { ...{ label, keybind, children } })
      ] })
    }
  );
};
var ContextMenuDivItem = ({
  variant,
  children,
  className,
  ...props
}) => /* @__PURE__ */ jsx14("div", { className: contextMenuItemStyles({ variant, className }), children: children || /* @__PURE__ */ jsx14(ItemInternals, { ...props }) });
var ItemInternals = ({
  icon,
  label,
  rightArrow,
  keybind,
  iconProps
}) => {
  const ItemIcon = icon;
  return /* @__PURE__ */ jsxs9(Fragment3, { children: [
    ItemIcon && /* @__PURE__ */ jsx14(ItemIcon, { size: 18, ...iconProps }),
    label && /* @__PURE__ */ jsx14("span", { className: "flex-1 truncate", children: label }),
    keybind && /* @__PURE__ */ jsx14("span", { className: "text-xs font-medium group-radix-highlighted:text-white", children: keybind }),
    rightArrow && /* @__PURE__ */ jsx14(
      CaretRight2,
      {
        weight: "fill",
        size: 12,
        className: "text-menu-faint group-radix-highlighted:text-white group-radix-state-open:text-white"
      }
    )
  ] });
};
var ContextMenu = {
  Root: Root15,
  Item: Item6,
  CheckboxItem: CheckboxItem4,
  Separator: Separator4,
  SubMenu
};

// src/Dropdown.tsx
var Dropdown_exports = {};
__export(Dropdown_exports, {
  Button: () => Button2,
  Item: () => Item7,
  Root: () => Root16,
  Section: () => Section,
  Separator: () => Separator5
});
import { Menu, Transition } from "@headlessui/react";
import { cva as cva7 } from "class-variance-authority";
import clsx12 from "clsx";
import { forwardRef as forwardRef8, Fragment as Fragment4 } from "react";
import { Fragment as Fragment5, jsx as jsx15, jsxs as jsxs10 } from "react/jsx-runtime";
var CaretDown = (props) => /* @__PURE__ */ jsxs10("svg", { width: "29", height: "18", viewBox: "0 0 29 18", fill: "none", ...props, children: [
  /* @__PURE__ */ jsx15(
    "path",
    {
      d: "M0.214203 3.80705L4.02126 0L17.9805 13.9592L14.1734 17.7663L0.214203 3.80705Z",
      fill: "#D9D9D9"
    }
  ),
  /* @__PURE__ */ jsx15(
    "path",
    {
      d: "M28.1356 3.80705L24.3286 0L10.3694 13.9592L14.1764 17.7663L28.1356 3.80705Z",
      fill: "#D9D9D9"
    }
  )
] });
var Section = ({
  className,
  ...props
}) => /* @__PURE__ */ jsx15("div", { className: clsx12("space-y-0.5 px-1 py-1", className), ...props });
var itemStyles = cva7(
  "group flex w-full shrink-0 grow items-center whitespace-nowrap rounded px-2 py-1 text-sm font-medium disabled:opacity-50",
  {
    variants: {
      selected: {
        true: "bg-accent text-white hover:!bg-accent",
        undefined: "hover:bg-sidebar-selected/40",
        false: "hover:bg-sidebar-selected/40"
      },
      active: {
        true: "bg-sidebar-selected/40 text-sidebar-ink"
      }
    }
  }
);
var itemIconStyles = cva7("mr-2 size-4", {
  variants: {}
});
var Item7 = ({
  to,
  className,
  icon: Icon2,
  children,
  ...props
}) => {
  const content = /* @__PURE__ */ jsxs10(Fragment5, { children: [
    Icon2 && /* @__PURE__ */ jsx15(
      Icon2,
      {
        weight: "bold",
        className: clsx12(itemIconStyles(props), props.iconClassName)
      }
    ),
    /* @__PURE__ */ jsx15("span", { className: "text-left", children })
  ] });
  return /* @__PURE__ */ jsx15(Menu.Item, { children: to ? /* @__PURE__ */ jsx15(
    "a",
    {
      ...props,
      href: to,
      className: clsx12(itemStyles(props), className),
      children: content
    }
  ) : /* @__PURE__ */ jsx15(
    "button",
    {
      ...props,
      className: clsx12(itemStyles(props), className),
      children: content
    }
  ) });
};
var Button2 = forwardRef8(
  ({ children, className, ...props }, ref) => {
    return /* @__PURE__ */ jsxs10(
      Button,
      {
        size: "sm",
        ref,
        className: clsx12("group flex text-left", className),
        ...props,
        children: [
          children,
          /* @__PURE__ */ jsx15("span", { className: "grow" }),
          /* @__PURE__ */ jsx15(
            CaretDown,
            {
              className: "ml-2 w-[12px] shrink-0 translate-y-px text-ink-dull transition-transform ui-open:-translate-y-px ui-open:rotate-180 group-radix-state-open:-translate-y-px group-radix-state-open:rotate-180",
              "aria-hidden": "true"
            }
          )
        ]
      }
    );
  }
);
Button2.displayName = "DropdownButton";
var Root16 = (props) => {
  return /* @__PURE__ */ jsx15("div", { className: props.className, children: /* @__PURE__ */ jsxs10(
    Menu,
    {
      as: "div",
      className: clsx12(
        "relative flex w-full justify-end text-left"
      ),
      children: [
        /* @__PURE__ */ jsx15(Menu.Button, { role: "button", as: "div", className: "outline-none", children: props.button }),
        /* @__PURE__ */ jsx15(
          Transition,
          {
            as: Fragment4,
            enter: "transition duration-100 ease-out",
            enterFrom: "transform -translate-y-2 opacity-0",
            enterTo: "transform translate-y-0 opacity-100",
            leave: "transition duration-75 ease-out",
            leaveFrom: "transform translate-y-0 opacity-100",
            leaveTo: "transform -translate-y-2 opacity-0",
            children: /* @__PURE__ */ jsx15(
              Menu.Items,
              {
                className: clsx12(
                  "absolute top-full z-50 w-full min-w-fit space-y-0.5 divide-y divide-menu-line rounded-md border border-menu-line bg-menu text-menu-ink shadow-xl shadow-menu-shade/30 focus:outline-none",
                  props.itemsClassName,
                  { "left-0": props.align === "left" },
                  { "right-0": props.align === "right" }
                ),
                children: props.children
              }
            )
          }
        )
      ]
    }
  ) });
};
var Separator5 = ({
  className,
  ...props
}) => /* @__PURE__ */ jsx15(
  "div",
  {
    className: clsx12("border-b border-menu-line", className),
    ...props
  }
);

// src/Select.tsx
import { Check as Check4 } from "@phosphor-icons/react";
import * as RS from "@radix-ui/react-select";
import { cva as cva8 } from "class-variance-authority";
import clsx13 from "clsx";
import { forwardRef as forwardRef9 } from "react";
import { jsx as jsx16, jsxs as jsxs11 } from "react/jsx-runtime";
var ChevronDouble = (props) => /* @__PURE__ */ jsxs11("svg", { width: "24", height: "24", viewBox: "0 0 24 24", fill: "none", ...props, children: [
  /* @__PURE__ */ jsx16(
    "path",
    {
      fillRule: "evenodd",
      clipRule: "evenodd",
      d: "M6.29289 14.2929C6.68342 13.9024 7.31658 13.9024 7.70711 14.2929L12 18.5858L16.2929 14.2929C16.6834 13.9024 17.3166 13.9024 17.7071 14.2929C18.0976 14.6834 18.0976 15.3166 17.7071 15.7071L12.7071 20.7071C12.3166 21.0976 11.6834 21.0976 11.2929 20.7071L6.29289 15.7071C5.90237 15.3166 5.90237 14.6834 6.29289 14.2929Z",
      fill: "currentColor"
    }
  ),
  /* @__PURE__ */ jsx16(
    "path",
    {
      fillRule: "evenodd",
      clipRule: "evenodd",
      d: "M6.29289 9.70711C6.68342 10.0976 7.31658 10.0976 7.70711 9.70711L12 5.41421L16.2929 9.70711C16.6834 10.0976 17.3166 10.0976 17.7071 9.70711C18.0976 9.31658 18.0976 8.68342 17.7071 8.29289L12.7071 3.29289C12.3166 2.90237 11.6834 2.90237 11.2929 3.29289L6.29289 8.29289C5.90237 8.68342 5.90237 9.31658 6.29289 9.70711Z",
      fill: "currentColor"
    }
  )
] });
var selectStyles = cva8(
  [
    "flex items-center justify-between whitespace-nowrap rounded-md border py-0.5 pl-3 pr-[10px] text-sm",
    "shadow-sm outline-none transition-[background-color,border-color,box-shadow] focus:ring-2",
    "text-ink radix-placeholder:text-ink-faint"
  ],
  {
    variants: {
      variant: {
        default: ["bg-app-input", "border-app-line"]
      },
      size: {
        sm: "h-[25px] text-xs font-normal",
        md: "h-[34px]",
        lg: "h-[38px]"
      }
    },
    defaultVariants: {
      variant: "default",
      size: "sm"
    }
  }
);
var Select = forwardRef9(
  (props, ref) => /* @__PURE__ */ jsx16("div", { className: props.containerClassName, ref, children: /* @__PURE__ */ jsxs11(
    RS.Root,
    {
      defaultValue: props.value,
      value: props.value,
      onValueChange: props.onChange,
      disabled: props.disabled,
      children: [
        /* @__PURE__ */ jsxs11(
          RS.Trigger,
          {
            className: selectStyles({
              size: props.size,
              className: props.className
            }),
            children: [
              /* @__PURE__ */ jsx16("span", { className: "truncate", children: /* @__PURE__ */ jsx16(RS.Value, { placeholder: props.placeholder }) }),
              /* @__PURE__ */ jsx16(RS.Icon, { className: "ml-2", children: /* @__PURE__ */ jsx16(ChevronDouble, { className: "text-ink-dull" }) })
            ]
          }
        ),
        /* @__PURE__ */ jsx16(RS.Portal, { children: /* @__PURE__ */ jsx16(RS.Content, { className: "z-[100] rounded-md border border-app-line bg-app-box shadow-2xl shadow-app-shade/20", children: /* @__PURE__ */ jsx16(RS.Viewport, { className: "p-1", children: props.children }) }) })
      ]
    }
  ) })
);
function SelectOption(props) {
  return /* @__PURE__ */ jsxs11(
    RS.Item,
    {
      value: props.value,
      defaultChecked: props.default,
      className: clsx13(
        "relative flex h-6 cursor-pointer select-none items-center rounded pl-6 pr-3",
        "text-sm text-ink radix-highlighted:text-white",
        "focus:outline-none radix-disabled:opacity-50 radix-highlighted:bg-accent"
      ),
      children: [
        /* @__PURE__ */ jsx16(RS.ItemText, { children: props.children }),
        /* @__PURE__ */ jsx16(RS.ItemIndicator, { className: "absolute left-1 inline-flex items-center", children: /* @__PURE__ */ jsx16(Check4, { className: "size-4" }) })
      ]
    }
  );
}
var SelectRoot = RS.Root;
var SelectGroup = RS.Group;
var SelectValue = RS.Value;
var SelectTrigger = ({
  className,
  children,
  ...props
}) => /* @__PURE__ */ jsxs11(
  RS.Trigger,
  {
    className: clsx13(
      "flex h-8 w-full items-center justify-between rounded-md border border-app-line bg-app-dark-box px-3 py-1.5 text-sm transition-colors placeholder:text-ink-faint focus:border-accent/50 focus:outline-none disabled:cursor-not-allowed disabled:opacity-50 [&>span]:line-clamp-1",
      className
    ),
    ...props,
    children: [
      children,
      /* @__PURE__ */ jsx16(RS.Icon, { asChild: true, children: /* @__PURE__ */ jsx16(ChevronDouble, { className: "size-4 opacity-50" }) })
    ]
  }
);
SelectTrigger.displayName = "SelectTrigger";
var SelectScrollUpButton = ({
  className,
  ...props
}) => /* @__PURE__ */ jsx16(
  RS.ScrollUpButton,
  {
    className: clsx13(
      "flex cursor-default items-center justify-center py-1",
      className
    ),
    ...props,
    children: /* @__PURE__ */ jsx16(ChevronDouble, { className: "size-4 rotate-180" })
  }
);
SelectScrollUpButton.displayName = "SelectScrollUpButton";
var SelectScrollDownButton = ({
  className,
  ...props
}) => /* @__PURE__ */ jsx16(
  RS.ScrollDownButton,
  {
    className: clsx13(
      "flex cursor-default items-center justify-center py-1",
      className
    ),
    ...props,
    children: /* @__PURE__ */ jsx16(ChevronDouble, { className: "size-4" })
  }
);
SelectScrollDownButton.displayName = "SelectScrollDownButton";
var SelectContent = ({
  className,
  children,
  position = "popper",
  ...props
}) => /* @__PURE__ */ jsx16(RS.Portal, { children: /* @__PURE__ */ jsxs11(
  RS.Content,
  {
    className: clsx13(
      "relative z-50 max-h-96 min-w-[8rem] overflow-hidden rounded-md border border-app-line bg-app-box text-ink shadow-md data-[state=open]:animate-in data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=open]:fade-in-0 data-[state=closed]:zoom-out-95 data-[state=open]:zoom-in-95 data-[side=bottom]:slide-in-from-top-2 data-[side=left]:slide-in-from-right-2 data-[side=right]:slide-in-from-left-2 data-[side=top]:slide-in-from-bottom-2",
      position === "popper" && "data-[side=bottom]:translate-y-1 data-[side=left]:-translate-x-1 data-[side=right]:translate-x-1 data-[side=top]:-translate-y-1",
      className
    ),
    position,
    ...props,
    children: [
      /* @__PURE__ */ jsx16(SelectScrollUpButton, {}),
      /* @__PURE__ */ jsx16(
        RS.Viewport,
        {
          className: clsx13(
            "p-1",
            position === "popper" && "h-[var(--radix-select-trigger-height)] w-full min-w-[var(--radix-select-trigger-width)]"
          ),
          children
        }
      ),
      /* @__PURE__ */ jsx16(SelectScrollDownButton, {})
    ]
  }
) });
SelectContent.displayName = "SelectContent";
var SelectLabel = ({ className, ...props }) => /* @__PURE__ */ jsx16(
  RS.Label,
  {
    className: clsx13(
      "py-1.5 pl-8 pr-2 text-sm font-semibold text-ink-dull",
      className
    ),
    ...props
  }
);
SelectLabel.displayName = "SelectLabel";
var SelectItem = ({
  className,
  children,
  ...props
}) => /* @__PURE__ */ jsxs11(
  RS.Item,
  {
    className: clsx13(
      "relative flex w-full cursor-default select-none items-center rounded-sm py-1.5 pl-8 pr-2 text-sm outline-none focus:bg-app-hover focus:text-ink data-[disabled]:pointer-events-none data-[disabled]:opacity-50",
      className
    ),
    ...props,
    children: [
      /* @__PURE__ */ jsx16("span", { className: "absolute left-2 flex h-3.5 w-3.5 items-center justify-center", children: /* @__PURE__ */ jsx16(RS.ItemIndicator, { children: /* @__PURE__ */ jsx16(Check4, { className: "size-4" }) }) }),
      /* @__PURE__ */ jsx16(RS.ItemText, { children })
    ]
  }
);
SelectItem.displayName = "SelectItem";
var SelectSeparator = ({
  className,
  ...props
}) => /* @__PURE__ */ jsx16(
  RS.Separator,
  {
    className: clsx13("-mx-1 my-1 h-px bg-app-divider", className),
    ...props
  }
);
SelectSeparator.displayName = "SelectSeparator";

// src/SelectPill.tsx
import { CaretDown as CaretDown2 } from "@phosphor-icons/react";
import { cva as cva9 } from "class-variance-authority";
import clsx14 from "clsx";
import { forwardRef as forwardRef10 } from "react";
import { jsx as jsx17, jsxs as jsxs12 } from "react/jsx-runtime";
var selectPillStyles = cva9(
  ["flex items-center gap-2 border font-medium transition-colors", "text-left"],
  {
    variants: {
      variant: {
        default: [
          "bg-app-overlay/80 border-app-line/50 text-ink-dull",
          "hover:bg-app-box hover:text-ink"
        ],
        sidebar: [
          "border-sidebar-line/30 bg-sidebar-box/20 text-sidebar-inkDull backdrop-blur-xl",
          "hover:bg-sidebar-box/30 hover:text-sidebar-ink"
        ],
        ghost: [
          "border-transparent bg-transparent text-ink-dull",
          "hover:bg-app-hover hover:text-ink"
        ]
      },
      size: {
        sm: "h-7 rounded-full px-2.5 text-[11px]",
        md: "h-8 rounded-full px-3 text-xs",
        lg: "h-9 rounded-full px-3.5 text-sm"
      }
    },
    defaultVariants: {
      variant: "default",
      size: "md"
    }
  }
);
var SelectPill = forwardRef10(
  ({ className, variant, size, hideCaret, children, ...props }, ref) => /* @__PURE__ */ jsxs12(
    "button",
    {
      ref,
      type: "button",
      className: clsx14(selectPillStyles({ variant, size }), className),
      ...props,
      children: [
        /* @__PURE__ */ jsx17("span", { className: "flex flex-1 items-center gap-1.5 truncate text-left", children }),
        !hideCaret && /* @__PURE__ */ jsx17(CaretDown2, { className: "size-3 shrink-0", weight: "bold" })
      ]
    }
  )
);
SelectPill.displayName = "SelectPill";

// src/Toast.tsx
import {
  CheckCircle,
  Info,
  Warning as Warning2,
  WarningCircle,
  X as X2
} from "@phosphor-icons/react";
import clsx15 from "clsx";
import {
  forwardRef as forwardRef11,
  useEffect as useEffect2,
  useState as useState4
} from "react";
import { toast as SonnerToast } from "sonner";
import { Toaster } from "sonner";
import { jsx as jsx18, jsxs as jsxs13 } from "react/jsx-runtime";
var TOAST_TIMEOUT = 4e3;
var actionButtonClassName = "!rounded !px-1.5 !py-0.5 !font-normal";
var toastClassName = clsx15(
  "w-full overflow-hidden rounded-md p-3 shadow-lg",
  "cursor-default select-none",
  "border border-app-line",
  "bg-app-dark-box/90 backdrop-blur",
  "text-sm text-ink-faint"
);
function isStructuredToastMessage(message) {
  return message !== null && typeof message === "object" && "title" in message;
}
var icons = {
  success: CheckCircle,
  error: WarningCircle,
  info: Info,
  warning: Warning2
};
var ToastComponent = forwardRef11(
  ({ closable = true, action, cancel, ...props }, ref) => {
    const message = typeof props.message === "function" ? props.message(props.id) : props.message;
    const title = isStructuredToastMessage(message) ? message.title : message;
    const body = isStructuredToastMessage(message) ? message.body : void 0;
    const typeIcon = (type) => {
      const IconComponent = icons[type];
      return /* @__PURE__ */ jsx18(
        IconComponent,
        {
          size: 16,
          weight: "fill",
          className: clsx15(
            type === "success" && "text-green-500",
            type === "error" && "text-red-500",
            type === "warning" && "text-yellow-500"
          )
        }
      );
    };
    return /* @__PURE__ */ jsxs13(
      "div",
      {
        ref,
        className: clsx15(
          "pointer-events-auto flex gap-2",
          body || action || cancel ? "items-start" : "items-center"
        ),
        children: [
          (props.icon || props.type) && /* @__PURE__ */ jsx18("div", { className: clsx15((body || action || cancel) && "mt-px"), children: props.icon || props.type && typeIcon(props.type) }),
          /* @__PURE__ */ jsxs13("div", { className: "flex grow flex-col", children: [
            title && /* @__PURE__ */ jsx18(
              "span",
              {
                className: "font-medium text-ink",
                style: { wordBreak: "break-word" },
                children: title
              }
            ),
            body && /* @__PURE__ */ jsx18("div", { className: "mt-0.5", style: { wordBreak: "break-word" }, children: body }),
            (action || cancel) && /* @__PURE__ */ jsxs13("div", { className: "mt-2.5 flex gap-2", children: [
              action && /* @__PURE__ */ jsx18(
                Button,
                {
                  variant: "accent",
                  onClick: () => {
                    action.onClick();
                    props.onClose?.({
                      id: props.id,
                      event: "on-action"
                    });
                    toast.dismiss(props.id);
                  },
                  className: clsx15(actionButtonClassName, action.className),
                  children: action.label
                }
              ),
              cancel && /* @__PURE__ */ jsx18(
                Button,
                {
                  variant: "gray",
                  onClick: () => {
                    if (typeof cancel === "object") cancel.onClick?.();
                    props.onClose?.({
                      id: props.id,
                      event: "on-cancel"
                    });
                    toast.dismiss(props.id);
                  },
                  className: clsx15(
                    actionButtonClassName,
                    typeof cancel === "object" ? cancel.className : null
                  ),
                  children: typeof cancel === "object" ? cancel.label : cancel
                }
              )
            ] })
          ] }),
          closable && /* @__PURE__ */ jsx18(
            "button",
            {
              className: "relative transition-colors before:absolute before:-inset-2 before:content-[''] hover:text-ink",
              onClick: () => {
                props.onDismiss?.(props.id);
                props.onClose?.({
                  id: props.id,
                  event: "on-dismiss"
                });
                toast.dismiss(props.id);
              },
              children: /* @__PURE__ */ jsx18(X2, { weight: "bold" })
            }
          )
        ]
      }
    );
  }
);
ToastComponent.displayName = "Toast";
var PromiseToast = ({
  showLoader = true,
  ...props
}) => {
  const [type, setType] = useState4();
  const [message, setMessage] = useState4(props.loading);
  useEffect2(() => {
    const resolve = async () => {
      try {
        const res = await (props.promise instanceof Promise ? props.promise : props.promise());
        const message2 = typeof props.success === "function" ? props.success(res) : props.success;
        setMessage(message2);
        setType("success");
      } catch (err) {
        const message2 = typeof props.error === "function" ? props.error(err) : props.error;
        setMessage(message2);
        setType("error");
      }
      setTimeout(() => {
        props.onAutoClose?.(props.id);
        props.onClose?.({ id: props.id, event: "on-auto-close" });
        toast.dismiss(props.id);
      }, props.duration || TOAST_TIMEOUT);
    };
    resolve();
  }, [
    props.id,
    props.promise,
    props.success,
    props.error,
    props.duration,
    props
  ]);
  return /* @__PURE__ */ jsx18(
    ToastComponent,
    {
      id: props.id,
      type,
      message,
      icon: !type && showLoader && (props.loader ?? /* @__PURE__ */ jsx18(Loader, { className: "!h-4 !w-4" })),
      closable: !!type,
      onDismiss: props.onDismiss,
      onClose: ({ id, event }) => {
        if (event === "on-action" || event === "on-cancel") return;
        props.onClose?.({ id, event });
      }
    }
  );
};
var renderToast = (message, {
  ref,
  type,
  icon,
  action,
  cancel,
  onDismiss,
  onClose,
  onAutoClose,
  className,
  ...options
} = {}) => {
  return SonnerToast.custom(
    (id) => /* @__PURE__ */ jsx18(
      ToastComponent,
      {
        id,
        ref,
        type,
        icon,
        message,
        action,
        cancel,
        onDismiss,
        onClose
      }
    ),
    {
      className: clsx15(toastClassName, className),
      onAutoClose: ({ id }) => {
        onAutoClose?.(id);
        onClose?.({ id, event: "on-auto-close" });
      },
      ...options
    }
  );
};
var renderCustomToast = (jsx43, { onAutoClose, className, ...options } = {}) => {
  return SonnerToast.custom(jsx43, {
    className: clsx15(toastClassName, className),
    onAutoClose: ({ id }) => onAutoClose?.(id),
    ...options
  });
};
var renderPromiseToast = (promise, {
  loading,
  success,
  error,
  onDismiss,
  onAutoClose,
  onClose,
  duration,
  className,
  loader,
  showLoader,
  ...options
}) => {
  return SonnerToast.custom(
    (id) => /* @__PURE__ */ jsx18(
      PromiseToast,
      {
        id,
        promise,
        loading,
        success,
        error,
        duration,
        onDismiss,
        onAutoClose,
        onClose,
        loader,
        showLoader
      }
    ),
    {
      duration: Infinity,
      className: clsx15(toastClassName, className),
      ...options
    }
  );
};
var toast = Object.assign(renderToast, {
  info: (message, options) => {
    return renderToast(message, { ...options, type: "info" });
  },
  success: (message, options) => {
    return renderToast(message, { ...options, type: "success" });
  },
  error: (message, options) => {
    return renderToast(message, { ...options, type: "error" });
  },
  warning: (message, options) => {
    return renderToast(message, { ...options, type: "warning" });
  },
  custom: renderCustomToast,
  promise: renderPromiseToast,
  dismiss: SonnerToast.dismiss
});

// src/Divider.tsx
import { jsx as jsx19 } from "react/jsx-runtime";
var Divider = ({
  className,
  ...props
}) => /* @__PURE__ */ jsx19(
  "div",
  {
    className: `bg-app-line/60 my-1 h-[1px] w-full ${className ?? ""}`,
    ...props
  }
);

// src/ProgressBar.tsx
import * as ProgressPrimitive from "@radix-ui/react-progress";
import clsx16 from "clsx";
import { memo } from "react";
import { jsx as jsx20 } from "react/jsx-runtime";
var ProgressBar = memo((props) => {
  const percentage = props.pending ? 0 : "percent" in props ? props.percent : Math.round(props.value / props.total * 100);
  if (props.pending) {
    return /* @__PURE__ */ jsx20("div", { className: "indeterminate-progress-bar h-1 bg-app-button", children: /* @__PURE__ */ jsx20("div", { className: "indeterminate-progress-bar__progress bg-accent" }) });
  }
  return /* @__PURE__ */ jsx20(
    ProgressPrimitive.Root,
    {
      value: percentage,
      className: clsx16(
        "h-1 w-[94%] overflow-hidden rounded-full bg-app-button"
      ),
      children: /* @__PURE__ */ jsx20(
        ProgressPrimitive.Indicator,
        {
          style: { width: `${percentage}%` },
          className: clsx16("h-full bg-accent duration-500 ease-in-out")
        }
      )
    }
  );
});
ProgressBar.displayName = "ProgressBar";

// src/CircularProgress.tsx
import clsx17 from "clsx";
import { useEffect as useEffect3, useState as useState5 } from "react";
import { jsx as jsx21, jsxs as jsxs14 } from "react/jsx-runtime";
var CircularProgress = ({
  radius,
  progress,
  steps: _steps = 100,
  cut: _cut = 0,
  rotate = -90,
  strokeWidth = 20,
  strokeColor = "indianred",
  fillColor = "none",
  strokeLinecap = "round",
  transition = ".3s ease",
  pointerRadius = 0,
  pointerStrokeWidth = 20,
  pointerStrokeColor = "indianred",
  pointerFillColor = "white",
  trackStrokeColor = "#e6e6e6",
  trackStrokeWidth = 20,
  trackStrokeLinecap = "round",
  trackTransition = ".3s ease",
  counterClockwise: _counterClockwise = false,
  inverse: _inverse = false,
  initialAnimation = false,
  initialAnimationDelay = 0,
  className = "",
  children
}) => {
  const [animationInitialized, setAnimationInitialized] = useState5(false);
  useEffect3(() => {
    if (initialAnimation) {
      const timeout = setTimeout(
        () => setAnimationInitialized(true),
        initialAnimationDelay
      );
      return () => clearTimeout(timeout);
    }
  }, [initialAnimation, initialAnimationDelay]);
  if (Number.isNaN(progress)) progress = 0;
  const getProgress = () => initialAnimation && !animationInitialized ? 0 : progress;
  const circumference = radius * 2 * Math.PI;
  const strokeDasharray = `${circumference} ${circumference}`;
  const strokeDashoffset = (100 - getProgress()) / 100 * circumference;
  const fullStrokeWidth = strokeWidth * 2;
  const svgSize = radius * 2 + fullStrokeWidth;
  const viewBox = `0 0 ${svgSize} ${svgSize}`;
  const center = radius + strokeWidth;
  return /* @__PURE__ */ jsxs14(
    "div",
    {
      className: clsx17("relative", className),
      style: {
        width: `${svgSize}px`,
        height: `${svgSize}px`
      },
      children: [
        /* @__PURE__ */ jsxs14(
          "svg",
          {
            width: svgSize,
            height: svgSize,
            viewBox,
            style: { transform: `rotate(${rotate}deg)` },
            children: [
              trackStrokeWidth > 0 && /* @__PURE__ */ jsx21(
                "circle",
                {
                  cx: center,
                  cy: center,
                  r: radius,
                  fill: "none",
                  stroke: trackStrokeColor,
                  strokeWidth: trackStrokeWidth,
                  strokeDasharray,
                  strokeLinecap: trackStrokeLinecap,
                  style: { transition: trackTransition },
                  className: "track-stroke"
                }
              ),
              strokeWidth > 0 && /* @__PURE__ */ jsx21(
                "circle",
                {
                  cx: center,
                  cy: center,
                  r: radius,
                  fill: fillColor,
                  stroke: strokeColor,
                  strokeWidth,
                  strokeDasharray,
                  strokeLinecap,
                  style: { transition, strokeDashoffset },
                  className: "progress-stroke"
                }
              ),
              pointerRadius > 0 && /* @__PURE__ */ jsx21(
                "circle",
                {
                  cx: radius,
                  cy: radius,
                  r: pointerRadius,
                  fill: pointerFillColor,
                  stroke: pointerStrokeColor,
                  strokeWidth: pointerStrokeWidth,
                  style: {
                    transform: `rotate(${rotate}deg)`,
                    transformOrigin: `${radius}px ${radius}px`
                  },
                  className: "pointer-stroke"
                }
              )
            ]
          }
        ),
        children
      ]
    }
  );
};

// src/SearchBar.tsx
import { MagnifyingGlass as MagnifyingGlass2, X as X3 } from "@phosphor-icons/react";
import clsx18 from "clsx";
import { forwardRef as forwardRef12, useState as useState6 } from "react";
import { jsx as jsx22, jsxs as jsxs15 } from "react/jsx-runtime";
var SearchBar = forwardRef12(
  ({ value, onChange, onClear, className, placeholder = "Search...", ...props }, ref) => {
    const [internalValue, setInternalValue] = useState6("");
    const currentValue = value !== void 0 ? value : internalValue;
    const handleChange = (e) => {
      const newValue = e.target.value;
      if (onChange) {
        onChange(newValue);
      } else {
        setInternalValue(newValue);
      }
    };
    const handleClear = () => {
      if (onChange) {
        onChange("");
      } else {
        setInternalValue("");
      }
      onClear?.();
    };
    return /* @__PURE__ */ jsxs15(
      "div",
      {
        className: clsx18(
          "flex h-8 items-center gap-2 px-3",
          "rounded-full backdrop-blur-xl",
          "border border-app-line/30 bg-app-overlay/80 hover:bg-app-box",
          "transition-colors focus-within:bg-sidebar-box/30",
          className
        ),
        children: [
          /* @__PURE__ */ jsx22(
            MagnifyingGlass2,
            {
              className: "size-[18px] flex-shrink-0 text-ink-faint",
              weight: "bold"
            }
          ),
          /* @__PURE__ */ jsx22(
            "input",
            {
              ref,
              type: "text",
              value: currentValue,
              onChange: handleChange,
              placeholder,
              className: clsx18(
                "min-w-0 flex-1 border-0 bg-transparent p-0 outline-none",
                "text-xs font-medium text-sidebar-ink placeholder:text-sidebar-inkFaint",
                "focus:border-0 focus:outline-none focus:ring-0"
              ),
              ...props
            }
          ),
          currentValue && /* @__PURE__ */ jsx22(
            "button",
            {
              type: "button",
              onClick: handleClear,
              "aria-label": "Clear Search",
              className: "flex-shrink-0 rounded-full p-0.5 transition-colors hover:bg-sidebar-selected/40",
              children: /* @__PURE__ */ jsx22(X3, { className: "size-3 text-sidebar-inkDull", weight: "bold" })
            }
          )
        ]
      }
    );
  }
);
SearchBar.displayName = "SearchBar";

// src/Shortcut.tsx
import clsx19 from "clsx";
import { jsx as jsx23 } from "react/jsx-runtime";
var Shortcut = (props) => {
  const { className, chars, ...rest } = props;
  return /* @__PURE__ */ jsx23(
    "kbd",
    {
      className: clsx19(
        "border border-b-2 px-1",
        "font-ink-dull rounded-md text-xs font-bold",
        "border-app-line dark:border-transparent",
        className
      ),
      ...rest,
      children: chars
    }
  );
};

// src/CircleButton.tsx
import { cva as cva10 } from "class-variance-authority";
import clsx20 from "clsx";
import { forwardRef as forwardRef13 } from "react";
import { jsx as jsx24, jsxs as jsxs16 } from "react/jsx-runtime";
var circleButtonStyles = cva10(
  [
    "flex items-center justify-center",
    "backdrop-blur-xl transition-[background-color,border-color,color,transform]",
    "border border-app-line/50",
    "rounded-full",
    "active:scale-95"
  ],
  {
    variants: {
      variant: {
        default: "bg-app-overlay/80 text-sidebar-inkDull hover:bg-app-box hover:text-sidebar-ink",
        active: "bg-app-overlay text-sidebar-ink",
        accent: "border-accent/30 bg-accent/20 text-accent",
        solid: "border-app-line bg-app-box text-ink-dull hover:bg-app-hover hover:text-ink"
      },
      size: {
        sm: "h-7 w-7",
        md: "h-8 w-8",
        lg: "h-9 w-9"
      }
    },
    defaultVariants: {
      variant: "default",
      size: "md"
    }
  }
);
var CircleButton = forwardRef13(
  ({
    icon: Icon2,
    active,
    activeAccent,
    variant,
    size,
    className,
    children,
    ...props
  }, ref) => {
    const resolvedVariant = variant ?? (active && activeAccent ? "accent" : active ? "active" : "default");
    return /* @__PURE__ */ jsxs16(
      "button",
      {
        ref,
        className: clsx20(
          circleButtonStyles({
            variant: resolvedVariant,
            size
          }),
          children && "w-auto gap-2 px-3",
          className
        ),
        ...props,
        children: [
          Icon2 && /* @__PURE__ */ jsx24(Icon2, { className: "size-[18px]", weight: "bold" }),
          children && /* @__PURE__ */ jsx24("span", { className: "text-xs font-medium", children })
        ]
      }
    );
  }
);
CircleButton.displayName = "CircleButton";

// src/CircleButtonGroup.tsx
import clsx21 from "clsx";
import { Children, cloneElement, isValidElement as isValidElement2 } from "react";
import { jsx as jsx25, jsxs as jsxs17 } from "react/jsx-runtime";
function CircleButtonGroup({
  children,
  className
}) {
  const childArray = Children.toArray(children);
  return /* @__PURE__ */ jsx25(
    "div",
    {
      className: clsx21(
        "flex h-8 items-center rounded-full",
        "border border-app-line/50 backdrop-blur-xl",
        "overflow-hidden bg-app-overlay/80",
        className
      ),
      children: childArray.map((child, index) => {
        if (!isValidElement2(child)) return child;
        return /* @__PURE__ */ jsxs17("div", { className: "relative flex items-center", children: [
          cloneElement(child, {
            className: clsx21(
              child.props.className,
              "!rounded-none !border-0 !backdrop-blur-none !bg-transparent",
              "hover:!bg-app-box"
            )
          }),
          index < childArray.length - 1 && /* @__PURE__ */ jsx25("div", { className: "h-5 w-px bg-sidebar-line/30" })
        ] }, index);
      })
    }
  );
}

// src/TabBar.tsx
import clsx22 from "clsx";
import { forwardRef as forwardRef14 } from "react";
import { LayoutGroup, motion } from "framer-motion";
import { jsx as jsx26, jsxs as jsxs18 } from "react/jsx-runtime";
var TabBar = forwardRef14(
  ({ layoutId = "tab-bar", trailing, className, children, ...props }, ref) => /* @__PURE__ */ jsxs18(
    "div",
    {
      ref,
      className: clsx22(
        "mx-2 flex h-9 shrink-0 items-center gap-1 rounded-full px-1",
        "bg-app-box/80 shadow-sm backdrop-blur-sm",
        className
      ),
      ...props,
      children: [
        /* @__PURE__ */ jsx26(LayoutGroup, { id: layoutId, children: /* @__PURE__ */ jsx26("div", { className: "flex min-w-0 flex-1 items-center gap-1", children }) }),
        trailing
      ]
    }
  )
);
TabBar.displayName = "TabBar";
var TabBarItem = forwardRef14(
  ({
    active,
    label,
    onClose,
    closable,
    closeIcon,
    className,
    ...props
  }, ref) => {
    const showClose = closable ?? !!onClose;
    return /* @__PURE__ */ jsxs18(
      "div",
      {
        className: clsx22(
          "group relative flex min-w-0 flex-1 items-center justify-center",
          className
        ),
        children: [
          /* @__PURE__ */ jsxs18(
            "button",
            {
              ref,
              className: clsx22(
                "relative flex w-full min-w-0 items-center justify-center rounded-full py-1.5 text-[13px]",
                active ? "text-ink" : "text-ink-dull hover:text-ink hover:bg-app-hover/50"
              ),
              ...props,
              children: [
                active && /* @__PURE__ */ jsx26(
                  motion.div,
                  {
                    layoutId: "activeTab",
                    className: "absolute inset-0 rounded-full bg-app-selected shadow-sm",
                    initial: false,
                    transition: {
                      type: "spring",
                      stiffness: 500,
                      damping: 35
                    }
                  }
                ),
                /* @__PURE__ */ jsx26("span", { className: "relative z-10 truncate px-6", children: label })
              ]
            }
          ),
          showClose && active && /* @__PURE__ */ jsx26(
            "button",
            {
              type: "button",
              "aria-label": "Close tab",
              onClick: (e) => {
                e.stopPropagation();
                onClose?.();
              },
              className: "z-10 flex cursor-pointer items-center justify-center rounded-full opacity-60 transition-[background-color,opacity] hover:bg-app-hover hover:opacity-100",
              style: { position: "absolute", left: 6, top: "50%", transform: "translateY(-50%)", width: 20, height: 20 },
              title: "Close tab",
              children: closeIcon ?? /* @__PURE__ */ jsx26(
                "svg",
                {
                  width: "10",
                  height: "10",
                  viewBox: "0 0 10 10",
                  fill: "none",
                  stroke: "currentColor",
                  strokeWidth: "1.5",
                  strokeLinecap: "round",
                  children: /* @__PURE__ */ jsx26("path", { d: "M2 2l6 6M8 2l-6 6" })
                }
              )
            }
          )
        ]
      }
    );
  }
);
TabBarItem.displayName = "TabBarItem";

// src/ShinyButton.tsx
import { ArrowCircleDown } from "@phosphor-icons/react";
import { cva as cva11 } from "class-variance-authority";
import clsx23 from "clsx";
import { useId } from "react";
import { Fragment as Fragment6, jsx as jsx27, jsxs as jsxs19 } from "react/jsx-runtime";
var shinyButtonVariants = cva11(
  [
    "noise with-rounded-2px-border-images inline-flex flex-row items-center justify-center gap-x-2 overflow-hidden",
    "bg-gradient-to-b from-[#42B2FD] to-[#0078F0] [--border-image:linear-gradient(to_bottom,hsl(200_100%_77%/100%),hsl(200_0%_100%/5%)75%)]",
    "will-change-transform will-change-[box-shadow]",
    "transition-[box-shadow,filter,transform] duration-200 ease-out",
    "cursor-pointer"
  ],
  {
    variants: {
      size: {
        default: "rounded-xl py-2 pe-4 ps-3",
        small: "rounded-xl px-3 py-1.5 text-sm"
      },
      glow: {
        lg: "shadow-[0_0px_2.5rem_hsl(207_100%_65%/50%)] hover:shadow-[0_0px_3.5rem_hsl(207_100%_65%/70%)] hover:brightness-105",
        sm: "shadow-[0_0.125rem_1.25rem_hsl(207_50%_65%/50%)] hover:shadow-[0_0.25rem_2rem_hsl(207_50%_65%/70%)] hover:brightness-105",
        none: ""
      }
    },
    defaultVariants: {
      size: "default",
      glow: "lg"
    }
  }
);
function ShinyButton({
  icon: IconComponent = /* @__PURE__ */ jsx27(ArrowCircleDown, { weight: "bold", size: 22 }),
  size,
  glow,
  children,
  className,
  href,
  ...props
}) {
  const id = useId();
  const iconSize = size === "small" ? 18 : 22;
  const content = /* @__PURE__ */ jsxs19(Fragment6, { children: [
    typeof IconComponent === "function" ? /* @__PURE__ */ jsx27(
      IconComponent,
      {
        weight: "bold",
        size: iconSize,
        fill: `url(#${id}-cta-gradient)`,
        children: /* @__PURE__ */ jsxs19(
          "linearGradient",
          {
            id: `${id}-cta-gradient`,
            x1: "0%",
            y1: "0%",
            x2: "0%",
            y2: "100%",
            children: [
              /* @__PURE__ */ jsx27(
                "stop",
                {
                  stopColor: "hsl(0 100% 100% / 100%)",
                  offset: "0%"
                }
              ),
              /* @__PURE__ */ jsx27(
                "stop",
                {
                  stopColor: "hsl(0 100% 100% / 70%)",
                  offset: "100%"
                }
              )
            ]
          }
        )
      }
    ) : IconComponent,
    /* @__PURE__ */ jsx27(
      "span",
      {
        className: clsx23(
          "text-center font-sans font-semibold leading-normal text-white drop-shadow-md will-change-transform",
          size === "small" ? "text-sm" : "text-base"
        ),
        children
      }
    )
  ] });
  const classes = clsx23(shinyButtonVariants({ size, glow }), className);
  if (href) {
    return /* @__PURE__ */ jsx27(
      "a",
      {
        ...props,
        href,
        className: classes,
        children: content
      }
    );
  }
  return /* @__PURE__ */ jsx27(
    "button",
    {
      ...props,
      className: classes,
      children: content
    }
  );
}

// src/ShinyToggle.tsx
import { motion as motion2 } from "framer-motion";
import { jsx as jsx28, jsxs as jsxs20 } from "react/jsx-runtime";
function ShinyToggle({
  value,
  onChange,
  options,
  className = ""
}) {
  return /* @__PURE__ */ jsx28(
    "div",
    {
      className: `inline-flex gap-1 rounded-full border border-app-line bg-app-box p-1 ${className}`,
      children: options.map((option) => /* @__PURE__ */ jsxs20(
        "button",
        {
          type: "button",
          onClick: () => onChange(option.value),
          className: "relative px-3 py-1.5 text-sm font-medium transition-colors",
          children: [
            value === option.value && /* @__PURE__ */ jsx28(
              motion2.div,
              {
                layoutId: "shinyToggleActive",
                className: "absolute inset-0 rounded-full border-2 border-[#42B2FD]/40 bg-gradient-to-b from-[#42B2FD]/80 to-[#0078F0]/80 shadow-[0_0_0.75rem_hsl(207_100%_65%/30%)]",
                transition: {
                  type: "spring",
                  bounce: 0.2,
                  duration: 0.6
                }
              }
            ),
            /* @__PURE__ */ jsxs20(
              "span",
              {
                className: `relative ${value === option.value ? "text-white" : "text-ink-dull"}`,
                children: [
                  option.label,
                  option.count !== void 0 && ` (${option.count})`
                ]
              }
            )
          ]
        },
        option.value
      ))
    }
  );
}

// src/InfoBanner.tsx
import { cva as cva12 } from "class-variance-authority";
import { jsx as jsx29, jsxs as jsxs21 } from "react/jsx-runtime";
var infoBannerStyles = cva12(
  "relative mb-6 overflow-hidden rounded-xl border-2",
  {
    variants: {
      variant: {
        info: "border-accent/30 bg-accent/10",
        gray: "border-app-line/20 bg-app-box/10",
        warning: "border-yellow-500/30 bg-yellow-500/10",
        danger: "border-red-500/30 bg-red-500/10"
      }
    },
    defaultVariants: {
      variant: "info"
    }
  }
);
var infoBannerIconStyles = cva12("mt-0.5", {
  variants: {
    variant: {
      info: "text-accent",
      gray: "text-ink-dull",
      warning: "text-yellow-500",
      danger: "text-red-500"
    }
  },
  defaultVariants: {
    variant: "info"
  }
});
function InfoBanner({ icon, children, variant }) {
  return /* @__PURE__ */ jsx29("div", { className: infoBannerStyles({ variant }), children: /* @__PURE__ */ jsx29("div", { className: "p-4", children: /* @__PURE__ */ jsxs21("div", { className: "flex items-start gap-3", children: [
    /* @__PURE__ */ jsx29("div", { className: infoBannerIconStyles({ variant }), children: icon }),
    /* @__PURE__ */ jsx29("div", { children })
  ] }) }) });
}
function InfoBannerText({ children }) {
  return /* @__PURE__ */ jsx29("p", { className: "text-sm text-ink", children });
}
function InfoBannerSubtext({ children }) {
  return /* @__PURE__ */ jsx29("p", { className: "mt-1 text-xs text-ink-dull", children });
}

// src/utils.tsx
import clsx24 from "clsx";
import React2 from "react";
var twFactory = (element) => ([newClassNames, ..._]) => React2.forwardRef(
  ({ className, ...props }, ref) => React2.createElement(element, {
    ...props,
    className: clsx24(newClassNames, className),
    ref
  })
);
var tw = new Proxy((() => {
}), {
  get: (_, property) => twFactory(property),
  apply: (_, __, [el]) => twFactory(el)
});

// src/Layout.tsx
var CardImpl = tw.div`flex px-4 py-2 text-sm border rounded-md shadow-sm border-app-line bg-app-box`;
var Card = CardImpl;
var GridLayoutImpl = tw.div`grid grid-cols-2 gap-3 lg:grid-cols-3`;
var GridLayout = GridLayoutImpl;

// src/Card.tsx
import { clsx as clsx25 } from "clsx";
import { forwardRef as forwardRef15 } from "react";
import { jsx as jsx30 } from "react/jsx-runtime";
var variantStyles = {
  default: "bg-app-box",
  dark: "bg-app"
};
var Card2 = forwardRef15(
  ({ className, variant = "default", ...props }, ref) => /* @__PURE__ */ jsx30(
    "div",
    {
      ref,
      className: clsx25(
        "rounded-2xl border border-app-line text-ink shadow-sm",
        variantStyles[variant],
        className
      ),
      ...props
    }
  )
);
Card2.displayName = "Card";
var CardHeader = forwardRef15(({ className, ...props }, ref) => /* @__PURE__ */ jsx30(
  "div",
  {
    ref,
    className: clsx25("flex flex-col space-y-1.5 p-6", className),
    ...props
  }
));
CardHeader.displayName = "CardHeader";
var CardTitle = forwardRef15(({ className, ...props }, ref) => /* @__PURE__ */ jsx30(
  "h3",
  {
    ref,
    className: clsx25(
      "text-2xl font-semibold leading-none tracking-tight text-ink",
      className
    ),
    ...props
  }
));
CardTitle.displayName = "CardTitle";
var CardDescription = forwardRef15(({ className, ...props }, ref) => /* @__PURE__ */ jsx30(
  "p",
  {
    ref,
    className: clsx25("text-sm text-ink-dull", className),
    ...props
  }
));
CardDescription.displayName = "CardDescription";
var CardContent = forwardRef15(({ className, ...props }, ref) => /* @__PURE__ */ jsx30("div", { ref, className: clsx25("p-6 pt-0", className), ...props }));
CardContent.displayName = "CardContent";
var CardFooter = forwardRef15(({ className, ...props }, ref) => /* @__PURE__ */ jsx30(
  "div",
  {
    ref,
    className: clsx25("flex items-center p-6 pt-0", className),
    ...props
  }
));
CardFooter.displayName = "CardFooter";

// src/Typography.tsx
var CategoryHeadingImpl = tw.h3`text-xs font-semibold text-ink-dull`;
var CategoryHeading = CategoryHeadingImpl;
var ScreenHeadingImpl = tw.h3`text-xl font-bold`;
var ScreenHeading = ScreenHeadingImpl;

// src/Resizable.tsx
import clsx26 from "clsx";
import {
  createContext as createContext2,
  useContext as useContext2,
  useEffect as useEffect4,
  useRef
} from "react";
import {
  useResizable
} from "react-resizable-layout";
import { jsx as jsx31 } from "react/jsx-runtime";
var ResizableContext = createContext2(null);
var useResizableContext = () => {
  const context = useContext2(ResizableContext);
  if (!context)
    throw new Error("ResizableContext.Provider not found!");
  return context;
};
var Resizable = ({ axis = "x", ...props }) => {
  const resizable = useResizable({ axis, ...props });
  const minSizeClientX = useRef(null);
  useEffect4(() => {
    if (!props.onCollapseChange || !resizable.isDragging || !props.min)
      return;
    const handleMouseMove = (e) => {
      if (minSizeClientX.current === null) {
        if (props.min === resizable.position && !props.collapsed) {
          minSizeClientX.current = e.clientX;
        }
        return;
      }
      const half = minSizeClientX.current / 2;
      if (e.clientX < half && !props.collapsed)
        props.onCollapseChange(true);
      else if (e.clientX > half && props.collapsed)
        props.onCollapseChange(false);
    };
    document.addEventListener("mousemove", handleMouseMove);
    return () => document.removeEventListener("mousemove", handleMouseMove);
  }, [
    props.min,
    props.collapsed,
    props.onCollapseChange,
    resizable.isDragging,
    resizable.position
  ]);
  useEffect4(() => {
    if (!resizable.isDragging) {
      minSizeClientX.current = null;
      document.body.style.cursor = "";
    } else {
      const cursor = axis === "x" ? "col-resize" : "row-resize";
      document.body.style.setProperty("cursor", cursor, "important");
    }
  }, [resizable.isDragging, axis]);
  return /* @__PURE__ */ jsx31(
    ResizableContext.Provider,
    {
      value: {
        ...resizable,
        size: props.collapsed ? 0 : resizable.position,
        collapsed: !!props.collapsed
      },
      children: props.children
    }
  );
};
var ResizablePanel = (props) => {
  const resizable = useResizableContext();
  return /* @__PURE__ */ jsx31("div", { style: { width: resizable.size }, ...props });
};
var ResizableHandle = ({
  className,
  ...props
}) => {
  const resizable = useResizableContext();
  return /* @__PURE__ */ jsx31(
    "div",
    {
      className: clsx26(
        "w-2",
        "aria-[orientation=horizontal]:cursor-row-resize aria-[orientation=vertical]:cursor-col-resize",
        "after:absolute after:inset-y-0 after:left-0.5 after:w-0.5 after:bg-accent after:opacity-0 after:transition-opacity hover:after:opacity-100",
        resizable.isDragging && "after:opacity-100",
        className
      ),
      ...props,
      ...resizable.separatorProps
    }
  );
};

// src/keys.ts
var ModifierKeys = /* @__PURE__ */ ((ModifierKeys2) => {
  ModifierKeys2["Alt"] = "Alt";
  ModifierKeys2["Shift"] = "Shift";
  ModifierKeys2["AltGraph"] = "AltGraph";
  ModifierKeys2["CapsLock"] = "CapsLock";
  ModifierKeys2["Control"] = "Control";
  ModifierKeys2["Fn"] = "Fn";
  ModifierKeys2["FnLock"] = "FnLock";
  ModifierKeys2["Meta"] = "Meta";
  ModifierKeys2["NumLock"] = "NumLock";
  ModifierKeys2["ScrollLock"] = "ScrollLock";
  ModifierKeys2["Symbol"] = "Symbol";
  ModifierKeys2["SymbolLock"] = "SymbolLock";
  return ModifierKeys2;
})(ModifierKeys || {});
var EditingKeys = /* @__PURE__ */ ((EditingKeys2) => {
  EditingKeys2["Backspace"] = "Backspace";
  EditingKeys2["Delete"] = "Delete";
  return EditingKeys2;
})(EditingKeys || {});
var UIKeys = /* @__PURE__ */ ((UIKeys2) => {
  UIKeys2["Escape"] = "Escape";
  return UIKeys2;
})(UIKeys || {});
var NavigationKeys = /* @__PURE__ */ ((NavigationKeys2) => {
  NavigationKeys2["ArrowUp"] = "ArrowUp";
  NavigationKeys2["ArrowDown"] = "ArrowDown";
  NavigationKeys2["ArrowLeft"] = "ArrowLeft";
  NavigationKeys2["ArrowRight"] = "ArrowRight";
  return NavigationKeys2;
})(NavigationKeys || {});
var modifierSymbols = {
  Alt: { macOS: "\u2325", Other: "Alt" },
  AltGraph: { macOS: "\u2325", Other: "Alt" },
  CapsLock: { Other: "\u21EA" },
  Control: { macOS: "\u2303", Other: "Ctrl" },
  Fn: { macOS: "fn", Other: "Fn" },
  FnLock: { macOS: "fn", Other: "Fn" },
  Meta: { macOS: "\u2318", Windows: "\u229E Win", Other: "Meta" },
  NumLock: { macOS: "\u21ED", Other: "Num" },
  ScrollLock: { macOS: "\u2913", Other: "ScrLk" },
  Shift: { Other: "Shift", macOS: "\u21E7" },
  Symbol: { macOS: "\u2384", Other: "Sym" },
  SymbolLock: { macOS: "\u2384", Other: "Sym" },
  Escape: { macOS: "\u238B", Other: "Esc" },
  Delete: { macOS: "\u2326", Other: "Del" },
  Backspace: { macOS: "\u232B", Other: "\u27F5" },
  ArrowUp: { Other: "\u2191" },
  ArrowDown: { Other: "\u2193" },
  ArrowLeft: { Other: "\u2190" },
  ArrowRight: { Other: "\u2192" }
};
var keySymbols = {
  " ": { Other: "\u2423" },
  Tab: { macOS: "\u21E5", Other: "\u2B7E" },
  Enter: { macOS: "", Other: "\u21B5" },
  Escape: { macOS: "\u238B", Other: "Esc" },
  Backspace: { macOS: "\u232B", Other: "\u27F5" },
  ArrowUp: { Other: "\u2191" },
  ArrowDown: { Other: "\u2193" },
  ArrowLeft: { Other: "\u2190" },
  ArrowRight: { Other: "\u2192" },
  Insert: { Other: "Ins" },
  Delete: { macOS: "\u2326", Other: "Del" },
  Home: { macOS: "", Other: "Home" },
  End: { macOS: "", Other: "End" },
  PageUp: { macOS: "\u21DE", Other: "PgUp" },
  PageDown: { macOS: "\u21DF", Other: "PgDn" },
  Shift: { macOS: "\u21E7", Other: "Shift" },
  PrintScreen: { Other: "PrtSc" },
  ScrollLock: { macOS: "\u2913", Other: "ScrLk" },
  Pause: { macOS: "\u2389", Other: "Pause" }
};

// src/SpaceItem.tsx
import { forwardRef as forwardRef16 } from "react";
import clsx27 from "clsx";
import { jsx as jsx32, jsxs as jsxs22 } from "react/jsx-runtime";
var SpaceItem = forwardRef16(
  function SpaceItem2({
    icon,
    label,
    active,
    color,
    iconElement,
    right,
    onClick,
    onContextMenu,
    className
  }, ref) {
    const isImageUrl = typeof icon === "string";
    const Icon2 = isImageUrl ? null : icon;
    return /* @__PURE__ */ jsxs22(
      "button",
      {
        ref,
        onClick,
        onContextMenu,
        className: clsx27(
          "flex w-full items-center gap-2 rounded-md px-2 py-1.5 text-sm font-medium transition-colors relative cursor-default",
          active ? "bg-sidebar-selected/30 text-sidebar-ink" : "text-sidebar-inkDull",
          className
        ),
        children: [
          iconElement ? iconElement : color ? /* @__PURE__ */ jsx32(
            "span",
            {
              className: "size-4 shrink-0 rounded-full",
              style: { backgroundColor: color }
            }
          ) : isImageUrl ? /* @__PURE__ */ jsx32("img", { src: icon, alt: "", width: 16, height: 16, className: "size-4 shrink-0" }) : Icon2 && /* @__PURE__ */ jsx32("span", { className: "shrink-0", children: /* @__PURE__ */ jsx32(Icon2, { size: 16, weight: "bold" }) }),
          /* @__PURE__ */ jsx32("span", { className: "flex-1 truncate text-left", children: label }),
          right
        ]
      }
    );
  }
);

// src/Badge.tsx
import { clsx as clsx28 } from "clsx";
import { cva as cva13 } from "class-variance-authority";
import { jsx as jsx33 } from "react/jsx-runtime";
var badgeVariants = cva13(
  "inline-flex items-center rounded-full border font-medium transition-colors",
  {
    variants: {
      variant: {
        default: "border border-app-line bg-app-button text-ink-dull",
        secondary: "border-transparent bg-app-box text-ink-dull",
        outline: "border-app-line text-ink-dull",
        accent: "border-transparent bg-accent/10 text-accent",
        success: "border-transparent bg-status-success/10 text-status-success",
        warning: "border-transparent bg-status-warning/10 text-status-warning",
        error: "border-transparent bg-status-error/10 text-status-error",
        destructive: "border-transparent bg-status-error/10 text-status-error",
        info: "border-transparent bg-status-info/10 text-status-info"
      },
      size: {
        sm: "px-1.5 py-px text-[9px]",
        default: "px-2 py-0.5 text-xs",
        md: "px-2.5 py-1 text-xs"
      }
    },
    defaultVariants: {
      variant: "default",
      size: "default"
    }
  }
);
function Badge({ className, variant, size, ...props }) {
  return /* @__PURE__ */ jsx33(
    "div",
    {
      className: clsx28(badgeVariants({ variant, size }), className),
      ...props
    }
  );
}

// src/Banner.tsx
import { clsx as clsx29 } from "clsx";
import { cva as cva14 } from "class-variance-authority";
import { forwardRef as forwardRef17 } from "react";
import { Warning as Warning3, CheckCircle as CheckCircle2, Info as Info2, XCircle } from "@phosphor-icons/react";
import { jsx as jsx34, jsxs as jsxs23 } from "react/jsx-runtime";
var bannerVariants = cva14(
  "relative flex w-full items-center gap-3 rounded-lg border px-4 py-3 text-sm",
  {
    variants: {
      variant: {
        default: "border-accent/20 bg-accent/10 text-accent",
        info: "border-status-info/20 bg-status-info/10 text-status-info",
        success: "border-status-success/20 bg-status-success/10 text-status-success",
        warning: "border-status-warning/20 bg-status-warning/10 text-status-warning",
        error: "border-status-error/20 bg-status-error/10 text-status-error"
      }
    },
    defaultVariants: {
      variant: "default"
    }
  }
);
var dotVariants = cva14("size-2 rounded-full", {
  variants: {
    variant: {
      default: "bg-accent",
      info: "bg-status-info",
      success: "bg-status-success",
      warning: "bg-status-warning",
      error: "bg-status-error"
    }
  },
  defaultVariants: {
    variant: "default"
  }
});
var Banner = forwardRef17(
  ({ className, variant, showDot = true, children, ...props }, ref) => {
    const icons2 = {
      default: Info2,
      info: Info2,
      success: CheckCircle2,
      warning: Warning3,
      error: XCircle
    };
    const Icon2 = icons2[variant || "default"];
    return /* @__PURE__ */ jsxs23(
      "div",
      {
        ref,
        role: "alert",
        className: clsx29(bannerVariants({ variant }), className),
        ...props,
        children: [
          showDot && /* @__PURE__ */ jsx34("span", { className: clsx29(dotVariants({ variant })) }),
          /* @__PURE__ */ jsx34(Icon2, { className: "size-4 shrink-0" }),
          /* @__PURE__ */ jsx34("div", { className: "flex-1", children })
        ]
      }
    );
  }
);
Banner.displayName = "Banner";

// src/ToggleGroup.tsx
import * as ToggleGroupPrimitive from "@radix-ui/react-toggle-group";
import { clsx as clsx30 } from "clsx";
import { forwardRef as forwardRef18 } from "react";
import { jsx as jsx35, jsxs as jsxs24 } from "react/jsx-runtime";
var ToggleGroup = forwardRef18(
  ({ options, value, onChange, className, disabled, itemClassName }, ref) => {
    return /* @__PURE__ */ jsx35(
      ToggleGroupPrimitive.Root,
      {
        ref,
        type: "single",
        value,
        onValueChange: (val) => val && onChange(val),
        disabled,
        className: clsx30(
          "inline-flex items-center rounded-md border border-app-line bg-app-box p-1",
          className
        ),
        children: options.map((option) => /* @__PURE__ */ jsxs24(
          ToggleGroupPrimitive.Item,
          {
            value: option.value,
            title: option.title,
            className: clsx30(
              "inline-flex items-center gap-1.5 rounded-sm px-3 py-1 text-sm font-medium",
              "transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-accent",
              "data-[state=on]:bg-app data-[state=on]:text-ink text-ink-dull hover:text-ink",
              disabled && "cursor-not-allowed opacity-50",
              itemClassName,
              option.className
            ),
            children: [
              option.icon && /* @__PURE__ */ jsx35(option.icon, { className: "size-4" }),
              option.label
            ]
          },
          option.value
        ))
      }
    );
  }
);
ToggleGroup.displayName = "ToggleGroup";

// src/Collapsible.tsx
import * as CollapsiblePrimitive from "@radix-ui/react-collapsible";
import { clsx as clsx31 } from "clsx";
import {
  forwardRef as forwardRef19
} from "react";
import { jsx as jsx36 } from "react/jsx-runtime";
var Collapsible = CollapsiblePrimitive.Root;
var CollapsibleTrigger2 = forwardRef19(({ className, children, ...props }, ref) => /* @__PURE__ */ jsx36(
  CollapsiblePrimitive.CollapsibleTrigger,
  {
    ref,
    className: clsx31(
      "flex items-center justify-between w-full",
      "hover:bg-app-hover rounded-md transition-colors",
      "focus:outline-none focus:ring-2 focus:ring-accent",
      className
    ),
    ...props,
    children
  }
));
CollapsibleTrigger2.displayName = CollapsiblePrimitive.CollapsibleTrigger.displayName;
var CollapsibleTriggerExp = CollapsibleTrigger2;
var CollapsibleContent2 = forwardRef19(({ className, children, ...props }, ref) => /* @__PURE__ */ jsx36(
  CollapsiblePrimitive.CollapsibleContent,
  {
    ref,
    className: clsx31(
      "overflow-hidden",
      "data-[state=closed]:animate-collapsible-up data-[state=open]:animate-collapsible-down",
      className
    ),
    ...props,
    children
  }
));
CollapsibleContent2.displayName = CollapsiblePrimitive.CollapsibleContent.displayName;
var CollapsibleContentExp = CollapsibleContent2;

// src/NumberStepper.tsx
import { clsx as clsx32 } from "clsx";
import { forwardRef as forwardRef20, useState as useState7, useCallback, useEffect as useEffect5, useRef as useRef2 } from "react";
import { Minus, Plus } from "@phosphor-icons/react";
import { jsx as jsx37, jsxs as jsxs25 } from "react/jsx-runtime";
var NumberStepper = forwardRef20(
  ({
    value,
    onChange,
    min = 0,
    max,
    step = 1,
    allowFloat = false,
    disabled = false,
    className,
    showProgress = false,
    label,
    description,
    suffix
  }, ref) => {
    const [editing, setEditing] = useState7(false);
    const [inputValue, setInputValue] = useState7("");
    const inputRef = useRef2(null);
    useEffect5(() => {
      if (editing && inputRef.current) {
        inputRef.current.focus();
        inputRef.current.select();
      }
    }, [editing]);
    const clampValue = useCallback(
      (v) => {
        let clamped = Math.max(min, v);
        if (max !== void 0) clamped = Math.min(max, clamped);
        return clamped;
      },
      [min, max]
    );
    const handleDecrement = () => {
      const newValue = allowFloat ? clampValue(value - step) : clampValue(Math.floor(value - step));
      onChange(newValue);
    };
    const handleIncrement = () => {
      const newValue = allowFloat ? clampValue(value + step) : clampValue(Math.ceil(value + step));
      onChange(newValue);
    };
    const commitInput = () => {
      const parsed = allowFloat ? parseFloat(inputValue) : parseInt(inputValue, 10);
      if (!isNaN(parsed)) {
        onChange(clampValue(parsed));
      }
      setEditing(false);
    };
    const progress = max !== void 0 ? (value - min) / (max - min) * 100 : void 0;
    return /* @__PURE__ */ jsxs25("div", { ref, className: clsx32("flex flex-col gap-1", className), children: [
      (label || description) && /* @__PURE__ */ jsxs25("div", { className: "flex flex-col gap-0.5", children: [
        label && /* @__PURE__ */ jsx37("span", { className: "text-sm font-medium text-ink", children: label }),
        description && /* @__PURE__ */ jsx37("span", { className: "text-xs text-ink-faint", children: description })
      ] }),
      /* @__PURE__ */ jsxs25("div", { className: "flex w-fit items-center bg-app-dark-box border border-app-line rounded-lg p-1 gap-2", children: [
        /* @__PURE__ */ jsx37(
          "button",
          {
            type: "button",
            onClick: handleDecrement,
            disabled: disabled || value <= min,
            className: clsx32(
              "flex h-8 w-8 items-center justify-center rounded-md border border-app-line bg-app-box",
              "hover:bg-app-hover disabled:opacity-50 disabled:cursor-not-allowed",
              "focus:outline-none focus:ring-2 focus:ring-accent"
            ),
            children: /* @__PURE__ */ jsx37(Minus, { className: "size-4 text-ink" })
          }
        ),
        editing ? /* @__PURE__ */ jsx37(
          "input",
          {
            ref: inputRef,
            type: "text",
            inputMode: "decimal",
            value: inputValue,
            onChange: (e) => setInputValue(e.target.value),
            onBlur: commitInput,
            onKeyDown: (e) => {
              if (e.key === "Enter") commitInput();
              if (e.key === "Escape") setEditing(false);
            },
            className: "min-w-[3rem] w-[4rem] bg-transparent text-center text-sm font-medium text-ink outline-none"
          }
        ) : /* @__PURE__ */ jsxs25(
          "span",
          {
            className: "min-w-[3rem] text-center text-sm font-medium text-ink cursor-text select-none",
            onDoubleClick: () => {
              if (!disabled) {
                setInputValue(String(allowFloat ? value.toFixed(1) : value));
                setEditing(true);
              }
            },
            children: [
              allowFloat ? value.toFixed(1) : value,
              suffix
            ]
          }
        ),
        /* @__PURE__ */ jsx37(
          "button",
          {
            type: "button",
            onClick: handleIncrement,
            disabled: disabled || max !== void 0 && value >= max,
            className: clsx32(
              "flex h-8 w-8 items-center justify-center rounded-md border border-app-line bg-app-box",
              "hover:bg-app-hover disabled:opacity-50 disabled:cursor-not-allowed",
              "focus:outline-none focus:ring-2 focus:ring-accent"
            ),
            children: /* @__PURE__ */ jsx37(Plus, { className: "size-4 text-ink" })
          }
        )
      ] }),
      showProgress && progress !== void 0 && /* @__PURE__ */ jsx37("div", { className: "h-1 w-full overflow-hidden rounded-full bg-app-line", children: /* @__PURE__ */ jsx37(
        "div",
        {
          className: "h-full bg-accent transition-[width] duration-200",
          style: { width: `${progress}%` }
        }
      ) })
    ] });
  }
);
NumberStepper.displayName = "NumberStepper";

// src/FilterButton.tsx
import { clsx as clsx33 } from "clsx";
import { forwardRef as forwardRef21 } from "react";
import { jsx as jsx38 } from "react/jsx-runtime";
var FilterButton = forwardRef21(
  ({ className, active, label, children, ...props }, ref) => {
    return /* @__PURE__ */ jsx38(
      "button",
      {
        ref,
        type: "button",
        className: clsx33(
          "inline-flex items-center justify-center rounded-md px-2.5 py-1 text-xs font-medium transition-colors",
          active ? "bg-accent text-white" : "bg-app-box text-ink-dull hover:bg-app-hover hover:text-ink",
          className
        ),
        ...props,
        children: label ?? children
      }
    );
  }
);
FilterButton.displayName = "FilterButton";

// src/OptionList.tsx
import { cva as cva15 } from "class-variance-authority";
import { clsx as clsx34 } from "clsx";
import { forwardRef as forwardRef22 } from "react";
import { jsx as jsx39 } from "react/jsx-runtime";
var optionListItemStyles = cva15(
  ["w-full cursor-pointer text-left font-medium", "text-ink"],
  {
    variants: {
      size: {
        sm: "rounded-lg px-2.5 py-1 text-[11px]",
        md: "rounded-lg px-3 py-1.5 text-xs",
        lg: "rounded-lg px-3.5 py-2 text-sm"
      },
      selected: {
        true: "bg-app-selected text-ink",
        false: "hover:bg-app-hover hover:text-ink"
      }
    },
    defaultVariants: {
      size: "md",
      selected: false
    }
  }
);
var OptionList = forwardRef22(
  ({ className, ...props }, ref) => {
    return /* @__PURE__ */ jsx39("div", { ref, className: clsx34("space-y-0.5", className), ...props });
  }
);
OptionList.displayName = "OptionList";
var OptionListItem = forwardRef22(
  ({ className, selected, size, type = "button", ...props }, ref) => {
    return /* @__PURE__ */ jsx39(
      "button",
      {
        ref,
        type,
        className: clsx34(
          optionListItemStyles({ size, selected: !!selected }),
          className
        ),
        ...props
      }
    );
  }
);
OptionListItem.displayName = "OptionListItem";

// src/SelectTriggerButton.tsx
import { CaretDown as CaretDown3 } from "@phosphor-icons/react";
import { clsx as clsx35 } from "clsx";
import { forwardRef as forwardRef23 } from "react";
import { jsx as jsx40, jsxs as jsxs26 } from "react/jsx-runtime";
var SelectTriggerButton = forwardRef23(({ className, children, placeholder, ...props }, ref) => {
  return /* @__PURE__ */ jsxs26(
    "button",
    {
      ref,
      type: "button",
      className: clsx35(
        "flex h-9 min-w-[220px] items-center gap-2 rounded-full border border-app-line bg-app-box px-4 text-left text-sm font-medium text-ink-dull transition-colors",
        "hover:bg-app-hover hover:text-ink",
        "disabled:cursor-not-allowed disabled:opacity-60",
        className
      ),
      ...props,
      children: [
        /* @__PURE__ */ jsx40("span", { className: "flex-1 truncate text-left", children: children || placeholder }),
        /* @__PURE__ */ jsx40(CaretDown3, { className: "size-3.5 shrink-0", weight: "bold" })
      ]
    }
  );
});
SelectTriggerButton.displayName = "SelectTriggerButton";

// src/useEndAnchoredVirtualizer.ts
import {
  useVirtualizer
} from "@tanstack/react-virtual";
function useEndAnchoredVirtualizer(options) {
  return useVirtualizer({
    anchorTo: "end",
    followOnAppend: true,
    ...options
  });
}

// src/VirtualList.tsx
import clsx36 from "clsx";
import {
  useEffect as useEffect6,
  useImperativeHandle,
  useRef as useRef3
} from "react";
import { jsx as jsx41 } from "react/jsx-runtime";
function VirtualList({
  items,
  getItemKey,
  estimateSize,
  renderItem,
  overscan = 8,
  anchorTo = "end",
  followOnAppend = true,
  scrollEndThreshold,
  onReachStart,
  handleRef,
  className,
  innerClassName
}) {
  const scrollRef = useRef3(null);
  const virtualizer = useEndAnchoredVirtualizer({
    count: items.length,
    getScrollElement: () => scrollRef.current,
    estimateSize,
    getItemKey,
    overscan,
    anchorTo,
    followOnAppend,
    ...scrollEndThreshold !== void 0 && { scrollEndThreshold }
  });
  useImperativeHandle(
    handleRef,
    () => ({
      virtualizer,
      scrollToEnd: (opts) => virtualizer.scrollToEnd(opts ?? {}),
      scrollToIndex: (index, opts) => virtualizer.scrollToIndex(index, opts),
      isAtEnd: () => virtualizer.isAtEnd(),
      getDistanceFromEnd: () => virtualizer.getDistanceFromEnd()
    }),
    [virtualizer]
  );
  const range = virtualizer.range;
  const startIndex = range?.startIndex ?? -1;
  const reachedStartRef = useRef3(false);
  useEffect6(() => {
    if (!onReachStart) return;
    if (startIndex === 0 && items.length > 0) {
      if (!reachedStartRef.current) {
        reachedStartRef.current = true;
        onReachStart();
      }
    } else {
      reachedStartRef.current = false;
    }
  }, [startIndex, items.length, onReachStart]);
  const virtualItems = virtualizer.getVirtualItems();
  return /* @__PURE__ */ jsx41(
    "div",
    {
      ref: scrollRef,
      className: clsx36("relative h-full w-full overflow-auto", className),
      children: /* @__PURE__ */ jsx41(
        "div",
        {
          className: clsx36("relative w-full", innerClassName),
          style: { height: `${virtualizer.getTotalSize()}px` },
          children: virtualItems.map((virtual) => /* @__PURE__ */ jsx41(
            "div",
            {
              "data-index": virtual.index,
              ref: virtualizer.measureElement,
              className: "absolute left-0 top-0 w-full",
              style: { transform: `translateY(${virtual.start}px)` },
              children: renderItem(items[virtual.index], virtual.index)
            },
            virtual.key
          ))
        }
      )
    }
  );
}

// src/JumpToEndButton.tsx
import { ArrowDown } from "@phosphor-icons/react";
import clsx37 from "clsx";
import { useEffect as useEffect7, useState as useState8 } from "react";
import { Fragment as Fragment7, jsx as jsx42, jsxs as jsxs27 } from "react/jsx-runtime";
function JumpToEndButton({
  virtualizer,
  className,
  children,
  "aria-label": ariaLabel = "Jump to latest"
}) {
  const [visible, setVisible] = useState8(false);
  useEffect7(() => {
    if (!virtualizer) return;
    const el = virtualizer.scrollElement;
    if (!el) return;
    const update = () => setVisible(!virtualizer.isAtEnd());
    update();
    el.addEventListener("scroll", update, { passive: true });
    return () => el.removeEventListener("scroll", update);
  }, [virtualizer, virtualizer?.scrollElement]);
  if (!virtualizer || !visible) return null;
  return /* @__PURE__ */ jsx42(
    "button",
    {
      type: "button",
      onClick: () => virtualizer.scrollToEnd({ behavior: "smooth" }),
      "aria-label": ariaLabel,
      className: clsx37(
        "absolute bottom-4 left-1/2 z-10 flex -translate-x-1/2 items-center gap-1.5 rounded-full border border-app-line bg-app-box px-3 py-1.5 text-xs text-ink shadow-md transition-opacity hover:bg-app-selected",
        className
      ),
      children: children ?? /* @__PURE__ */ jsxs27(Fragment7, { children: [
        /* @__PURE__ */ jsx42(ArrowDown, { weight: "bold", className: "h-3 w-3" }),
        /* @__PURE__ */ jsx42("span", { children: "Jump to latest" })
      ] })
    }
  );
}
export {
  Badge,
  Banner,
  Button,
  Card2 as Card,
  CardContent,
  CardDescription,
  CardFooter,
  CardHeader,
  CardTitle,
  CategoryHeading,
  CheckBox,
  CheckboxIndicator,
  CheckboxRoot,
  CircleButton,
  CircleButtonGroup,
  CircularProgress,
  Collapsible,
  CollapsibleContentExp as CollapsibleContent,
  CollapsibleTriggerExp as CollapsibleTrigger,
  ContextMenu,
  ContextMenuDivItem,
  Dialog,
  DialogClose,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogOverlay,
  DialogPortal,
  DialogRoot,
  DialogTitle,
  DialogTrigger,
  Dialogs,
  Divider,
  Dropdown_exports as Dropdown,
  DropdownMenu,
  CheckboxItem2 as DropdownMenuCheckboxItem,
  Content8 as DropdownMenuContent,
  Group2 as DropdownMenuGroup,
  Item4 as DropdownMenuItem,
  Label3 as DropdownMenuLabel,
  Portal6 as DropdownMenuPortal,
  RadioGroup2 as DropdownMenuRadioGroup,
  RadioItem2 as DropdownMenuRadioItem,
  Root13 as DropdownMenuRoot,
  Separator2 as DropdownMenuSeparator,
  Sub2 as DropdownMenuSub,
  SubContent2 as DropdownMenuSubContent,
  SubTrigger2 as DropdownMenuSubTrigger,
  Trigger8 as DropdownMenuTrigger,
  EditingKeys,
  ErrorMessage,
  FilterButton,
  Form,
  GridLayout,
  InfoBanner,
  InfoBannerSubtext,
  InfoBannerText,
  Input,
  JumpToEndButton,
  Kbd,
  Label,
  Card as LayoutCard,
  Loader,
  ModifierKeys,
  NavigationKeys,
  NumberStepper,
  OptionList,
  OptionListItem,
  PasswordInput,
  Popover,
  Anchor2 as PopoverAnchor,
  Close3 as PopoverClose,
  Content3 as PopoverContent,
  Portal3 as PopoverPortal,
  Root8 as PopoverRoot,
  Trigger3 as PopoverTrigger,
  ProgressBar,
  Item2 as RadioGroupItem,
  Root5 as RadioGroupRoot,
  RadixCheckbox,
  Resizable,
  ResizableHandle,
  ResizablePanel,
  ScreenHeading,
  SearchBar,
  SearchInput,
  Select,
  SelectContent,
  SelectGroup,
  SelectItem,
  SelectLabel,
  SelectOption,
  SelectPill,
  SelectRoot,
  SelectScrollDownButton,
  SelectScrollUpButton,
  SelectSeparator,
  SelectTrigger,
  SelectTriggerButton,
  SelectValue,
  ShinyButton,
  ShinyToggle,
  Shortcut,
  Slider,
  SpaceItem,
  Switch,
  TOAST_TIMEOUT,
  TabBar,
  TabBarItem,
  Content6 as TabsContent,
  List2 as TabsList,
  Root11 as TabsRoot,
  Trigger6 as TabsTrigger,
  TextArea,
  Toaster,
  ToggleGroup,
  Tooltip,
  TooltipContent,
  TooltipPortal,
  TooltipProvider,
  TooltipRoot,
  TooltipTrigger,
  CircleButton as TopBarButton,
  CircleButtonGroup as TopBarButtonGroup,
  UIKeys,
  VirtualList,
  badgeVariants,
  bannerVariants,
  buttonStyles,
  buttonStyles as buttonVariants,
  circleButtonStyles,
  contextMenuClassNames,
  contextMenuItemClassNames,
  contextMenuSeparatorClassNames,
  dialogManager,
  errorStyles,
  inputSizes,
  inputStyles,
  keySymbols,
  modifierSymbols,
  selectPillStyles,
  selectStyles,
  toast,
  tw,
  useContextMenuContext,
  useDialog,
  useEndAnchoredVirtualizer,
  usePopover,
  useResizableContext,
  z
};
//# sourceMappingURL=index.js.map