import React, { Suspense, ComponentType, ReactNode } from "react";

interface SuspenseWrapperProps {
  fallback?: ReactNode;
}

export function withSuspense<P extends object>(
  Component: ComponentType<P>,
  defaultFallback: ReactNode = <div>Loading...</div>,
) {
  const WithSuspense = (props: P & SuspenseWrapperProps) => {
    const { fallback = defaultFallback, ...componentProps } = props;

    return (
      <Suspense fallback={fallback}>
        <Component {...(componentProps as P)} />
      </Suspense>
    );
  };

  // Set display name for better debugging
  WithSuspense.displayName = `WithSuspense(${
    Component.displayName || Component.name || "Component"
  })`;

  return WithSuspense;
}
