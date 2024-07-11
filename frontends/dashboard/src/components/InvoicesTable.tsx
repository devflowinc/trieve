/* eslint-disable @typescript-eslint/no-unsafe-member-access */
import {
  createEffect,
  createMemo,
  createSignal,
  onCleanup,
  useContext,
  For,
  Show,
} from "solid-js";
import { UserContext } from "../contexts/UserContext";
import { createToast } from "./ShowToasts";
import { StripeInvoice } from "../types/apiTypes";
import { TbFileInvoice } from "solid-icons/tb";
import { formatDate, usdFormatter } from "../formatters";

export const InvoicesTable = () => {
  const api_host = import.meta.env.VITE_API_HOST as unknown as string;
  const selectedOrganization = createMemo(() => {
    const userContext = useContext(UserContext);
    const selectedOrgId = userContext.selectedOrganizationId?.();
    if (!selectedOrgId) return null;
    return (
      userContext.user?.()?.orgs.find((org) => org.id === selectedOrgId) ?? null
    );
  });

  const [orgInvoices, setOrgInvoices] = createSignal<StripeInvoice[]>([]);

  createEffect(() => {
    const selectedOrgId = selectedOrganization()?.id;
    if (!selectedOrgId) return;

    const orgInvoiceAbortController = new AbortController();
    void fetch(`${api_host}/stripe/invoices/${selectedOrgId}`, {
      credentials: "include",
      headers: {
        "TR-Organization": selectedOrgId,
      },
      signal: orgInvoiceAbortController.signal,
    })
      .then((res) => {
        if (res.status === 403) {
          createToast({
            title: "Error",
            type: "error",
            message:
              "It is likely that an admin or owner recently increased your role to admin or owner. Please sign out and sign back in to see the changes.",
            timeout: 10000,
          });
          return null;
        }

        return res.json();
      })
      .then((data) => {
        console.log(data);
        setOrgInvoices(data);
      });

    onCleanup(() => {
      orgInvoiceAbortController.abort();
    });
  });

  return (
    <Show when={orgInvoices().length > 0}>
      <div class="mb-8 flex flex-col gap-4">
        <div class="space-y-2">
          <h3 class="text-lg font-semibold text-neutral-800">
            Invoices for {selectedOrganization()?.name} Organization
          </h3>
          <div class="overflow-hidden rounded shadow ring-1 ring-black ring-opacity-5">
            <table class="min-w-full divide-y divide-neutral-300">
              <thead class="bg-neutral-100">
                <tr>
                  <th
                    scope="col"
                    class="py-3.5 pl-4 text-left text-sm font-semibold sm:pl-6"
                  >
                    Date
                  </th>
                  <th
                    scope="col"
                    class="px-3 py-3.5 text-left text-sm font-semibold"
                  >
                    Status
                  </th>
                  <th
                    scope="col"
                    class="px-3 py-3.5 text-left text-sm font-semibold"
                  >
                    Total
                  </th>
                  <th
                    scope="col"
                    class="px-3 py-3.5 text-left text-sm font-semibold"
                  >
                    View Stripe Invoice
                  </th>
                  <th scope="col" class="px-3 py-3.5" />
                </tr>
              </thead>
              <tbody class="divide-y divide-neutral-200 bg-white">
                <For each={orgInvoices()}>
                  {(invoice) => {
                    return (
                      <tr>
                        <td class="whitespace-nowrap py-4 pl-4 pr-3 text-sm font-medium">
                          {formatDate(new Date(invoice.created_at))}
                        </td>
                        <td class="whitespace-nowrap py-4 pl-4 pr-3 text-sm font-medium">
                          {invoice.status}
                        </td>
                        <td class="whitespace-nowrap py-4 pl-4 pr-3 text-sm font-medium">
                          {usdFormatter.format(invoice.total / 100)}
                        </td>
                        <td class="whitespace-nowrap py-4 pl-4 pr-3 text-sm font-medium">
                          <a
                            href={invoice.hosted_invoice_url}
                            target="_blank"
                            class="hover:text-fuchsia-500"
                          >
                            <TbFileInvoice class="h-6 w-6" />
                          </a>
                        </td>
                      </tr>
                    );
                  }}
                </For>
              </tbody>
            </table>
          </div>
        </div>
      </div>
    </Show>
  );
};
