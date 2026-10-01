'use client';

import * as React from 'react';

const CardHeader = React.forwardRef<
  HTMLDivElement,
  React.HTMLAttributes<HTMLDivElement>
>(({ className, ...props }, ref) => (
  <div
    className={[
      'flex flex-col space-y-1.5 pt-4',
      className,
    ].filter(Boolean).join(' ')}
    ref={ref}
    {...props}
  />
));
CardHeader.displayName = 'CardHeader';

export { CardHeader };