'use client';

import * as React from 'react';

const CardTitle = React.forwardRef<
  HTMLHeadingElement,
  React.HTMLAttributes<HTMLHeadingElement>
>(({ className, ...props }, ref) => (
  <h3
    className={[
      'text-lg',
      '[&_&]:leading-revert',
      className,
    ].filter(Boolean).join(' ')}
    ref={ref}
    {...props}
  />
));
CardTitle.displayName = 'CardTitle';

export { CardTitle };