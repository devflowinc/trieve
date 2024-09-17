# SearchGroupResponseTypes


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**chunks** | [**List[ScoreChunk]**](ScoreChunk.md) |  | 
**corrected_query** | **str** |  | [optional] 
**id** | **str** |  | 
**total_pages** | **int** |  | 
**bookmarks** | [**List[ScoreChunkDTO]**](ScoreChunkDTO.md) |  | 
**group** | [**ChunkGroupAndFileId**](ChunkGroupAndFileId.md) |  | 

## Example

```python
from trieve_py_client.models.search_group_response_types import SearchGroupResponseTypes

# TODO update the JSON string below
json = "{}"
# create an instance of SearchGroupResponseTypes from a JSON string
search_group_response_types_instance = SearchGroupResponseTypes.from_json(json)
# print the JSON string representation of the object
print(SearchGroupResponseTypes.to_json())

# convert the object into a dict
search_group_response_types_dict = search_group_response_types_instance.to_dict()
# create an instance of SearchGroupResponseTypes from a dict
search_group_response_types_form_dict = search_group_response_types.from_dict(search_group_response_types_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


