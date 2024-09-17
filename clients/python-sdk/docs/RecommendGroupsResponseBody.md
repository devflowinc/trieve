# RecommendGroupsResponseBody


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**id** | **str** |  | 
**results** | [**List[SearchOverGroupsResults]**](SearchOverGroupsResults.md) |  | 

## Example

```python
from trieve_py_client.models.recommend_groups_response_body import RecommendGroupsResponseBody

# TODO update the JSON string below
json = "{}"
# create an instance of RecommendGroupsResponseBody from a JSON string
recommend_groups_response_body_instance = RecommendGroupsResponseBody.from_json(json)
# print the JSON string representation of the object
print(RecommendGroupsResponseBody.to_json())

# convert the object into a dict
recommend_groups_response_body_dict = recommend_groups_response_body_instance.to_dict()
# create an instance of RecommendGroupsResponseBody from a dict
recommend_groups_response_body_form_dict = recommend_groups_response_body.from_dict(recommend_groups_response_body_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


