'use client';

import { useState, useEffect, useRef } from 'react';
import { Button } from '@/components/button';
import { Toast } from '@/components/toast';
import { Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle } from '@/components/card';
import { Switch, SwitchThumb } from '@/components/switch';
import { LucideIcon, Zap, Monitor, Activity, ShieldCheck, Lock, X } from 'lucide-react';
import { Socket } from '@/components/socket';

export default function Home() {
  const [isLocked, setIsLocked] = useState(false);
  const [toasts, setToasts] = useState<Array<{ id: string; title: string; description: string }>>([]);
  const [logs, setLogs] = useState<string[]>([]);
  const socketRef = useRef<Socket | null>(null);

  useEffect(() => {
    // Initialize socket connection to Rust agent
    const socket = new Socket('ws://localhost:3001/api/socket');
    socketRef.current = socket;

    socket.onMessage((message: string) => {
      const data = JSON.parse(message);

      // Handle different types of messages
      if (data.type === 'security-event') {
        // Add to audit log
        setLogs(prev => [...prev, `[${new Date().toLocaleTimeString()}] ${data.description}`]);
        // Keep only last 100 logs
        if (logs.length > 100) {
          setLogs(logs.slice(-100));
        }

        // Show toast for interactive events
        if data.requiresAction {
          const toastId = Math.random().toString(36).substr(2, 9);
          setToasts(prev => [
            ...prev,
            {
              id: toastId,
              title: `Security Alert: ${data.processName}`,
              description: data.description
            }
          ]);

          // Auto-remove toast after 30 seconds if not actioned
          setTimeout(() => {
            setToasts(prev => prev.filter(t => t.id !== toastId));
          }, 30000);
        }
      }
    });

    socket.connect();

    return () => {
      socket.disconnect();
    };
  }, []);

  const handleLockToggle = async (locked: boolean) => {
    setIsLocked(locked);
    // Send command to Rust agent to enable/disable network isolation
    if (socketRef.current) {
      await socketRef.current.sendCommand({
        type: 'set-lock',
        locked
      });
    }
  };

  const handleToastAction = (toastId: string, action: string) => {
    setToasts(prev => prev.filter(t => t.id !== toastId));

    // Send action to Rust agent
    if (socketRef.current) {
      // In a real implementation, we would identify which event this toast corresponds to
      // and send the appropriate action
      socketRef.current.sendCommand({
        type: 'toast-action',
        toastId,
        action
      });
    }
  };

  return (
    <main className="min-h-screen bg-background text-foreground">
      <div className="container mx-auto px-4 py-8">
        <h1 className="text-3xl font-bold mb-6 text-center">
          KryptonOS Zero-Trust Host Access Controller
        </h1>

        <div className="grid gap-6 md:grid-cols-2 lg:grid-cols-3">
          {/* Master Lock Switch */}
          <Card className="hover:shadow-lg transition-shadow">
            <CardHeader className="pb-4">
              <div className="flex items-center space-x-3">
                <LucideIcon className="h-6 w-6 text-muted-foreground" {...{ ...Lock }} />
                <CardTitle className="text-lg">System Lockdown</CardTitle>
              </div>
            </CardHeader>
            <CardContent className="space-y-4">
              <p className="text-muted-foreground">
                Instantly isolate the OS from all network connections when enabled.
              </p>
              <div className="flex items-center justify-between">
                <Switch
                  checked={isLocked}
                  onCheckedChange={handleLockToggle}
                  aria-label="System lockdown switch"
                >
                  <SwitchThumb className="translate-y-[2px]" />
                </Switch>
                <span className={`
                  px-3 py-1 rounded-full text-sm font-medium
                  ${isLocked ? 'bg-destructive/20 text-destructive' : 'bg-secondary/20 text-secondary'}
                `}>
                  {isLocked ? 'LOCKED' : 'ACTIVE'}
                </span>
              </div>
            </CardContent>
          </Card>

          {/* JIT Request Toast Container */}
          <Card className="hover:shadow-lg transition-shadow">
            <CardHeader className="pb-4">
              <div className="flex items-center space-x-3">
                <LucideIcon className="h-6 w-6 text-muted-foreground" {...{ ...Zap }} />
                <CardTitle className="text-lg">Just-In-Time Requests</CardTitle>
              </div>
            </CardHeader>
            <CardContent>
              {toasts.length > 0 ? (
                toasts.map(toast => (
                  <div key={toast.id} className="mb-4 p-4 bg-muted/50 rounded-lg">
                    <h3 className="font-semibold mb-2">{toast.title}</h3>
                    <p className="text-sm text-muted-foreground mb-4">{toast.description}</p>
                    <div className="flex space-x-2">
                      <Button
                        variant="outline"
                        size="sm"
                        onClick={() => handleToastAction(toast.id, 'approve-permanently')}
                      >
                        Approve Permanently
                      </Button>
                      <Button
                        variant="outline"
                        size="sm"
                        onClick={() => handleToastAction(toast.id, 'allow-15m')}
                      >
                        Allow 15m
                      </Button>
                      <Button
                        variant="destructive"
                        size="sm"
                        onClick={() => handleToastAction(toast.id, 'nuke-process')}
                      >
                        Nuke Process
                      </Button>
                    </div>
                  </div>
                ))
              ) : (
                <p className="text-muted-foreground text-center py-8">
                  No pending security requests
                </p>
              )}
            </CardContent>
          </Card>

          {/* Audit Stream Terminal */}
          <Card className="hover:shadow-lg transition-shadow">
            <CardHeader className="pb-4">
              <div className="flex items-center space-x-3">
                <LucideIcon className="h-6 w-6 text-muted-foreground" {...{ ...Activity }} />
                <CardTitle className="text-lg">Audit Stream</CardTitle>
              </div>
            </CardHeader>
            <CardContent className="h-96 overflow-y-auto">
              <div className="space-y-2">
                {logs.map((log, index) => (
                  <div key={index} className="flex items-start space-x-2 text-sm font-mono">
                    <div className="w-2 h-2 bg-primary/50 rounded-full mt-1 flex-shrink-0"></div>
                    <div className="whitespace-pre-wrap">{log}</div>
                  </div>
                ))}
                {logs.length === 0 && (
                  <div className="flex items-center justify-center h-full text-muted-foreground">
                    Waiting for security events...
                  </div>
                )}
              </div>
            </CardContent>
            <CardFooter className="flex justify-between items-center pt-4 text-xs text-muted-foreground">
              <span>Last updated: {new Date().toLocaleTimeString()}</span>
              <Button variant="ghost" size="icon" onClick={() => setLogs([])}>
                <LucideIcon className="h-4 w-4" {...{ ...X }} />
              </Button>
            </CardFooter>
          </Card>
        </div>

        {/* Connection Status Indicator */}
        <div className="fixed bottom-4 right-4 flex items-center space-x-2">
          <div className="w-2 h-2 bg-{socketRef.current?.isConnected ? 'success' : 'destructive'}/20 rounded-full"></div>
          <span className="text-xs">
            {socketRef.current?.isConnected ? 'Connected to Agent' : 'Disconnected'}
          </span>
        </div>

        {/* Toast Container */}
        <div className="fixed bottom-4 left-4 z-50">
          <Toast toasts={toasts} onAction={handleToastAction} />
        </div>

        {/* WebSocket Connection Component */}
        <Socket
          url="ws://localhost:3001/api/socket"
          ref={socketRef}
        />
      </div>
    </main>
  );
}