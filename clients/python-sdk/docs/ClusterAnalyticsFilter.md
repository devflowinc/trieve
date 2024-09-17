# ClusterAnalyticsFilter


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**date_range** | [**DateRange**](DateRange.md) |  | [optional] 

## Example

```python
from trieve_py_client.models.cluster_analytics_filter import ClusterAnalyticsFilter

# TODO update the JSON string below
json = "{}"
# create an instance of ClusterAnalyticsFilter from a JSON string
cluster_analytics_filter_instance = ClusterAnalyticsFilter.from_json(json)
# print the JSON string representation of the object
print(ClusterAnalyticsFilter.to_json())

# convert the object into a dict
cluster_analytics_filter_dict = cluster_analytics_filter_instance.to_dict()
# create an instance of ClusterAnalyticsFilter from a dict
cluster_analytics_filter_form_dict = cluster_analytics_filter.from_dict(cluster_analytics_filter_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


