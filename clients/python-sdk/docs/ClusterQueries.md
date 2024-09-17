# ClusterQueries


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**cluster_id** | **str** |  | 
**page** | **int** |  | [optional] 
**type** | **str** |  | 

## Example

```python
from trieve_py_client.models.cluster_queries import ClusterQueries

# TODO update the JSON string below
json = "{}"
# create an instance of ClusterQueries from a JSON string
cluster_queries_instance = ClusterQueries.from_json(json)
# print the JSON string representation of the object
print(ClusterQueries.to_json())

# convert the object into a dict
cluster_queries_dict = cluster_queries_instance.to_dict()
# create an instance of ClusterQueries from a dict
cluster_queries_form_dict = cluster_queries.from_dict(cluster_queries_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


