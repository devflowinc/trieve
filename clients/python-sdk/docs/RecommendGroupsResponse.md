# RecommendGroupsResponse


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**id** | **str** |  | 
**results** | [**List[SearchOverGroupsResults]**](SearchOverGroupsResults.md) |  | 
**file_id** | **str** |  | [optional] 
**group_created_at** | **datetime** |  | 
**group_dataset_id** | **str** |  | 
**group_description** | **str** |  | [optional] 
**group_id** | **str** |  | 
**group_metadata** | **object** |  | [optional] 
**group_name** | **str** |  | [optional] 
**group_tag_set** | **List[Optional[str]]** |  | [optional] 
**group_tracking_id** | **str** |  | [optional] 
**group_updated_at** | **datetime** |  | 
**metadata** | [**List[ScoreChunkDTO]**](ScoreChunkDTO.md) |  | 

## Example

```python
from trieve_py_client.models.recommend_groups_response import RecommendGroupsResponse

# TODO update the JSON string below
json = "{}"
# create an instance of RecommendGroupsResponse from a JSON string
recommend_groups_response_instance = RecommendGroupsResponse.from_json(json)
# print the JSON string representation of the object
print(RecommendGroupsResponse.to_json())

# convert the object into a dict
recommend_groups_response_dict = recommend_groups_response_instance.to_dict()
# create an instance of RecommendGroupsResponse from a dict
recommend_groups_response_form_dict = recommend_groups_response.from_dict(recommend_groups_response_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


