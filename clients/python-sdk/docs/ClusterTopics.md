# ClusterTopics


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**filter** | [**ClusterAnalyticsFilter**](ClusterAnalyticsFilter.md) |  | [optional] 
**type** | **str** |  | 

## Example

```python
from trieve_py_client.models.cluster_topics import ClusterTopics

# TODO update the JSON string below
json = "{}"
# create an instance of ClusterTopics from a JSON string
cluster_topics_instance = ClusterTopics.from_json(json)
# print the JSON string representation of the object
print(ClusterTopics.to_json())

# convert the object into a dict
cluster_topics_dict = cluster_topics_instance.to_dict()
# create an instance of ClusterTopics from a dict
cluster_topics_form_dict = cluster_topics.from_dict(cluster_topics_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


