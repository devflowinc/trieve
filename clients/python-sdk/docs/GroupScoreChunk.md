# GroupScoreChunk


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
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
from trieve_py_client.models.group_score_chunk import GroupScoreChunk

# TODO update the JSON string below
json = "{}"
# create an instance of GroupScoreChunk from a JSON string
group_score_chunk_instance = GroupScoreChunk.from_json(json)
# print the JSON string representation of the object
print(GroupScoreChunk.to_json())

# convert the object into a dict
group_score_chunk_dict = group_score_chunk_instance.to_dict()
# create an instance of GroupScoreChunk from a dict
group_score_chunk_form_dict = group_score_chunk.from_dict(group_score_chunk_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


