# SearchWithinGroupResults


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**bookmarks** | [**List[ScoreChunkDTO]**](ScoreChunkDTO.md) |  | 
**corrected_query** | **str** |  | [optional] 
**group** | [**ChunkGroupAndFileId**](ChunkGroupAndFileId.md) |  | 
**total_pages** | **int** |  | 

## Example

```python
from trieve_py_client.models.search_within_group_results import SearchWithinGroupResults

# TODO update the JSON string below
json = "{}"
# create an instance of SearchWithinGroupResults from a JSON string
search_within_group_results_instance = SearchWithinGroupResults.from_json(json)
# print the JSON string representation of the object
print(SearchWithinGroupResults.to_json())

# convert the object into a dict
search_within_group_results_dict = search_within_group_results_instance.to_dict()
# create an instance of SearchWithinGroupResults from a dict
search_within_group_results_form_dict = search_within_group_results.from_dict(search_within_group_results_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


