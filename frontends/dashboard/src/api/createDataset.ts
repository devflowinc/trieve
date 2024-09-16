import { Dataset, DefaultError } from "shared/types";
import { DatasetConfigurationDTO } from "trieve-ts-sdk";

const api_host = import.meta.env.VITE_API_HOST as unknown as string;

export const createNewDataset = async ({
  name,
  organizationId,
  serverConfig,
}: {
  name: string;
  organizationId: string;
  serverConfig: DatasetConfigurationDTO;
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
      client_configuration: "{}",
    }),
  });
  if (response.ok) {
    return (await response.json()) as unknown as Dataset;
  } else {
    const error = (await response.json()) as DefaultError;
    throw new Error(error.message);
  }
};
