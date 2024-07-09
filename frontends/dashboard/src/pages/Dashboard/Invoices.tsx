import {
  createEffect,
  createMemo,
  createSignal,
  onCleanup,
  useContext,
  For,
} from "solid-js";
import { UserContext } from "../../contexts/UserContext";
import { createToast } from "../../components/ShowToasts";
import { StripeInvoice } from "../../types/apiTypes";
import { TbFileInvoice } from "solid-icons/tb";
import { formatDate, usdFormatter } from "../../formatters";

export const Invoices = () => {
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
    <div class="flex flex-col gap-4">
      <div class="space-y-1">
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
                  Link
                </th>
                <th scope="col" class="px-3 py-3.5" />
              </tr>
            </thead>
            <tbody class="divide-y divide-neutral-200 bg-white">
              <For each={orgInvoices()}>
                {(invoice) => {
                  return (
                    <tr>
                      <td class="whitespace-nowrap py-4 pl-4 pr-3 text-sm font-medium sm:pl-6">
                        {formatDate(new Date(invoice.created_at))}
                      </td>
                      <td class="whitespace-nowrap py-4 pl-4 pr-3 text-sm font-medium sm:pl-6">
                        {invoice.status}
                      </td>
                      <td class="whitespace-nowrap py-4 pl-4 pr-3 text-sm font-medium sm:pl-6">
                        {usdFormatter.format(invoice.total / 100)}
                      </td>
                      <td class="whitespace-nowrap py-4 pl-4 pr-3 text-sm font-medium sm:pl-6">
                        <a href={invoice.hosted_invoice_url}>
                          <TbFileInvoice size={15} />
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
  );
};
