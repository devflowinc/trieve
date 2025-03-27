// app/routes/app.chat.$id.tsx
import { LoaderFunctionArgs } from '@remix-run/node';
import { useLoaderData } from '@remix-run/react';
import { Box, Card, InlineStack, Text } from '@shopify/polaris';
import { ChatHistoryView } from 'app/components/analytics/chat/ChatHistoryView';
import { Suspense } from 'react';

// Loader function to fetch chat data based on the ID
export async function loader({ params }: LoaderFunctionArgs) {
  const chatId = params.topic;

  // Example: Fetch chat data (replace with your actual data fetching logic)
  return { chatId };
}

export default function ChatRoute() {
  const { chatId } = useLoaderData<typeof loader>();

  return (
    <div>
      <Card>
        <InlineStack align="space-between">
          <Text variant="headingLg" as="h2">
            Chat History
          </Text>
          <Suspense fallback={null}>
            <ChatHistoryView />
          </Suspense>
        </InlineStack>
      </Card>
    </div>
  );
}
