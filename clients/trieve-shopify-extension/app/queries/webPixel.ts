import { TrieveKey } from "app/types";

import { AdminApiCaller } from "app/loaders";

export const isWebPixelInstalled = async (adminApi: AdminApiCaller, trieveKey: TrieveKey) => {
    const response = await adminApi<{ webPixel: { id: string | null, settings: string } }>(
        `
      #graphql
      query {
            webPixel {
                id
                settings
            }
        }
      `,
    );

    if (response.error) {
        console.error(response.error);
        return false;
    }
    return response.data.webPixel.id !== null;
};

export const createWebPixel = async (adminApi: AdminApiCaller, trieveKey: TrieveKey) => {
    const response = await adminApi(
        `
      #graphql
      mutation webPixelCreate($webPixel: WebPixelInput!) {
        webPixelCreate(webPixel: $webPixel) {
          userErrors {
            code
            field
            message
          }
          webPixel {
            settings
            id
          }
        }
      }
      `,
        {
            variables: {
                webPixel: {
                    settings: `{"apiKey":"${trieveKey.key}", "datasetId": "${trieveKey.currentDatasetId}"}`,
                },
            },
        },
    );
    console.log("Web pixel created", response);
};