import { useTrieve } from "app/context/trieveContext";
import { k } from "node_modules/vite/dist/node/types.d-aGj9QkWt";
import { TrieveModalSearch } from "trieve-search-component";
import "trieve-search-component/styles";


export function ChatHistoryView() {
  // only render on client
  if (typeof window === "undefined") {
    return null;
  }
  const { dataset, trieveKey } = useTrieve();

  return (
    <TrieveModalSearch
      type="ecommerce"
      defaultSearchMode="chat"
      allowSwitchingModes={false}
      defaultAiQuestions={[
        "What is snow",
        "I want a rug",
        "I want a chair",
      ]}
      apiKey={trieveKey.key}
      datasetId={dataset.id}
      inline={true}
      debounceMs={10}
    />
  )
}
