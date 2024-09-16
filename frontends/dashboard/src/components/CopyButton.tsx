import { FiCheck, FiClipboard } from "solid-icons/fi";
import { createSignal, Show } from "solid-js";

interface CopyButtonProps {
  text: string;
  size?: number;
}
export const CopyButton = (props: CopyButtonProps) => {
  const [copied, setCopied] = createSignal(false);
  return (
    <button
      onClick={() => {
        void navigator.clipboard.writeText(props.text);
        setCopied(true);
        setTimeout(() => setCopied(false), 1000);
      }}
    >
      <Show when={copied()}>
        <FiCheck />
      </Show>
      <Show when={!copied()}>
        <FiClipboard class="text-gray-500" />
      </Show>
    </button>
  );
};
