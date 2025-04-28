import { validateTrieveAuth } from "app/auth";
import { getTrieveBaseUrlEnv } from "app/env.server";
import { tryCatch } from "app/loaders";
import { Hono } from "hono";
import { nanoid } from "nanoid";
import { ChunkReqPayload } from "trieve-ts-sdk";

type JudgeMeReviewer = {
  id: string;
  email: string;
  name: string;
  external_id: number;
  phone?: string;
  tags?: string[];
  accepts_marketing?: boolean;
  unsubscribed_at?: string;
};

type JudgeMeReview = {
  id: number;
  title: string;
  body: string;
  rating: number;
  product_external_id: number;
  reviewer: JudgeMeReviewer;
  source: string;
  curated: "not-yet" | "ok" | "spam";
  published: boolean;
  hidden: boolean;
  verified: string;
  featured: boolean;
  created_at: string;
  updated_at: string;
  has_published_pictures: boolean;
  has_published_videos: boolean;
  // pictures: [];
  // ip_address: null;
  product_title: string;
  product_handle: string;
};

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
    const trieveKey = await validateTrieveAuth(c.req.raw);
    const judgeMeKey = await tryCatch(
      prisma.judgeMeKeys.findFirst({
        where: {
          trieveApiKeyId: trieveKey.id,
        },
      }),
    );
    if (judgeMeKey.error || !judgeMeKey.data) {
      throw new Error("Failed to find judge.me key", {
        cause: judgeMeKey.error,
      });
    }

    // Get judgeme reviews
    let page = 1;
    const perPage = 100;
    let isDone = false;
    while (!isDone) {
      const params = new URLSearchParams({
        page: page.toString(),
        per_page: perPage.toString(),
        api_token: judgeMeKey.data.authKey,
      });

      const response = await fetch(
        "https://api.judge.me/api/v1/reviews?" + params.toString(),
        {
          headers: {
            "Content-Type": "application/json",
          },
        },
      );

      if (!response.ok) {
        throw new Error("Failed to get judge.me reviews", {
          cause: await response.text(),
        });
      }

      const reviews = (await response.json()) as {
        current_page: number;
        per_page: number;
        reviews: JudgeMeReview[];
      };

      if (reviews.reviews.length === 0) {
        isDone = true;
        break;
      }

      const chunks = reviews.reviews.map(transformReviewToChunk);
      console.log(`Sending ${JSON.stringify(chunks)} chunks to Trieve`);

      fetch(`${getTrieveBaseUrlEnv()}/api/chunk`, {
        method: "POST",
        headers: {
          Authorization: `Bearer ${trieveKey.key}`,
          "TR-Dataset": trieveKey.currentDatasetId,
          "Content-Type": "application/json",
        },
        body: JSON.stringify(chunks),
      }).catch((e) => {
        console.error(`Error sending chunks to Trieve: ${e}`);
      });

      page += 1;
    }

    return c.json({ success: true });
  })
  .get("/syncedReviewCount", async (c) => {
    const trieveKey = await validateTrieveAuth(c.req.raw);
    let tagsScanned = 0;
    let tagAmmount = 999999999999999;
    let page = 1;
    let page_size = 200;
    let done = false;

    while (!done || tagsScanned < tagAmmount) {
      const response = await fetch(
        `${getTrieveBaseUrlEnv()}/api/dataset/get_all_tags`,
        {
          method: "POST",
          headers: {
            Authorization: `Bearer ${trieveKey.key}`,
            "TR-Dataset": trieveKey.currentDatasetId,
            "Content-Type": "application/json",
          },
          body: JSON.stringify({
            page,
            page_size,
          }),
        },
      );

      if (!response.ok) {
        throw new Error("Failed to get synced review tag", {
          cause: await response.text(),
        });
      }

      const data = (await response.json()) as {
        tags: { count: number; tag: string }[];
        total: number;
      };

      if (data.tags.length === 0) {
        done = true;
        break;
      }

      const judgeTag = data.tags.find((t) => t.tag === "judge-me-review");
      if (judgeTag) {
        return c.json({ reviewCount: judgeTag?.count });
      }

      tagAmmount = data.total;
      tagsScanned += data.total;

      page += 1;
    }

    return c.json({ reviewCount: 0 });
  });

const transformReviewToChunk = (review: JudgeMeReview): ChunkReqPayload => {
  return {
    chunk_html: `This is a Review from: ${review.reviewer.name} from ${review.created_at}. This review was given a rating of ${review.rating}/5 stars.\n\n<h1>${review.title}</h1>\n\n<p>${review.body}.</p>`,
    group_tracking_ids: [review.product_external_id.toString()],
    upsert_by_tracking_id: true,
    metadata: {
      rating: review.rating,
    },
    tag_set: ["judge-me-review"],
    tracking_id: review.id.toString(),
  };
};
