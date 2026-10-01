'use client';

import { useEffect, useState, useRef } from 'react';

interface SocketProps {
  url: string;
  onMessage?: (message: string) => void;
  onConnect?: () => void;
  onDisconnect?: () => void;
}

export function Socket({ url, onMessage, onConnect, onDisconnect }: SocketProps) {
  const [isConnected, setIsConnected] = useState(false);
  const socketRef = useRef<WebSocket | null>(null);

  useEffect(() => {
    const socket = new WebSocket(url);
    socketRef.current = socket;

    socket.onopen = () => {
      setIsConnected(true);
      onConnect?.();
    };

    socket.onmessage = (event) => {
      onMessage?.(event.data);
    };

    socket.onclose = () => {
      setIsConnected(false);
      onDisconnect?.();
    };

    socket.onerror = (error) => {
      console.error('WebSocket error:', error);
      setIsConnected(false);
      onDisconnect?.();
    };

    return () => {
      socket.close();
    };
  }, [url, onConnect, onDisconnect, onMessage]);

  const sendMessage = (message: any) => {
    if (socketRef.current?.readyState === WebSocket.OPEN) {
      socketRef.current.send(JSON.stringify(message));
    }
  };

  return null; // This component doesn't render anything, it just manages the socket connection
}

// Hook to use the socket connection
export function useSocket() {
  const socketRef = useRef<WebSocket | null>(null);
  const [isConnected, setIsConnected] = useState(false);

  const connect = (url: string) => {
    socketRef.current = new WebSocket(url);

    socketRef.current.onopen = () => {
      setIsConnected(true);
    };

    socketRef.current.onclose = () => {
      setIsConnected(false);
    };

    socketRef.current.onerror = (error) => {
      console.error('WebSocket error:', error);
      setIsConnected(false);
    };
  };

  const sendMessage = (message: any) => {
    if (socketRef.current?.readyState === WebSocket.OPEN) {
      socketRef.current.send(JSON.stringify(message));
    }
  };

  const onMessage = (callback: (message: string) => void) => {
    if (socketRef.current) {
      socketRef.current.onmessage = (event) => {
        callback(event.data);
      };
    }
  };

  const disconnect = () => {
    socketRef.current?.close();
    setIsConnected(false);
  };

  return {
    isConnected,
    connect,
    sendMessage,
    onMessage,
    disconnect
  };
}