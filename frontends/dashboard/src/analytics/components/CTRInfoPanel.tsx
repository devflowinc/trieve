import { Show } from "solid-js";
import { useCTRNeedsSetup } from "../hooks/useCTRNeedsSetup";

export const CTRInfoPanel = () => {
  const ctrNeedsSetup = useCTRNeedsSetup();
  return (
    <Show when={ctrNeedsSetup()}>
      <div class="rounded border border-blue-300 bg-blue-100/60 p-4 text-blue-900">
        <div>
          Note: Click-through rate analytics have not been set up for this
          dataset yet. Data will be viewable as soon as you begin sending
          events. Please visit the{" "}
          <a
            class="underline"
            href="https://docs.trieve.ai/api-reference/analytics/send-ctr-data"
          >
            Analytics API reference
          </a>{" "}
          for help sending click-through rate data.
        </div>
      </div>
    </Show>
  );
};
