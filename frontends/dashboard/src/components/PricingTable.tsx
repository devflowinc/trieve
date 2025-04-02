import { BillingEstimate, OrganizationWithSubAndPlan } from "trieve-ts-sdk";
import { createEffect, createSignal, For, onCleanup, Show } from "solid-js";
import { formatDateForApi } from "../analytics/utils/formatDate";
import { usdFormatter } from "../utils/formatters";
import { addMonths } from "date-fns";

export interface PricingTableProps {
  currentOrgSubPlan: OrganizationWithSubAndPlan | null;
}

export const PricingTable = (props: PricingTableProps) => {
  const apiHost = import.meta.env.VITE_API_HOST as unknown as string;

  const [billingEstimate, setBillingEstimate] =
    createSignal<BillingEstimate | null>(null);

  const [startOfBill, setStartOfBill] = createSignal<Date>(new Date());

  const names_to_guages = [
    {
      name: "Chunk Storage (mb)",
      gauge: "chunk_storage_mb",
    },
    {
      name: "File Storage (mb)",
      gauge: "file_storage_mb",
    },
    {
      name: "Users",
      gauge: "users",
    },
    {
      name: "Datasets",
      gauge: "dataset_count",
    },
    {
      name: "Search Tokens",
      gauge: "search_tokens",
    },
    {
      name: "Message Tokens",
      gauge: "message_tokens",
    },
    {
      name: "Bytes Ingested",
      gauge: "bytes_ingested",
    },
    {
      name: "Tokens Ingested",
      gauge: "tokens_ingested",
    },
    {
      name: "Pages Crawled",
      gauge: "pages_crawled",
    },
    {
      name: "OCR Pages",
      gauge: "ocr_pages",
    },
    {
      name: "Analytics Events",
      gauge: "analytics_events",
    },
  ];

  createEffect(() => {
    const availablePlansAbortController = new AbortController();

    console.log(props.currentOrgSubPlan);
    if (
      props.currentOrgSubPlan?.subscription?.type === "usage_based" &&
      props.currentOrgSubPlan?.plan?.id
    ) {
      const startOfBill = formatDateForApi(
        new Date(
          `${props.currentOrgSubPlan?.subscription.last_cycle_timestamp}Z`,
        ),
      );
      setStartOfBill(new Date(startOfBill));
      void fetch(
        `${apiHost}/stripe/estimate_bill/${props.currentOrgSubPlan?.plan?.id}`,
        {
          credentials: "include",
          method: "POST",
          headers: {
            "TR-Organization": props.currentOrgSubPlan?.organization.id ?? "",
            "Content-Type": "application/json",
          },
          body: JSON.stringify({
            date_range: {
              gte: formatDateForApi(
                new Date(
                  props.currentOrgSubPlan?.subscription.last_cycle_timestamp,
                ),
              ),
            },
          }),
          signal: availablePlansAbortController.signal,
        },
      )
        .then((res) => res.json())
        .then((data) => {
          setBillingEstimate(data as BillingEstimate);
        });
    }

    onCleanup(() => {
      availablePlansAbortController.abort();
    });
  });

  return (
    <Show when={billingEstimate()}>
      <div class="overflow-hidden pb-8">
        <h3 class="text-lg font-semibold text-neutral-800">
          Upcoming Bill For{" "}
          {startOfBill().toLocaleString("default", {
            month: "long",
            day: "numeric",
            year: "numeric",
          })}{" "}
          -{" "}
          {addMonths(startOfBill(), 1).toLocaleString("default", {
            month: "long",
            day: "numeric",
            year: "numeric",
          })}
        </h3>
        <table class="min-w-full divide-y divide-neutral-300 rounded border shadow ring-1 ring-black ring-opacity-5">
          <thead class="bg-neutral-50">
            <tr>
              <td class="whitespace-nowrap py-4 pl-4 pr-3 text-sm font-medium">
                Item
              </td>
              <td class="whitespace-nowrap py-4 pl-4 pr-3 text-sm font-medium">
                Price
              </td>
            </tr>
          </thead>
          <tbody class="divide-y divide-neutral-200 bg-white">
            <For each={billingEstimate()?.items}>
              {(item) => {
                return (
                  <tr>
                    <td class="whitespace-nowrap py-4 pl-4 pr-3 text-sm font-medium">
                      {
                        names_to_guages.find(
                          (gauge) => gauge.gauge === item.name,
                        )?.name
                      }
                    </td>
                    <td class="whitespace-nowrap py-4 pl-4 pr-3 text-sm font-medium">
                      {usdFormatter.format(item.amount)}
                    </td>
                  </tr>
                );
              }}
            </For>
            <tr>
              <td class="whitespace-nowrap py-4 pl-4 pr-3 text-sm font-medium">
                Total
              </td>
              <td class="whitespace-nowrap py-4 pl-4 pr-3 text-sm font-medium">
                {usdFormatter.format(billingEstimate()!.total)}
              </td>
            </tr>
          </tbody>
        </table>
      </div>
    </Show>
  );
};
