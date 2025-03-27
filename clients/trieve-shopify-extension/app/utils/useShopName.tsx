import { useMatches } from "@remix-run/react";

export const useShopName = () => {
  const matches = useMatches();
  const shopname: string | null =
    (matches.find((m) => m.pathname == "/app/")?.data as any).shopDomain ||
    null;

  return shopname;
};
