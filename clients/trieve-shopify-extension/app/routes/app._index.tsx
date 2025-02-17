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
  SkeletonBodyText,
  Select,
  Button,
  InlineStack,
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

export function HydrateFallback() {
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
                  <SkeletonBodyText lines={8} />
                </BlockStack>
              </Card>
            </BlockStack>
          </Layout.Section>
        </Layout>
      </BlockStack>
    </Page>
  );
}

export default function Index() {
  const datasets = useLoaderData<typeof clientLoader>();

  let chosenDataset = datasets[0].dataset;

  return (
    <Page>
      <BlockStack gap="500">
        <Layout>
          <Layout.Section variant="oneHalf">
            <BlockStack gap="500">
              <Card>
                <InlineStack gap="200" align="space-between">
                  <Text as="h2" variant="headingMd">
                    Indexed Dataset: {chosenDataset.name}
                  </Text>

                  <Link to="/app/dataset">
                    <Button>
                      Edit settings
                    </Button>
                  </Link>
                </InlineStack>
              </Card>
            </BlockStack>
          </Layout.Section>
        </Layout>
      </BlockStack>
    </Page>
  );
}
