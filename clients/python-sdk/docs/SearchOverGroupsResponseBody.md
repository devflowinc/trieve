# SearchOverGroupsResponseBody


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**corrected_query** | **str** |  | [optional] 
**id** | **str** |  | 
**results** | [**List[SearchOverGroupsResults]**](SearchOverGroupsResults.md) |  | 
**total_pages** | **int** |  | 

## Example

```python
from trieve_py_client.models.search_over_groups_response_body import SearchOverGroupsResponseBody

# TODO update the JSON string below
json = "{}"
# create an instance of SearchOverGroupsResponseBody from a JSON string
search_over_groups_response_body_instance = SearchOverGroupsResponseBody.from_json(json)
# print the JSON string representation of the object
print(SearchOverGroupsResponseBody.to_json())

# convert the object into a dict
search_over_groups_response_body_dict = search_over_groups_response_body_instance.to_dict()
# create an instance of SearchOverGroupsResponseBody from a dict
search_over_groups_response_body_form_dict = search_over_groups_response_body.from_dict(search_over_groups_response_body_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


