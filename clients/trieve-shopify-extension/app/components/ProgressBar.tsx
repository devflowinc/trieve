import { Box, Text, InlineStack } from "@shopify/polaris";
import React from "react";

interface ProgressBarProps {
    progress: number;
}

export const ProgressBar: React.FC<ProgressBarProps> = ({ progress }) => {
    const progressPercentage = Math.max(0, Math.min(100, progress));

    return (
        <Box paddingBlockEnd="200">
            <div className="w-full bg-gray-200 rounded overflow-hidden h-2 mb-1">
                <div
                    className="bg-[#800080e6] h-full transition-width duration-300 ease-in-out"
                    style={{ width: `${progressPercentage}%` }}
                />
            </div>
            <InlineStack align="space-between" blockAlign="center" gap="0">
                <Text as="span" variant="bodySm" tone="subdued">0%</Text>
                <Text as="span" variant="bodySm" tone="subdued">25%</Text>
                <Text as="span" variant="bodySm" tone="subdued">50%</Text>
                <Text as="span" variant="bodySm" tone="subdued">75%</Text>
                <Text as="span" variant="bodySm" tone="subdued">100%</Text>
            </InlineStack>
        </Box>
    );
}; 