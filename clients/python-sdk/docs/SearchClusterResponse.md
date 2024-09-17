# SearchClusterResponse


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**clusters** | [**List[SearchClusterTopics]**](SearchClusterTopics.md) |  | 

## Example

```python
from trieve_py_client.models.search_cluster_response import SearchClusterResponse

# TODO update the JSON string below
json = "{}"
# create an instance of SearchClusterResponse from a JSON string
search_cluster_response_instance = SearchClusterResponse.from_json(json)
# print the JSON string representation of the object
print(SearchClusterResponse.to_json())

# convert the object into a dict
search_cluster_response_dict = search_cluster_response_instance.to_dict()
# create an instance of SearchClusterResponse from a dict
search_cluster_response_form_dict = search_cluster_response.from_dict(search_cluster_response_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


