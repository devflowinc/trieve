import { LoaderFunctionArgs } from "@remix-run/node";
import { nanoid } from "nanoid";
import { validateTrieveAuth } from "app/auth";
import { tryCatch } from "app/loaders";

export const loader = async (args: LoaderFunctionArgs) => {
  const trieveKeyResult = await tryCatch(validateTrieveAuth(args.request));
  if (trieveKeyResult.error) {
    throw new Error("Failed to authorize with trieve", {
      cause: trieveKeyResult.error,
    });
  }
  if (!trieveKeyResult.data) {
    throw new Error("Unauthorized with trieve");
  }

  const clientId = process.env.JUDGE_ME_CLIENT_ID;
  if (!clientId) {
    throw new Error("JUDGE_ME_CLIENT_ID is not set");
  }
  const judgeMeRedirectUri = process.env.JUDGE_ME_REDIRECT_URI;
  if (!judgeMeRedirectUri) {
    throw new Error("JUDGE_ME_REDIRECT_URI is not set");
  }

  const scopes = ["read_shops", "read_reviews"];
  const code = nanoid(28);

  // Save the code and trieve key so they can be reconnected during the redirect_uri
  const saveOauthResult = await tryCatch(
    prisma.judgeOAuthState.create({
      data: {
        code,
        apiKeyId: trieveKeyResult.data.id,
      },
    }),
  );

  if (saveOauthResult.error) {
    throw new Error("Failed to save oauth state", {
      cause: saveOauthResult.error,
    });
  }

  const params = new URLSearchParams({
    client_id: clientId,
    scope: scopes.join(" "),
    redirect_uri: judgeMeRedirectUri,
    response_type: "code",
    state: saveOauthResult.data.code,
  });

  return {
    url: "https://app.judge.me/oauth/authorize?" + params.toString(),
  };
};
