import { validateTrieveAuth } from "app/auth";
import { tryCatch } from "app/loaders";
import { Hono } from "hono";
import { nanoid } from "nanoid";

export const judgeMe = new Hono()
  .get("/login", async (c) => {
    const trieveKeyResult = await tryCatch(validateTrieveAuth(c.req.raw));
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

    return c.json({
      url: "https://app.judge.me/oauth/authorize?" + params.toString(),
    });
  })

  .get("/info", async (c) => {
    const trieveInfo = await validateTrieveAuth(c.req.raw);

    const judgeMeKey = await prisma.judgeMeKeys.findFirst({
      where: {
        trieveApiKeyId: trieveInfo.id,
      },
    });

    return c.json({
      judgeMeKey,
    });
  })

  .get("/reviewCount", async (c) => {
    const trieve = await validateTrieveAuth(c.req.raw);
    console.log(trieve);
    const judgeMeKey = await tryCatch(
      prisma.judgeMeKeys.findFirst({
        where: {
          trieveApiKeyId: trieve.id,
        },
      }),
    );
    if (judgeMeKey.error || !judgeMeKey.data) {
      throw new Error("Failed to find judge.me key", {
        cause: judgeMeKey.error,
      });
    }

    const params = new URLSearchParams({
      api_token: judgeMeKey.data.authKey,
    });
    const reviews = await fetch(
      "https://api.judge.me/api/v1/reviews/count?" + params.toString(),
      {
        headers: {
          "Content-Type": "application/json",
        },
      },
    );

    if (!reviews.ok) {
      throw new Error("Failed to get judge.me review count", {
        cause: await reviews.text(),
      });
    }

    const data = (await reviews.json()) as { count: number };
    return c.json(data);
  })

  .post("/sync", async (c) => {
    const trieve = await validateTrieveAuth(c.req.raw);
    const judgeMeKey = await tryCatch(
      prisma.judgeMeKeys.findFirst({
        where: {
          trieveApiKeyId: trieve.id,
        },
      }),
    );
    if (judgeMeKey.error || !judgeMeKey.data) {
      throw new Error("Failed to find judge.me key", {
        cause: judgeMeKey.error,
      });
    }

    // Get judgeme reviews
    const page = 1;
    const perPage = 100;

    const params = new URLSearchParams({
      page: page.toString(),
      per_page: perPage.toString(),
      api_token: judgeMeKey.data.authKey,
    });

    const reviews = await fetch(
      "https://api.judge.me/api/v1/reviews?" + params.toString(),
      {
        headers: {
          "Content-Type": "application/json",
        },
      },
    );

    console.log(reviews);
  });
