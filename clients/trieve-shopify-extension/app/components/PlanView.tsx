import { useSubmit } from "@remix-run/react";
import { BlockStack, Box, Button, Card, DescriptionList, DescriptionListProps, InlineStack, Text } from "@shopify/polaris";
import { ProgressBar } from "./ProgressBar";

export const PlanView = ({
    planItems,
    setShowCancelModal,
    usagePercentage,
}: {
    planItems: DescriptionListProps["items"];
    setShowCancelModal: (show: boolean) => void;
    usagePercentage: number;
}) => {
    const submit = useSubmit();

    return (
        <Card>
            <BlockStack gap="400">
                <Box paddingInline="400" paddingBlockStart="400">
                    <InlineStack align="space-between">
                        <Text variant="headingMd" as="h2">
                            Plan Details
                        </Text>
                        <div className="flex gap-2">
                            <Button
                                onClick={() => {
                                    submit({
                                        action: "modify",
                                    }, {
                                        method: "post",
                                    });
                                }}
                            >
                                Modify
                            </Button>
                            <Button
                                onClick={() => {
                                    setShowCancelModal(true);
                                }}
                            >
                                Cancel
                            </Button>
                        </div>
                    </InlineStack>
                </Box>

                <Box paddingInline="400" paddingBlockEnd="400">
                    <ProgressBar progress={usagePercentage} />
                    <DescriptionList items={planItems} />
                </Box>
            </BlockStack>
        </Card>
    );
};
