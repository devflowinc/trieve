import type { LoaderFunctionArgs } from "@remix-run/node";
import { useLoaderData } from "@remix-run/react";
import { Page, Layout, Text, Card, BlockStack, List } from "@shopify/polaris";
import { TitleBar } from "@shopify/app-bridge-react";
import { validateTrieveAuth } from "app/auth";

export const loader = async (args: LoaderFunctionArgs) => {
  const key = await validateTrieveAuth(args);
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
  return datasets as any[];
};

export default function Index() {
  const datasets = useLoaderData<typeof loader>();
  return (
    <Page>
      <TitleBar title="Remix app template"></TitleBar>
      <BlockStack gap="500">
        <Layout>
          <Layout.Section variant="oneHalf">
            <BlockStack gap="500">
              <Card>
                <BlockStack gap="200">
                  <Text as="h2" variant="headingMd">
                    Datasets
                  </Text>
                  {datasets.length > 0 ? (
                    <List>
                      {datasets.map((dataset) => (
                        <List.Item key={dataset.dataset.id}>
                          {dataset.dataset.name}
                        </List.Item>
                      ))}
                    </List>
                  ) : (
                    <Text as="p" variant="bodyMd">
                      No datasets available.
                    </Text>
                  )}
                </BlockStack>
              </Card>
            </BlockStack>
          </Layout.Section>
        </Layout>
      </BlockStack>
    </Page>
  );
}
