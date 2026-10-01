'use client';

import * as React from 'react';

const CardContent = React.forwardRef<
  HTMLDivElement,
  React.HTMLAttributes<HTMLDivElement>
>(({ className, ...props }, ref) => (
  <div
    className={[
      'flex w-full flex-col gap-4 space-y-4 text-[0.875rem]',
      className,
    ].filter(Boolean).join(' ')}
    ref={ref}
    {...props}
  />
));
CardContent.displayName = 'CardContent';

export { CardContent };