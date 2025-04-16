import { ActionFunctionArgs, LoaderFunctionArgs } from "@remix-run/node";
import { api } from "app/api/root";

export async function loader(args: LoaderFunctionArgs) {
  const response = await api.request(args.request);
  return response;
}

export async function action(args: ActionFunctionArgs) {
  const response = await api.request(args.request);
  return response;
}
