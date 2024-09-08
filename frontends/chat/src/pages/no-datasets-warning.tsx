import { FiExternalLink } from "solid-icons/fi";

interface NoDatasetsErrorPageProps {
  orgId: string | undefined;
}
export const NoDatasetsErrorPage = (props: NoDatasetsErrorPageProps) => {
  const dashboardLink = import.meta.env.VITE_DASHBOARD_URL as string;
  return (
    <div class="grid grow place-items-center pt-80">
      <div class="flex flex-col items-center rounded border border-neutral-200 bg-white p-4 text-center dark:border-neutral-700 dark:bg-neutral-800 dark:text-white">
        <div>You have no datasets in this organization to chat with.</div>
        <a
          class="mt-2 flex items-center gap-2 rounded border border-neutral-300 bg-neutral-200 p-2 text-sm text-white hover:underline dark:border-neutral-700 dark:bg-neutral-800"
          target="_blank"
          href={`${dashboardLink}/dashboard/${props.orgId}/overview`}
        >
          Create New Dataset
          <FiExternalLink />
        </a>
      </div>
    </div>
  );
};
