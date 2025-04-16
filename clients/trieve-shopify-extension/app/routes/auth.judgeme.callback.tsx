import { LoaderFunctionArgs } from "@remix-run/node";
import "../routes/_index/tailwind.css";
import { useRouteError } from "@remix-run/react";
import { tryCatch } from "app/loaders";

export const loader = async (args: LoaderFunctionArgs) => {
  const url = new URL(args.request.url);
  const queryParams = {
    code: url.searchParams.get("code"),
    state: url.searchParams.get("state"),
  };

  // verify that state is valid

  if (!queryParams.code) {
    throw new Error("No code provided");
  }
  if (!queryParams.state) {
    throw new Error("No state provided");
  }

  const matchingTrieveKey = await tryCatch(
    prisma.judgeOAuthState.findFirst({
      where: {
        code: queryParams.state,
      },
    }),
  );

  if (matchingTrieveKey.error) {
    throw new Error("Failed to find matching Trieve key", {
      cause: matchingTrieveKey.error,
    });
  }
  if (!matchingTrieveKey.data) {
    throw new Error("No matching Trieve key found");
  }

  const clientId = process.env.JUDGE_ME_CLIENT_ID;
  if (!clientId) {
    throw new Error("JUDGE_ME_CLIENT_ID is not set");
  }
  const clientSecret = process.env.JUDGE_ME_SECRET;
  if (!clientSecret) {
    throw new Error("JUDGE_ME_REDIRECT_URI is not set");
  }
  const judgeMeRedirectUri = process.env.JUDGE_ME_REDIRECT_URI;
  if (!judgeMeRedirectUri) {
    throw new Error("JUDGE_ME_REDIRECT_URI is not set");
  }

  const response = await fetch("https://judge.me/oauth/token", {
    headers: {
      "Content-Type": "application/json",
    },
    method: "POST",
    body: JSON.stringify({
      client_id: clientId,
      client_secret: clientSecret,
      code: queryParams.code,
      redirect_uri: judgeMeRedirectUri,
      state: queryParams.state,
      grant_type: "authorization_code",
    }),
  });

  if (!response.ok) {
    throw new Error("Failed to get token from judge.me", {
      cause: await response.text(),
    });
  }

  const data = (await response.json()) as {
    access_token: string;
    scope: string;
  };

  const savedKeys = await tryCatch(
    prisma.judgeMeKeys.create({
      data: {
        authKey: data.access_token,
        trieveApiKeyId: matchingTrieveKey.data.apiKeyId,
      },
    }),
  );

  if (savedKeys.error) {
    throw new Error("Failed to save judge.me keys", {
      cause: savedKeys.error,
    });
  }
  return { keys: savedKeys.data };
};

export default function JudgeMeCallbackDonePage() {
  return (
    <div className="w-screen h-screen bg-neutral-200 grid place-items-center">
      <div className="text-center shadow-sm bg-white border-neutral-300 border rounded-md p-8">
        <img
          className="w-14 mx-auto h-14"
          src="https://cdn.trieve.ai/trieve-logo.png"
        />
        <div className="text-lg">
          Trieve successfully connected with Judge.me
        </div>
        <div className="opacity-80 pt-2">You can now close this page.</div>
      </div>
    </div>
  );
}

export function ErrorBoundary() {
  const error = useRouteError();

  return (
    <div className="w-screen h-screen bg-neutral-200 grid place-items-center">
      <div className="text-center shadow-sm bg-white border-neutral-300 border rounded-md p-8">
        <div className="text-lg text-red-900">There was an error.</div>
        <div className="opacity-80 font-mono pt-2">
          {error instanceof Error && (error.message || "Internal server error")}
        </div>
      </div>
    </div>
  );
}
