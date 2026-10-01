'use client';

import * as React from 'react';
import * as ToastPrimitive from '@radix-ui/react-toast';
import { Button } from '@/components/button';

const VIEWPORT_PADDING = 25;

function getMaxHeight() {
  if (typeof window === 'undefined') return 0;
  return Math.max(
    0,
    window.innerHeight - VIEWPORT_PADDING
  );
}

function GetViewport() {
  return (
    <div
      style={{
        position: 'fixed',
        bottom: 0,
        right: 0,
        padding: VIEWPORT_PADDING,
        pointerEvents: 'none',
        display: 'flex',
        flexDirection: 'column',
        alignItems: 'flex-end',
        height: getMaxHeight(),
        width: '100%',
        zIndex: 2147483647,
      }}
      aria-live="assertive"
    >
      <div
        style={{
          display: 'flex',
          flexDirection: 'column',
          gap: 10,
          width: '100%',
          maxWidth: '400px',
        }}
        data-viewport
      >
        <ToastPrimitive.Provider>
          <ToastPrimitive.Viewport>
            <div data-toasts />
          </ToastPrimitive.Viewport>
        </ToastPrimitive.Provider>
      </div>
    </div>
  );
}

interface ToastProps {
  toasts: Array<{ id: string; title: string; description: string }>;
  onAction: (toastId: string, action: string) => void;
}

export function Toast({ toasts, onAction }: ToastProps) {
  return (
    <GetViewport>
      {toasts.map((toast) => (
        <ToastPrimitive.Root
          key={toast.id}
          role="alert"
        >
          <ToastPrimitive.Title className="mb-2">{toast.title}</ToastPrimitive.Title>
          <ToastPrimitive.Description className="mb-4">{toast.description}</ToastPrimitive.Description>
          <ToastPrimitive.Action>
            <Button
              variant="outline"
              size="sm"
              onClick={() => onAction(toast.id, 'approve-permanently')}
            >
              Approve Permanently
            </Button>
          </ToastPrimitive.Action>
          <ToastPrimitive.Action>
            <Button
              variant="outline"
              size="sm"
              onClick={() => onAction(toast.id, 'allow-15m')}
            >
              Allow 15m
            </Button>
          </ToastPrimitive.Action>
          <ToastPrimitive.Action>
            <Button
              variant="destructive"
              size="sm"
              onClick={() => onAction(toast.id, 'nuke-process')}
            >
              Nuke Process
            </Button>
          </ToastPrimitive.Action>
          <ToastPrimitive.Close>
            <button
              className="absolute right-2 top-2 rounded-xs p-0.5 hover:bg-accent/20"
              aria-label="Close"
            >
              <svg
                xmlns="http://www.w3.org/2000/svg"
                width="16"
                height="16"
                fill="currentColor"
                viewBox="0 0 16 16"
              >
                <path
                  d="M4.646 4.646a.5.5 0 0 1 .708 0L8 7.293l2.646-2.647a.5.5 0 0 1 .708.708L8.707 8l2.647 2.646a.5.5 0 0 1-.708.708L8 8.707l-2.646 2.647a.5.5 0 0 1-.708-.708L7.293 8 4.646 5.354a.5.5 0 0 1 0-.708z"
                />
              </svg>
            </button>
          </ToastPrimitive.Close>
        </ToastPrimitive.Root>
      ))}
    </GetViewport>
  );
}