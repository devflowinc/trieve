import { useLocation } from "@solidjs/router";
import { createMemo } from "solid-js";

export const usePathname = () => {
  const location = useLocation();
  const pathname = createMemo(() => location.pathname);
  return pathname;
};
