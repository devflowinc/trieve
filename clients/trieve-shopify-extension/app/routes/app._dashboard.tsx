// dashboard.tsx
import { LoaderFunctionArgs, redirect } from "@remix-run/node";
import { Outlet, useLoaderData } from "@remix-run/react";
import { Page, Layout, SkeletonBodyText } from "@shopify/polaris";
import { sdkFromKey, validateTrieveAuth } from "app/auth";
import {
  QueryClient,
  QueryClientProvider,
  HydrationBoundary,
} from "@tanstack/react-query";
import { TrieveProvider } from "app/context/trieveContext";
import { authenticate } from "app/shopify.server";
import { useDehydratedState } from "app/dehydrate";
import { StrongTrieveKey } from "app/types";
import { Dataset, OrganizationWithSubAndPlan } from "trieve-ts-sdk";
import { useState, Suspense } from "react";
import { ReactQueryDevtools } from "@tanstack/react-query-devtools";

export const loader = async (args: LoaderFunctionArgs) => {
  const { session } = await authenticate.admin(args.request);
  const key = await validateTrieveAuth(args.request, false);
  if (!key.currentDatasetId) {
    console.log("No dataset selected, redirecting to /app/setup");
    throw redirect("/app/setup");
  }

  const trieve = sdkFromKey(key);

  const dataset = await trieve.getDatasetByTrackingId(session.shop);
  const organization = await trieve.getOrganizationById(
    dataset.organization_id,
  );

  return {
    key: key as StrongTrieveKey,
    dataset,
    organization,
    shopDomain: session.shop,
  };
};

export default function Dashboard() {
  const { dataset, organization, key } = useLoaderData<typeof loader>();

  const [queryClient] = useState(
    () =>
      new QueryClient({
        defaultOptions: {
          queries: {
            // With SSR, we usually want to set some default staleTime
            // above 0 to avoid refetching immediately on the client
            staleTime: 6 * 1000,
          },
        },
      }),
  );
  const dehydratedState = useDehydratedState();

  return (
    <Page fullWidth>
      <Layout>
        <Layout.Section>
          <Suspense fallback={<SkeletonBodyText lines={3} />}>
            <TrieveProvider
              queryClient={queryClient}
              dataset={dataset as Dataset}
              organization={organization as OrganizationWithSubAndPlan}
              trieveKey={key}
            >
              <QueryClientProvider client={queryClient}>
                <ReactQueryDevtools initialIsOpen={false} />
                <HydrationBoundary state={dehydratedState}>
                  <div style={{ minHeight: "300px" }}>
                    <Outlet />
                  </div>
                </HydrationBoundary>
              </QueryClientProvider>
            </TrieveProvider>
          </Suspense>
        </Layout.Section>
      </Layout>
    </Page>
  );
}
