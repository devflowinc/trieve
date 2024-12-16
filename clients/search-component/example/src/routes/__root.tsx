import { createRootRoute, Outlet } from "@tanstack/react-router";
import { TanStackRouterDevtools } from "@tanstack/router-devtools";

export const Route = createRootRoute({
  component: () => (
    <>
      <div className="random-trigger-location" style={{ background: "red" }}>
        Hi, I'm a random root div that is outside the hierarchy
      </div>
      <Outlet />
      <TanStackRouterDevtools />
    </>
  ),
});
