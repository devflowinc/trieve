import { LoaderFunctionArgs, redirect } from "@remix-run/node";
import { validateTrieveAuth } from "app/auth";
import {
  defaultCrawlOptions,
  ExtendedCrawlOptions,
} from "app/components/DatasetSettings";
import { buildAdminApiFetcherForServer } from "app/loaders/serverLoader";
import { sendChunks } from "app/processors/getProducts";
import { authenticate } from "app/shopify.server";

export type AppInstallData = {
  currentAppInstallation: { id: string };
};

export const loader = async (args: LoaderFunctionArgs) => {
  const { session } = await authenticate.admin(args.request);
  let key = await validateTrieveAuth(args.request, true);

  let crawlSettings = await prisma.crawlSettings.findFirst({
    where: {
      datasetId: key.currentDatasetId,
      shop: session.shop,
    },
  });

  if (!crawlSettings) {
    crawlSettings = await prisma.crawlSettings.create({
      data: {
        datasetId: key.currentDatasetId,
        shop: session.shop,
        crawlSettings: defaultCrawlOptions,
      },
    });
  }
  const crawlOptions = crawlSettings.crawlSettings as ExtendedCrawlOptions;

  await prisma.crawlSettings.upsert({
    create: {
      datasetId: key.currentDatasetId,
      shop: session.shop,
      crawlSettings: crawlOptions,
    },
    update: {
      crawlSettings: crawlOptions,
    },
    where: {
      datasetId_shop: {
        datasetId: key.currentDatasetId,
        shop: session.shop,
      },
    },
  });

  const fetcher = buildAdminApiFetcherForServer(
    session.shop,
    session.accessToken!,
  );
  sendChunks(key.currentDatasetId ?? "", key, fetcher, session, crawlOptions).catch(
    console.error,
  );

  return redirect("/app");
};
