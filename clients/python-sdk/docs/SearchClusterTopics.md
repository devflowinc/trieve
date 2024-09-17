# SearchClusterTopics


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**avg_score** | **float** |  | 
**created_at** | **str** |  | 
**dataset_id** | **str** |  | 
**density** | **int** |  | 
**id** | **str** |  | 
**topic** | **str** |  | 

## Example

```python
from trieve_py_client.models.search_cluster_topics import SearchClusterTopics

# TODO update the JSON string below
json = "{}"
# create an instance of SearchClusterTopics from a JSON string
search_cluster_topics_instance = SearchClusterTopics.from_json(json)
# print the JSON string representation of the object
print(SearchClusterTopics.to_json())

# convert the object into a dict
search_cluster_topics_dict = search_cluster_topics_instance.to_dict()
# create an instance of SearchClusterTopics from a dict
search_cluster_topics_form_dict = search_cluster_topics.from_dict(search_cluster_topics_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


