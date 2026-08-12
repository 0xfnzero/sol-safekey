"use client";

import { Tooltip } from "@base-ui/react/tooltip";
import { CircleHelp } from "lucide-react";
import { useId } from "react";

interface FieldHelpProps {
  description: string;
  label: string;
}

export function FieldHelp({ description, label }: FieldHelpProps) {
  const tooltipId = useId();

  return (
    <Tooltip.Root>
      <Tooltip.Trigger
        type="button"
        delay={250}
        closeDelay={100}
        closeOnClick={false}
        aria-label={label}
        aria-describedby={tooltipId}
        data-field-help=""
        className="inline-flex h-6 w-6 shrink-0 items-center justify-center rounded-md text-gray-400 transition-colors hover:bg-white/10 hover:text-white focus-visible:bg-white/10 focus-visible:text-white focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-white/30"
      >
        <CircleHelp aria-hidden="true" className="h-4 w-4" />
      </Tooltip.Trigger>
      <Tooltip.Portal>
        <Tooltip.Positioner sideOffset={8} collisionPadding={12} className="z-[300]">
          <Tooltip.Popup
            id={tooltipId}
            role="tooltip"
            className="w-72 max-w-[calc(100vw-2rem)] rounded-md border border-white/15 bg-zinc-950 px-3 py-2 text-xs leading-5 text-gray-100 shadow-2xl transition-opacity data-[ending-style]:opacity-0 data-[starting-style]:opacity-0"
          >
            {description}
          </Tooltip.Popup>
        </Tooltip.Positioner>
      </Tooltip.Portal>
    </Tooltip.Root>
  );
}
