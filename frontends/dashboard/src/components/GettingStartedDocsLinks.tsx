import { IconTypes } from "solid-icons";
import {
  AiFillApi,
  AiFillCaretRight,
  AiOutlineShoppingCart,
} from "solid-icons/ai";
import { RiBusinessPresentationFill } from "solid-icons/ri";

interface GettingStartedDocsLinksProps {
  icon: IconTypes;
  link: string;
  title: string;
  description: string;
}
const GettingStartedLink = (props: GettingStartedDocsLinksProps) => {
  return (
    <a
      href={props.link}
      target="_blank"
      class="flex flex-col gap-1 rounded-md border border-neutral-200 bg-white p-4 shadow-sm transition-all hover:border-magenta-300 hover:shadow-md"
    >
      <div>
        {props.icon({
          class: "text-magenta-200",
          size: "1.5rem",
        })}
      </div>
      <div class="pt-2 text-lg">{props.title}</div>
      <div class="text-[14px] text-neutral-400">{props.description}</div>
    </a>
  );
};

export const GettingStartedDocsLinks = () => {
  return (
    <div>
      <div class="pb-2 font-medium">Introduction</div>
      <div class="grid grid-cols-2 gap-3 -md:grid-cols-1">
        <GettingStartedLink
          icon={AiFillApi}
          link="https://docs.trieve.ai/api-reference"
          description="Check out the API Reference to see all of the available endpoints and options for integrating Trieve into your application."
          title="API Reference"
        />
        <GettingStartedLink
          icon={AiFillCaretRight}
          link="https://docs.trieve.ai/getting-started/introduction"
          description="Get started with Trieve quickly"
          title="Getting Started"
        />
        <GettingStartedLink
          icon={RiBusinessPresentationFill}
          link="https://docs.trieve.ai/examples/job-board"
          description="Learn how to build a search experience for a job board using Trieve."
          title="Build Search for a Job Board"
        />
        <GettingStartedLink
          icon={AiOutlineShoppingCart}
          link="https://docs.trieve.ai/examples/ecommerce"
          description="Learn how to build a search experience for an ecommerce platform using Trieve."
          title="Build Search for Ecommerce"
        />
      </div>
    </div>
  );
};
