# ClusterAnalytics


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**filter** | [**ClusterAnalyticsFilter**](ClusterAnalyticsFilter.md) |  | [optional] 
**type** | **str** |  | 
**cluster_id** | **str** |  | 
**page** | **int** |  | [optional] 

## Example

```python
from trieve_py_client.models.cluster_analytics import ClusterAnalytics

# TODO update the JSON string below
json = "{}"
# create an instance of ClusterAnalytics from a JSON string
cluster_analytics_instance = ClusterAnalytics.from_json(json)
# print the JSON string representation of the object
print(ClusterAnalytics.to_json())

# convert the object into a dict
cluster_analytics_dict = cluster_analytics_instance.to_dict()
# create an instance of ClusterAnalytics from a dict
cluster_analytics_form_dict = cluster_analytics.from_dict(cluster_analytics_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


