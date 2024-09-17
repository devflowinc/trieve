# ChunkMetadataWithScore


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**chunk_html** | **str** |  | [optional] 
**created_at** | **datetime** |  | 
**dataset_id** | **str** |  | 
**id** | **str** |  | 
**link** | **str** |  | [optional] 
**metadata** | **object** |  | [optional] 
**score** | **float** |  | 
**tag_set** | **str** |  | [optional] 
**time_stamp** | **datetime** |  | [optional] 
**tracking_id** | **str** |  | [optional] 
**updated_at** | **datetime** |  | 
**weight** | **float** |  | 

## Example

```python
from trieve_py_client.models.chunk_metadata_with_score import ChunkMetadataWithScore

# TODO update the JSON string below
json = "{}"
# create an instance of ChunkMetadataWithScore from a JSON string
chunk_metadata_with_score_instance = ChunkMetadataWithScore.from_json(json)
# print the JSON string representation of the object
print(ChunkMetadataWithScore.to_json())

# convert the object into a dict
chunk_metadata_with_score_dict = chunk_metadata_with_score_instance.to_dict()
# create an instance of ChunkMetadataWithScore from a dict
chunk_metadata_with_score_form_dict = chunk_metadata_with_score.from_dict(chunk_metadata_with_score_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


