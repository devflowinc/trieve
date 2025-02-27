import { LoaderFunctionArgs, redirect } from "@remix-run/node";
import { Outlet } from "@remix-run/react";
import { validateTrieveAuth } from "app/auth";
import { StrongTrieveKey } from "app/types";

// Validates that user has a connected dataset, if not redirects to /app/setup and then right back
export const loader = async (args: LoaderFunctionArgs) => {
  const key = await validateTrieveAuth(args.request, false);
  if (!key.currentDatasetId) {
    console.log("redirecting!!!");
    throw redirect("/app/setup");
  }

  return {
    key: key as StrongTrieveKey,
  };
};

export default function Dashboard() {
  return (
    <div>
      <h1>Dashboard</h1>
      <Outlet />
    </div>
  );
}
