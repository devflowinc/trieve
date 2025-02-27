import { LoaderFunctionArgs } from "@remix-run/node";
import { useLoaderData } from "@remix-run/react";
import { validateTrieveAuth } from "app/auth";

export const loader = async (args: LoaderFunctionArgs) => {
  const key = await validateTrieveAuth(args.request);
  return { key };
};

export default function Dashboard() {
  const { key } = useLoaderData<typeof loader>();
  return <div>Datasetid: {key.currentDatasetId}</div>;
}
