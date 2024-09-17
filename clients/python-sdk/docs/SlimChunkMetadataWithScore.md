# SlimChunkMetadataWithScore


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**created_at** | **datetime** |  | 
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
from trieve_py_client.models.slim_chunk_metadata_with_score import SlimChunkMetadataWithScore

# TODO update the JSON string below
json = "{}"
# create an instance of SlimChunkMetadataWithScore from a JSON string
slim_chunk_metadata_with_score_instance = SlimChunkMetadataWithScore.from_json(json)
# print the JSON string representation of the object
print(SlimChunkMetadataWithScore.to_json())

# convert the object into a dict
slim_chunk_metadata_with_score_dict = slim_chunk_metadata_with_score_instance.to_dict()
# create an instance of SlimChunkMetadataWithScore from a dict
slim_chunk_metadata_with_score_form_dict = slim_chunk_metadata_with_score.from_dict(slim_chunk_metadata_with_score_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


