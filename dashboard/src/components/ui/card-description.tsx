'use client';

import * as React from 'react';

const CardDescription = React.forwardRef<
  HTMLParagraphElement,
  React.HTMLAttributes<HTMLParagraphElement>
>(({ className, ...props }, ref) => (
  <p
    className={[
      'text-sm',
      '[&_&]:leading-revert',
      className,
    ].filter(Boolean).join(' ')}
    ref={ref}
    {...props}
  />
));
CardDescription.displayName = 'CardDescription';

export { CardDescription };