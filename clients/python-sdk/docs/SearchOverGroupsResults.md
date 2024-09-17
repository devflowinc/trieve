# SearchOverGroupsResults


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**chunks** | [**List[ScoreChunk]**](ScoreChunk.md) |  | 
**file_id** | **str** |  | [optional] 
**group** | [**ChunkGroup**](ChunkGroup.md) |  | 

## Example

```python
from trieve_py_client.models.search_over_groups_results import SearchOverGroupsResults

# TODO update the JSON string below
json = "{}"
# create an instance of SearchOverGroupsResults from a JSON string
search_over_groups_results_instance = SearchOverGroupsResults.from_json(json)
# print the JSON string representation of the object
print(SearchOverGroupsResults.to_json())

# convert the object into a dict
search_over_groups_results_dict = search_over_groups_results_instance.to_dict()
# create an instance of SearchOverGroupsResults from a dict
search_over_groups_results_form_dict = search_over_groups_results.from_dict(search_over_groups_results_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


