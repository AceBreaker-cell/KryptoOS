'use client';

import * as React from 'react';
import * as SwitchPrimitive from '@radix-ui/react-switch';
import { Slot } from '@radix-ui/react-slot';

const Switch = React.forwardRef<
  HTMLElement,
  React.ComponentPropsWithoutRef<typeof SwitchPrimitive.Root>
>(({ className, children, ...props }, ref) => (
  <SwitchPrimitive.Root
    className={[
      'inline-flex h-[22px] w-[38px] shrink-0 items-center rounded-full border-2 border-transparent transition-all'
        ' data-state[checked]:bg-primary data-state[checked]:border-primary'
        ' data-state[unchecked]:bg-input data-state[unchecked]:border-input'
    ].filter(Boolean).join(' ')}
    ref={ref}
    {...props}
  >
    <SwitchPrimitive.Thumb
      className={[
        'block h-[18px] w-[18px] rounded-full bg-background shadow'
        ' ring-0 transition-transform data-state[checked]:translate-x-[16px]'
      ].filter(Boolean).join(' ')}
    >
      <Slot>{children}</Slot>
    </SwitchPrimitive.Thumb>
  </SwitchPrimitive.Root>
));
Switch.displayName = SwitchPrimitive.Root.displayName;

export { Switch };