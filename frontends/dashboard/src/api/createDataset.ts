import { Dataset, DefaultError } from "shared/types";
import { CrawlOptions, DatasetConfigurationDTO } from "trieve-ts-sdk";

const api_host = import.meta.env.VITE_API_HOST as unknown as string;

export const createNewDataset = async ({
  name,
  organizationId,
  serverConfig,
  crawlOptions,
}: {
  name: string;
  organizationId: string;
  serverConfig: DatasetConfigurationDTO;
  crawlOptions?: CrawlOptions;
}) => {
  const response = await fetch(`${api_host}/dataset`, {
    method: "POST",
    credentials: "include",
    headers: {
      "Content-Type": "application/json",
      "TR-Organization": organizationId,
    },
    body: JSON.stringify({
      dataset_name: name,
      organization_id: organizationId,
      server_configuration: serverConfig,
      // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment
      crawl_options: crawlOptions,
    }),
  });
  if (response.ok) {
    return (await response.json()) as unknown as Dataset;
  } else {
    const error = (await response.json()) as DefaultError;
    throw new Error(error.message);
  }
};
