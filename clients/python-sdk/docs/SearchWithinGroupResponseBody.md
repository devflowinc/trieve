# SearchWithinGroupResponseBody


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**chunks** | [**List[ScoreChunk]**](ScoreChunk.md) |  | 
**corrected_query** | **str** |  | [optional] 
**id** | **str** |  | 
**total_pages** | **int** |  | 

## Example

```python
from trieve_py_client.models.search_within_group_response_body import SearchWithinGroupResponseBody

# TODO update the JSON string below
json = "{}"
# create an instance of SearchWithinGroupResponseBody from a JSON string
search_within_group_response_body_instance = SearchWithinGroupResponseBody.from_json(json)
# print the JSON string representation of the object
print(SearchWithinGroupResponseBody.to_json())

# convert the object into a dict
search_within_group_response_body_dict = search_within_group_response_body_instance.to_dict()
# create an instance of SearchWithinGroupResponseBody from a dict
search_within_group_response_body_form_dict = search_within_group_response_body.from_dict(search_within_group_response_body_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


