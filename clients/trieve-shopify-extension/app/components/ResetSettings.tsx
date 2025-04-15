import { Button, Card, Text } from "@shopify/polaris";
import { useMutation } from "@tanstack/react-query";
import { useClientAdminApi } from "app/loaders/clientLoader";
import {
  deleteMetafields,
  getPdpMetafields,
  MetafieldIdentifierInput,
} from "app/utils/productMetafields";

export const ResetSettings = () => {
  const adminApi = useClientAdminApi();

  const resetMetafieldsMutation = useMutation({
    onError: (e) => {
      console.error("Error clearing app metafields", e);
    },
    mutationFn: async () => {
      const products = await getPdpMetafields(adminApi);
      const productsWithTrieveMetafields = products.filter(
        (p) => p.metafields.nodes.length > 0,
      );
      const metafieldsToDelete = productsWithTrieveMetafields.reduce(
        (acc, p) => {
          const productId = p.id;
          return acc.concat(
            p.metafields.nodes.map((m) => ({
              key: m.key,
              ownerId: productId,
              namespace: m.namespace,
            })),
          );
        },
        [] as MetafieldIdentifierInput[],
      );
      if (metafieldsToDelete.length !== 0) {
        await deleteMetafields(adminApi, metafieldsToDelete);
      } else {
        console.log("No metafields to delete");
      }
    },
  });

  return (
    <Card>
      <Text variant="headingLg" as="h1">
        Reset Widgets and Onboarding
      </Text>
      <Text variant="bodyMd" as="p">
        Clears all app metafields, onboarding data, and widget configuration.
      </Text>
      <div className="h-3"></div>
      <div className="flex items-center gap-4">
        <Button
          onClick={() => {
            resetMetafieldsMutation.mutate();
          }}
          disabled={resetMetafieldsMutation.isPending}
          tone="critical"
        >
          Reset
        </Button>
        {resetMetafieldsMutation.error && (
          <div className="text-red-500">
            {resetMetafieldsMutation.error.message}
          </div>
        )}
        {resetMetafieldsMutation.isSuccess && (
          <div className="opacity-80">Successfully reset app!</div>
        )}
      </div>
    </Card>
  );
};
