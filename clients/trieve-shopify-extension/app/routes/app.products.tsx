import { LoaderFunctionArgs } from "@remix-run/node";
import { Link, useLoaderData } from "@remix-run/react";
import { Page, Text, Link as PolLink, Box, Divider } from "@shopify/polaris";
import { authenticate } from "app/shopify.server";

export const loader = async ({ request }: LoaderFunctionArgs) => {
  const { admin } = await authenticate.admin(request);

  // Query For products from shopify api
  const response = await admin.graphql(
    `#graphql
  query {
    products(first: 10) {
      nodes {
        id
        title
        productType
        bodyHtml
        handle
        tags
        variants(first: 20) {
          nodes {
            displayName
            price
            title
          }
        }
        media(first: 20) {
          nodes {
            preview {
              image {
                url
              }
            }
          }
        }
      }
      pageInfo {
        hasNextPage
      }
    }
  }`,
  );

  const { data } = await response.json();

  // TODO send chunks to Trieve


  return {
    data
  };
};

export default function GetProduct() {
  const { data } = useLoaderData<typeof loader>();

  const renderValue = (value: any) => {
    if (typeof value === 'object' && value !== null) {
      if (Array.isArray(value)) {
        return (
          <ul>
            {value.map((item, index) => (
              <li key={index}>
                {typeof item === 'object' ? renderObject(item) : item}
              </li>
            ))}
          </ul>
        );
      } else {
        return renderObject(value);
      }
    }
    return value;
  };

  const renderObject = (obj: any) => {
    return (
      <ul>
        {Object.entries(obj).map(([key, value]) => (
          <li key={key}>
            <strong>{key}:</strong> {renderValue(value)}
          </li>
        ))}
      </ul>
    );
  };

  return (
    <Page>
      <Link to={`/app`}>
        <Box paddingBlockEnd="200">
          <PolLink>Back To Datasets</PolLink>
        </Box>
      </Link>
      <Text variant="headingXl" as="h2">
        These are your first 10 products displayed in order.
      </Text>
      <Box paddingBlockStart="400">
        <div>
                <Divider borderColor="border-inverse"/>
          {...data.products.nodes.map((product: any) => {
            return (
              <div>
                <div>
                  {renderObject(product)}
                </div>
                <Divider borderColor="border-inverse"/>
              </div>
            );
          })}
        </div>
      </Box>
    </Page>
  );
}
