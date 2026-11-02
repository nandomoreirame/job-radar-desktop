import type * as React from "react";

import { mergeProps } from "@base-ui/react/merge-props";
import { useRender } from "@base-ui/react/use-render";
import { cva, type VariantProps } from "class-variance-authority";
import { cn } from "@/shared/lib/utils";

const statusVariants = cva("inline-flex w-fit shrink-0 items-center gap-1.5 overflow-hidden rounded-4xl border px-2.5 py-1 text-xs font-medium whitespace-nowrap transition-colors", {
  variants: {
    variant: {
      default: "border-transparent bg-white/5 text-noir-muted **:data-[slot=status-indicator]:bg-noir-muted",
      success: "border-noir-teal/30 bg-noir-teal/10 text-noir-teal **:data-[slot=status-indicator]:bg-noir-teal",
      error: "border-noir-red/30 bg-noir-red/10 text-noir-red **:data-[slot=status-indicator]:bg-noir-red",
      warning: "border-noir-amber/40 bg-noir-amber/10 text-noir-amber **:data-[slot=status-indicator]:bg-noir-amber",
      info: "border-noir-cream/30 bg-noir-cream/10 text-noir-cream **:data-[slot=status-indicator]:bg-noir-cream",
    },
  },
  defaultVariants: {
    variant: "default",
  },
});

interface StatusProps
  extends
    VariantProps<typeof statusVariants>,
    React.ComponentProps<"div">,
    useRender.ComponentProps<"div"> {}

function Status(props: StatusProps) {
  const { className, variant = "default", render, ...rootProps } = props;

  return useRender({
    defaultTagName: "div",
    props: mergeProps<"div">(
      {
        className: cn(statusVariants({ variant }), className),
      },
      rootProps,
    ),
    render,
    state: {
      slot: "status",
      variant,
    },
  });
}

function StatusIndicator(props: React.ComponentProps<"div">) {
  const { className, ...indicatorProps } = props;

  return (
    <div
      data-slot="status-indicator"
      {...indicatorProps}
      className={cn("relative flex size-2 shrink-0 rounded-full before:absolute before:inset-0 before:animate-ping before:rounded-full before:bg-inherit after:absolute after:inset-0.5 after:rounded-full after:bg-inherit", className)}
    />
  );
}

function StatusLabel(props: React.ComponentProps<"div">) {
  const { className, ...labelProps } = props;

  return (
    <div
      data-slot="status-label"
      {...labelProps}
      className={cn("leading-none", className)}
    />
  );
}

export { Status, StatusIndicator, StatusLabel, statusVariants };
