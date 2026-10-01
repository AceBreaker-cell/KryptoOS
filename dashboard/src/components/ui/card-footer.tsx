'use client';

import * as React from 'react';

const CardFooter = React.forwardRef<
  HTMLDivElement,
  React.HTMLAttributes<HTMLDivElement>
>(({ className, ...props }, ref) => (
  <div
    className={[
      'flex flex-col sm:flex-sm-row sm:items-stretch sm:justify-start sm:space-x-2',
      '[&_>:not(template)]:mt-2',
      '[&_>:not(template):first-child]:mt-0',
      className,
    ].filter(Boolean).join(' ')}
    ref={ref}
    {...props}
  />
));
CardFooter.displayName = 'CardFooter';

export { CardFooter };