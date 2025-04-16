import { LoaderFunctionArgs } from "@remix-run/node";
import { validateTrieveAuth } from "app/auth";

export type JudgeMeKeyInfo = Awaited<ReturnType<typeof loader>>;

export const loader = async (args: LoaderFunctionArgs) => {
  const trieveInfo = await validateTrieveAuth(args.request);

  const judgeMeKey = await prisma.judgeMeKeys.findFirst({
    where: {
      trieveApiKeyId: trieveInfo.id,
    },
  });

  return {
    judgeMeKey,
  };
};
