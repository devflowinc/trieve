import type { LoaderFunctionArgs } from "@remix-run/node";
import {
  ClientLoaderFunctionArgs,
  json,
  Link,
  useLoaderData,
} from "@remix-run/react";
import {
  Page,
  Layout,
  Text,
  Link as PolarisLink,
  Card,
  BlockStack,
  List,
  SkeletonBodyText,
  Box,
} from "@shopify/polaris";
import { validateTrieveAuth } from "app/auth";

export const clientLoader = async (args: ClientLoaderFunctionArgs) => {
  try {
    const key = await args.serverLoader<typeof loader>();
    if (!key) {
      throw json({ message: "No Key" }, 401);
    }
    const datasetsResponse = await fetch(
      `https://api.trieve.ai/api/dataset/organization/${key.organizationId}`,
      {
        headers: {
          Authorization: `Bearer ${key.key}`,
          "TR-Organization": key.organizationId ?? "",
        },
      },
    );
    const datasets = await datasetsResponse.json();
    return datasets;
  } catch (e) {
    if (e instanceof Error && e.message != "No Key") {
      throw json({ message: e.message }, 401);
    }
    throw json({ message: "No Key" }, 401);
  }
};

clientLoader.hydrate = true;

export const loader = async (args: LoaderFunctionArgs) => {
  const key = await validateTrieveAuth(args);
  return key;
};

function PageLayout({ children }: { children: React.ReactNode }) {
  return (
    <Page>
      <BlockStack gap="500">
        <Layout>
          <Layout.Section variant="oneHalf">
            <BlockStack gap="500">
              <Card>
                <BlockStack gap="200">
                  <Text as="h2" variant="headingMd">
                    Select Dataset
                  </Text>
                  {children}
                </BlockStack>
              </Card>
            </BlockStack>
          </Layout.Section>
        </Layout>
      </BlockStack>
    </Page>
  );
}

export function HydrateFallback() {
  return (
    <PageLayout>
      <SkeletonBodyText lines={8} />
    </PageLayout>
  );
}

export default function Index() {
  const datasets = useLoaderData<typeof clientLoader>();
  return (
    <Page>
      <BlockStack gap="500">
        <Layout>
          <Layout.Section variant="oneHalf">
            <BlockStack gap="500">
              <Card>
                <BlockStack gap="200">
                  <Text as="h2" variant="headingMd">
                    Select Dataset
                  </Text>
                  <Box>
                    {datasets.length > 0 ? (
                      <List>
                        {datasets.map((dataset: any) => (
                          <List.Item key={dataset.dataset.id}>
                            <Link
                              to={`/app/dataset/${dataset.dataset.id}/products`}>
                              <PolarisLink>{dataset.dataset.name}</PolarisLink>
                            </Link>
                          </List.Item>
                        ))}
                      </List>
                    ) : (
                      <Text as="p" variant="bodyMd">
                        No datasets available.
                      </Text>
                    )}
                  </Box>
                </BlockStack>
              </Card>
            </BlockStack>
          </Layout.Section>
        </Layout>
      </BlockStack>
    </Page>
  );
}
