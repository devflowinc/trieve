import { Show, useContext } from "solid-js";
import {
  BsCalendar,
  BsDiscord,
  BsEnvelope,
  BsGithub,
  BsQuestionCircle,
  BsSunglasses,
} from "solid-icons/bs";
import { Popover, PopoverButton, PopoverPanel } from "terracotta";
import { A, useNavigate } from "@solidjs/router";
import { UserContext } from "../contexts/UserContext";
import { BiRegularLinkExternal } from "solid-icons/bi";

export const NavBar = () => {
  const userContext = useContext(UserContext);
  const navigator = useNavigate();

  return (
    <div class="flex justify-between gap-3">
      <A
        href={`/dashboard/${
          userContext.selectedOrganizationId?.() ?? ""
        }/overview`}
        class="flex items-center gap-1"
      >
        <img
          class="h-12 w-12 cursor-pointer"
          src="https://cdn.trieve.ai/trieve-logo.png"
          alt="Logo"
          onClick={() => {
            navigator(
              `/dashboard/${
                userContext.selectedOrganizationId?.() ?? ""
              }/overview`,
            );
          }}
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
        <Popover
          id="help-or-contact-popover"
          defaultOpen={false}
          class="relative flex items-center"
        >
          {({ isOpen }) => (
            <>
              <PopoverButton
                aria-label="Show help or contact"
                class="flex items-center gap-2 rounded-md border bg-neutral-100 px-2 py-1 text-sm focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-neutral-600"
              >
                <BsQuestionCircle class="h-3 w-3" />
                <span>Help or Contact</span>
              </PopoverButton>
              <Show when={isOpen()}>
                <PopoverPanel class="absolute right-0 top-full z-10 mt-1 w-fit min-w-[350px] space-y-2 rounded-md border bg-white px-3 py-2 shadow-lg">
                  <div class="text-nowrap">
                    <h5 class="text-lg">Need help or just want to chat?</h5>
                    <p class="mb-2 text-sm text-neutral-800">
                      Expected performance is based on your billing plan. Paid
                      projects are prioritized.
                    </p>
                    <div class="flex gap-2">
                      <a
                        href="mailto:humans@trieve.ai"
                        class="flex w-fit items-center space-x-1 rounded-md border-[0.5px] border-magenta-200 bg-magenta-50 px-2 py-1 text-sm font-medium"
                      >
                        <BsEnvelope class="h-3 w-3" />
                        <p>humans@trieve.ai</p>
                      </a>
                      <a
                        href="httsp://cal.com/nick.k"
                        class="flex w-fit items-center space-x-1 rounded-md border-[0.5px] border-magenta-200 bg-magenta-50 px-2 py-1 text-sm font-medium"
                      >
                        <BsCalendar class="h-3 w-3" />
                        <p>Talk to a Founder</p>
                      </a>
                    </div>
                  </div>
                  <div>
                    <h5 class="mt-3 text-lg">Reach out to the community</h5>
                    <p class="text-sm text-neutral-800">
                      Welcoming space for other support or advice, including
                      questions on API concepts, or best practices.
                    </p>
                    <div class="my-2 flex items-center space-x-2 text-sm">
                      <a
                        href="https://matrix.to/#/#trieve-general:trieve.ai"
                        class="flex w-fit items-center space-x-1 rounded-md border-[0.5px] border-green-100 bg-green-50 px-2 py-1 text-sm font-medium text-green-900"
                      >
                        <BsSunglasses class="h-3 w-3" />
                        <p>Join Matrix server</p>
                      </a>
                      <a
                        href="https://github.com/devflowinc/trieve/issues"
                        class="flex w-fit items-center space-x-1 rounded-md border-[0.5px] border-sky-100 bg-sky-50 px-2 py-1 text-sm font-medium text-sky-900"
                      >
                        <BsGithub class="h-3 w-3" />
                        <p>GitHub Issues</p>
                      </a>
                    </div>
                    <a
                      href="https://discord.gg/E9sPRZqpDT"
                      class="flex w-fit items-center space-x-1 rounded-md border-[0.5px] border-blue-100 bg-blue-50 px-2 py-1 text-sm font-medium text-blue-900"
                    >
                      <BsDiscord class="h-3 w-3" />
                      <p>Join Discord server</p>
                    </a>
                  </div>
                </PopoverPanel>
              </Show>
            </>
          )}
        </Popover>
      </div>
    </div>
  );
};

export default NavBar;
