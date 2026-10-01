/// <reference types="next" />
/// <reference types="next/image-types/global" />

// Type definitions for the project
declare namespace NodeJS {
  interface ProcessEnv {
    NEXT_PUBLIC_API_URL: string;
  }
}