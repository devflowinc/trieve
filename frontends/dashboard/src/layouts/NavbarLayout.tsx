import { JSX } from "solid-js";
import { A } from "@solidjs/router";
import { BiRegularLinkExternal } from "solid-icons/bi";

interface NavbarLayoutProps {
  children?: JSX.Element;
}
export const NavbarLayout = (props: NavbarLayoutProps) => {
  return (
    <div class="flex h-screen min-h-screen flex-col">
      <div class="flex justify-between gap-3 border-b border-b-neutral-300 p-2 px-4 shadow-md">
        <A
          href="/test"
          // href={`/dashboard/${
          //   userContext.selectedOrganizationId?.() ?? ""
          // }/overview`}
          class="flex items-center gap-1"
        >
          <img
            class="h-12 w-12 cursor-pointer"
            src="https://cdn.trieve.ai/trieve-logo.png"
            alt="Logo"
            // onClick={() => {
            //   navigator(
            //     `/dashboard/${
            //       userContext.selectedOrganizationId?.() ?? ""
            //     }/overview`,
            //   );
            // }}
          />
          <span class="text-2xl font-semibold">Trieve</span>
        </A>
        <div class="flex items-center justify-end gap-3">
          <a
            class="flex items-center gap-2 rounded-md border bg-neutral-100 px-2 py-1 text-sm focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-neutral-600"
            href="https://docs.trieve.ai"
            target="_blank"
          >
            <p>API Docs</p>
            <BiRegularLinkExternal class="opacity-80" />
          </a>
        </div>
      </div>
      <div class="flex grow flex-col overflow-scroll bg-neutral-100">
        {props.children}
      </div>
    </div>
  );
};
