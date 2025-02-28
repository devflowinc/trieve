import { Box } from "@shopify/polaris";
import { useSuspenseQuery } from "@tanstack/react-query";
import { createServerLoader } from "app/loaders/serverLoader";
import { Loader } from "app/loaders";
import { createClientLoader } from "app/loaders/clientLoader";

const load: Loader = async ({ queryClient }) => {
  await queryClient.ensureQueryData({
    queryKey: ["test"],
    queryFn: async () => "fromserver",
  });
};

export const loader = createServerLoader(load);
export const clientLoader = createClientLoader(load);

export default function Dataset() {
  const { data } = useSuspenseQuery({
    queryKey: ["test"],
    queryFn: async () => "fromclient",
  });

  return <Box paddingBlockStart="400">Search Analytics Page: {data}</Box>;
}
