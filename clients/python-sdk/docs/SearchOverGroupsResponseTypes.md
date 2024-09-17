# SearchOverGroupsResponseTypes


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**corrected_query** | **str** |  | [optional] 
**id** | **str** |  | 
**results** | [**List[SearchOverGroupsResults]**](SearchOverGroupsResults.md) |  | 
**total_pages** | **int** |  | 
**group_chunks** | [**List[GroupScoreChunk]**](GroupScoreChunk.md) |  | 
**total_chunk_pages** | **int** |  | 

## Example

```python
from trieve_py_client.models.search_over_groups_response_types import SearchOverGroupsResponseTypes

# TODO update the JSON string below
json = "{}"
# create an instance of SearchOverGroupsResponseTypes from a JSON string
search_over_groups_response_types_instance = SearchOverGroupsResponseTypes.from_json(json)
# print the JSON string representation of the object
print(SearchOverGroupsResponseTypes.to_json())

# convert the object into a dict
search_over_groups_response_types_dict = search_over_groups_response_types_instance.to_dict()
# create an instance of SearchOverGroupsResponseTypes from a dict
search_over_groups_response_types_form_dict = search_over_groups_response_types.from_dict(search_over_groups_response_types_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


