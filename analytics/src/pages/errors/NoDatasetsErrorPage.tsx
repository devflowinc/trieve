interface NoDatasetsErrorPageProps {
  orgId: string;
}
export const NoDatasetsErrorPage = (props: NoDatasetsErrorPageProps) => {
  const dashboardLink = import.meta.env.VITE_DASHBOARD_URL;
  return (
    <div class="grid place-items-center grow">
      <div class="bg-white text-center border border-neutral-200 p-4">
        <div>You Have No Datasets</div>
        <a
          target="_blank"
          href={`${dashboardLink}/dashboard/${props.orgId}/overview`}
        >
          Create A New One
        </a>
      </div>
    </div>
  );
};
