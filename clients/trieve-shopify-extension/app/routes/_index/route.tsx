import { LoaderFunctionArgs, redirect } from "@remix-run/node";
import { validateTrieveAuth } from "app/auth";

export const loader = async ({ request }: LoaderFunctionArgs) => {
  const url = new URL(request.url);
  // call validateTrieveAuth here to ensure the user is authenticated
  // The first time this gets called it will create a new dataset,
  // calling it here prevents race conditions.
  console.log("Validating Trieve Auth");
  await validateTrieveAuth(request);
  throw redirect(`/app?${url.searchParams.toString()}`);
};
