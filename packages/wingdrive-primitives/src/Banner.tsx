import { clsx } from 'clsx';
import { type VariantProps, cva } from 'class-variance-authority';
import { forwardRef } from 'react';
import { Warning, CheckCircle, Info, XCircle } from '@phosphor-icons/react';

const bannerVariants = cva(
  'relative flex w-full items-center gap-3 rounded-lg border px-4 py-3 text-sm',
  {
    variants: {
      variant: {
        default: 'border-accent/20 bg-accent/10 text-accent',
        info: 'border-status-info/20 bg-status-info/10 text-status-info',
        success: 'border-status-success/20 bg-status-success/10 text-status-success',
        warning: 'border-status-warning/20 bg-status-warning/10 text-status-warning',
        error: 'border-status-error/20 bg-status-error/10 text-status-error',
      },
    },
    defaultVariants: {
      variant: 'default',
    },
  }
);

const dotVariants = cva('size-2 rounded-full', {
  variants: {
    variant: {
      default: 'bg-accent',
      info: 'bg-status-info',
      success: 'bg-status-success',
      warning: 'bg-status-warning',
      error: 'bg-status-error',
    },
  },
  defaultVariants: {
    variant: 'default',
  },
});

export interface BannerProps
  extends React.HTMLAttributes<HTMLDivElement>,
    VariantProps<typeof bannerVariants> {
  showDot?: boolean;
}

const Banner = forwardRef<HTMLDivElement, BannerProps>(
  ({ className, variant, showDot = true, children, ...props }, ref) => {
    const icons = {
      default: Info,
      info: Info,
      success: CheckCircle,
      warning: Warning,
      error: XCircle,
    };

    const Icon = icons[variant || 'default'];

    return (
      <div
        ref={ref}
        role="alert"
        className={clsx(bannerVariants({ variant }), className)}
        {...props}
      >
        {showDot && <span className={clsx(dotVariants({ variant }))} />}
        <Icon className="size-4 shrink-0" />
        <div className="flex-1">{children}</div>
      </div>
    );
  }
);

Banner.displayName = 'Banner';

export { Banner, bannerVariants };
