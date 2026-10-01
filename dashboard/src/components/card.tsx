'use client';

import * as React from 'react';

import { CardContent } from '@/components/ui/card-content';
import { CardDescription } from '@/components/ui/card-description';
import { CardFooter } from '@/components/ui/card-footer';
import { CardHeader } from '@/components/ui/card-header';
import { CardTitle } from '@/components/ui/card-title';

export {
  CardContent,
  CardDescription,
  CardFooter,
  CardHeader,
  CardTitle,
  // Note: The actual Card component would be composed from these parts
  // For simplicity in this example, we're exporting the sub-components
  // In a full implementation, you'd have a Card component that combines them
};