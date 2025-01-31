import { LoaderFunctionArgs, json } from "@remix-run/node";
import { authenticate } from "app/shopify.server";

export const validateTrieveAuth = async ({ request }: LoaderFunctionArgs) => {
  let { sessionToken } = await authenticate.admin(request);

  const key = await prisma.apiKey.findFirst({
    where: {
      userId: (sessionToken.sub as string) ?? "",
    },
  });
  if (!key) {
    throw json({ message: "No Key" }, 404);
  }
  return key;
};
