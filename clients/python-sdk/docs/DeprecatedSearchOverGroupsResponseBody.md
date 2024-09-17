# DeprecatedSearchOverGroupsResponseBody


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**corrected_query** | **str** |  | [optional] 
**group_chunks** | [**List[GroupScoreChunk]**](GroupScoreChunk.md) |  | 
**total_chunk_pages** | **int** |  | 

## Example

```python
from trieve_py_client.models.deprecated_search_over_groups_response_body import DeprecatedSearchOverGroupsResponseBody

# TODO update the JSON string below
json = "{}"
# create an instance of DeprecatedSearchOverGroupsResponseBody from a JSON string
deprecated_search_over_groups_response_body_instance = DeprecatedSearchOverGroupsResponseBody.from_json(json)
# print the JSON string representation of the object
print(DeprecatedSearchOverGroupsResponseBody.to_json())

# convert the object into a dict
deprecated_search_over_groups_response_body_dict = deprecated_search_over_groups_response_body_instance.to_dict()
# create an instance of DeprecatedSearchOverGroupsResponseBody from a dict
deprecated_search_over_groups_response_body_form_dict = deprecated_search_over_groups_response_body.from_dict(deprecated_search_over_groups_response_body_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


