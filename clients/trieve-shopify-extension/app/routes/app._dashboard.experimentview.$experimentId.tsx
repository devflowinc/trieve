import { AnalyticsQueryBuilder } from "trieve-ts-sdk";
import { useTrieveServer } from "app/auth";
import { LoaderFunctionArgs } from "@remix-run/node";
import { ExperimentView } from "app/components/ExperimentView";

interface UserCountResult {
  treatment_name: string;
  user_count: number;
}

interface TreatmentEventResult {
  treatment_name: string;
  event_name: string;
  event_count: number;
}

interface ConvertedUsersResult {
  treatment_name: string;
  converted_user_count: number;
}

interface TotalConversionEventsResult {
  treatment_name: string;
  total_conversion_event_count: number;
}

export async function loader({ request, params }: LoaderFunctionArgs) {
  const { trieve } = await useTrieveServer(request);
  const experiment = await trieve.getExperiment(params.experimentId as string);
  const userCountsQuery = new AnalyticsQueryBuilder()
    .select("treatment_name")
    .select("user_id", {
      aggregation: "COUNT",
      alias: "user_count",
      distinct: true,
    })
    .from("experiment_user_assignments")
    .where({
      column: "experiment_id",
      operator: "=",
      value: experiment.id,
    })
    .groupBy(["treatment_name"])
    .build();

  const checkoutsQuery = new AnalyticsQueryBuilder()
    .select("experiment_user_assignments.treatment_name", {
      alias: "treatment_name",
    })
    .select("events.event_name", {
      alias: "event_name",
    })
    .select("events.user_id", {
      aggregation: "COUNT",
      alias: "event_count",
    })
    .from("events")
    .joinOn(
      "experiment_user_assignments",
      "experiment_user_assignments.user_id",
      "events.user_id",
    )
    .where({
      column: "events.event_name",
      operator: "=",
      value: "site-checkout_end",
      or_filter: [
        {
          column: "events.event_name",
          operator: "=",
          value: "site-add_to_cart",
        },
        {
          column: "events.event_name",
          operator: "=",
          value: "Click",
        },
      ],
    })
    .where({
      column: "events.created_at",
      operator: ">",
      value: new Date(experiment.created_at)
        .toISOString()
        .slice(0, 19)
        .replace("T", " "),
    })
    .where({
      column: "experiment_user_assignments.experiment_id",
      operator: "=",
      value: experiment.id,
    })
    .groupBy([
      "experiment_user_assignments.treatment_name",
      "events.event_name",
    ])
    .build();

  const conversionRateQuery = new AnalyticsQueryBuilder()
    .select("experiment_user_assignments.treatment_name", {
      alias: "treatment_name",
    })
    .select("events.user_id", {
      aggregation: "COUNT",
      alias: "converted_user_count",
      distinct: true,
    })
    .from("events")
    .joinOn(
      "experiment_user_assignments",
      "experiment_user_assignments.user_id",
      "events.user_id",
    )
    .where({
      column: "experiment_user_assignments.experiment_id",
      operator: "=",
      value: experiment.id,
    })
    .where({
      column: "events.is_conversion",
      operator: "=",
      value: true,
    })
    .where({
      column: "events.event_name",
      operator: "!=",
      value: "component_open",
      and_filter: [
        {
          column: "events.event_name",
          operator: "!=",
          value: "component_close",
        },
        {
          column: "events.event_name",
          operator: "!=",
          value: "site-followup_query",
        },
      ],
    })
    .where({
      column: "events.created_at",
      operator: ">",
      value: new Date(experiment.created_at)
        .toISOString()
        .slice(0, 19)
        .replace("T", " "),
    })
    .groupBy(["experiment_user_assignments.treatment_name"])
    .build();

  const totalConversionEventsQuery = new AnalyticsQueryBuilder()
    .select("experiment_user_assignments.treatment_name", {
      alias: "treatment_name",
    })
    .select("events.id", {
      aggregation: "COUNT",
      alias: "total_conversion_event_count",
    })
    .from("events")
    .joinOn(
      "experiment_user_assignments",
      "experiment_user_assignments.user_id",
      "events.user_id",
    )
    .where({
      column: "experiment_user_assignments.experiment_id",
      operator: "=",
      value: experiment.id,
    })
    .where({
      column: "events.is_conversion",
      operator: "=",
      value: true,
    })
    .where({
      column: "events.event_name",
      operator: "!=",
      value: "component_open",
      and_filter: [
        {
          column: "events.event_name",
          operator: "!=",
          value: "component_close",
        },
        {
          column: "events.event_name",
          operator: "!=",
          value: "site-followup_query",
        },
      ],
    })
    .where({
      column: "events.created_at",
      operator: ">",
      value: new Date(experiment.created_at)
        .toISOString()
        .slice(0, 19)
        .replace("T", " "),
    })
    .groupBy(["experiment_user_assignments.treatment_name"])
    .build();

  const [
    userCountsResult,
    treatmentEventsResult,
    convertedUsersResult,
    totalConversionEventsDataResult,
  ] = await Promise.all([
    trieve.getAnalytics<UserCountResult[]>(userCountsQuery),
    trieve.getAnalytics<TreatmentEventResult[]>(checkoutsQuery),
    trieve.getAnalytics<ConvertedUsersResult[]>(conversionRateQuery),
    trieve.getAnalytics<TotalConversionEventsResult[]>(
      totalConversionEventsQuery,
    ),
  ]);

  const conversionStats = userCountsResult.map((userCount) => {
    const uniqueConvertedUserData = convertedUsersResult.find(
      (convertedUser) =>
        convertedUser.treatment_name === userCount.treatment_name,
    );
    const converted_unique_users = uniqueConvertedUserData
      ? uniqueConvertedUserData.converted_user_count
      : 0;

    const totalConversionEventsData = totalConversionEventsDataResult.find(
      (totalConversions) =>
        totalConversions.treatment_name === userCount.treatment_name,
    );
    const total_conversion_events = totalConversionEventsData
      ? totalConversionEventsData.total_conversion_event_count
      : 0;

    const conversion_rate =
      userCount.user_count > 0
        ? converted_unique_users / userCount.user_count
        : 0;

    return {
      treatment_name: userCount.treatment_name,
      total_conversion_events,
      conversion_rate,
    };
  });

  return {
    experiment,
    userCountsResult,
    treatmentEventsResult,
    conversionStats,
  };
}

export default function ExperimentReportPage() {
  return <ExperimentView />;
}
