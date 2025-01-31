import { Card, BlockStack, Text, Box, Button } from "@shopify/polaris";
import { useRevalidator } from "@remix-run/react";
import { useEffect } from "react";
import { useAppBridge } from "@shopify/app-bridge-react";

export const MustLoginPage = ({ authUrl }: { authUrl: string }) => {
  const shopify = useAppBridge();
  const revalidator = useRevalidator();

  useEffect(() => {
    const handleFocus = () => {
      if (revalidator.state === "idle") {
        revalidator.revalidate();
      }
    };
    window.addEventListener("focus", handleFocus);
    return () => {
      window.removeEventListener("focus", handleFocus);
    };
  }, []);

  const navigateToAuth = () => {
    shopify.idToken().then((token) => {
      window.open(`${authUrl}?token=${token}`, "_blank");
    });
  };

  return (
    <Box padding="1000">
      <Card background="bg-surface-caution">
        <BlockStack gap="500">
          <BlockStack gap="200">
            <Text as="h2" variant="headingMd">
              Login to Trieve
            </Text>
            <Text variant="bodyMd" as="p">
              You must be logged in to use this page.
            </Text>
            <Button onClick={navigateToAuth}>Login With Trieve</Button>
          </BlockStack>
        </BlockStack>
      </Card>
    </Box>
  );
};
