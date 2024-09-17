# ClusterAnalyticsResponse


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**clusters** | [**List[SearchClusterTopics]**](SearchClusterTopics.md) |  | 
**queries** | [**List[SearchQueryEvent]**](SearchQueryEvent.md) |  | 

## Example

```python
from trieve_py_client.models.cluster_analytics_response import ClusterAnalyticsResponse

# TODO update the JSON string below
json = "{}"
# create an instance of ClusterAnalyticsResponse from a JSON string
cluster_analytics_response_instance = ClusterAnalyticsResponse.from_json(json)
# print the JSON string representation of the object
print(ClusterAnalyticsResponse.to_json())

# convert the object into a dict
cluster_analytics_response_dict = cluster_analytics_response_instance.to_dict()
# create an instance of ClusterAnalyticsResponse from a dict
cluster_analytics_response_form_dict = cluster_analytics_response.from_dict(cluster_analytics_response_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


